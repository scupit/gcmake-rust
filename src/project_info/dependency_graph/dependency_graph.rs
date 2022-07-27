use std::{cell::RefCell, rc::{Rc, Weak}, hash::{Hash, Hasher}, collections::{HashMap, HashSet}};

use crate::project_info::{LinkMode, link_spec_parser::LinkAccessMode, CompiledOutputItem, PreBuildScript, OutputItemLinks, final_project_data::FinalProjectData, final_dependencies::{FinalGCMakeDependency, FinalPredefinedDependencyConfig, GCMakeDependencyStatus}, LinkSpecifier, FinalProjectType};

use super::hash_wrapper::RcRefcHashWrapper;

enum SimpleOutputType {
  Executable,
  Library
}

type TargetId = i32;
type ProjectId = usize;

#[derive(Clone, Hash, PartialEq, Eq)]
struct ProjectGroupId(usize);

enum LinkResolutionFailureReason {
  PointsToInvalidOrNonexistentProject,
  NestedNamespaceInOtherProjectContext,
  PreBuildScriptLinksToUpperOrEqualLevel,
  DependencyCycle(Vec<Rc<RefCell<TargetNode>>>),
  WrongUserGivenPredefLinkMode {
    current_link_mode: LinkMode,
    needed_link_mode: LinkMode,
    target: Rc<RefCell<TargetNode>>,
    dependency: Rc<RefCell<TargetNode>>
  },
  LinkedInMultipleCategories {
    current_link_mode: LinkMode,
    attempted_link_mode: LinkMode,
    link_receiver_project: ProjectWrapper,
    link_receiver_name: String,
    link_giver_project: ProjectWrapper,
    link_giver_name: String,
  },
  LinkedToSelf {
    project: ProjectWrapper,
    target_name: String
  },
  NotALinkTarget {
    project: ProjectWrapper,
    target_name: String
  },
  AccessNotAllowed {
    project: ProjectWrapper,
    target_name: String,
    given_access_mode: LinkAccessMode
  },
  NotFound {
    project: ProjectWrapper,
    target_name: String
  }
}

// TODO: When writing these, I need to somehow specify whether the target needs
// to be linked using the individual targets themselves, or through a variable (like wxWidgets).
// This is determined by the dependency project the linked target is a member of.
struct Link {
  target_name: String,
  link_mode: LinkMode,
  target: Weak<RefCell<TargetNode>>
}

impl Link {
  pub fn new(
    target_name: String,
    target: Weak<RefCell<TargetNode>>,
    link_mode: LinkMode
  ) -> Self {
    
    Weak::upgrade(&target).unwrap().as_ref().borrow_mut().linked_to_count += 1;

    Self {
      target_name,
      target,
      link_mode
    }
  }

  fn target_id(&self) -> TargetId {
    Weak::upgrade(&self.target).unwrap().as_ref().borrow().unique_target_id()
  }
}

impl Hash for Link {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.target_id().hash(state);
  }
}

impl PartialEq for Link {
  fn eq(&self, other: &Self) -> bool {
    return self.target_id() == other.target_id();
  }
}

impl Eq for Link { }

enum ContainedItem<'a> {
  CompiledOutput(&'a CompiledOutputItem),
  PredefinedLibrary(String),
  PreBuild(&'a PreBuildScript)
}

struct TargetNode {
  name: String,
  the_unique_id: TargetId,
  can_link_to: bool,
  linked_to_count: i32,
  contained_in_graph: Weak<DependencyGraph>,
  output_type: SimpleOutputType,
  visibility: LinkAccessMode,
  // depends_on: HashSet<Link>,
  depends_on: HashMap<TargetId, Link>,
  // TODO: This doesn't need to be a copy. This is just easier to use, for now.
  raw_link_specifiers: Option<OutputItemLinks>
}

impl TargetNode {
  pub fn new(
    id_var: &mut TargetId,
    name: impl AsRef<str>,
    parent_graph: Weak<DependencyGraph>,
    contained_item: ContainedItem,
    visibility: LinkAccessMode,
    can_link_to: bool
  ) -> Self {
    let unique_id: TargetId = *id_var;
    *id_var = unique_id + 1;

    let output_type: SimpleOutputType;
    let raw_link_specifiers: Option<OutputItemLinks>;

    match contained_item {
      ContainedItem::PredefinedLibrary(_) => {
        raw_link_specifiers = None;
        output_type = SimpleOutputType::Library;
      },
      ContainedItem::CompiledOutput(output_item) => {
        raw_link_specifiers = Some(output_item.get_links().clone());
        output_type = if output_item.is_library_type()
          { SimpleOutputType::Library }
          else { SimpleOutputType::Executable }
      },
      ContainedItem::PreBuild(pre_build) => match pre_build {
        PreBuildScript::Exe(pre_build_exe) => {
          raw_link_specifiers = Some(pre_build_exe.get_links().clone());
          output_type = SimpleOutputType::Executable
        },
        PreBuildScript::Python(_) => {
          raw_link_specifiers = None;
          // This is just a placeholder. Not sure if this will cause issues yet, but it shouldn't.
          output_type = SimpleOutputType::Executable;
        }
      }
    }
    
    return Self {
      the_unique_id: unique_id,
      name: name.as_ref().to_string(),
      contained_in_graph: parent_graph,
      output_type,
      visibility,
      // depends_on: HashSet::new(),
      depends_on: HashMap::new(),
      can_link_to,
      raw_link_specifiers,
      linked_to_count: 0
    }
  }

  pub fn unique_target_id(&self) -> TargetId {
    self.the_unique_id
  }

  pub fn container_project_id(&self) -> ProjectId {
    self.container_project().graph_id
  }

  pub fn container_project_group_id(&self) -> ProjectGroupId {
    self.container_project().project_group_id.clone()
  }

  fn container_project(&self) -> Rc<DependencyGraph> {
    return Weak::upgrade(&self.contained_in_graph).unwrap();
  }

  pub fn insert_link(&mut self, link: Link) {
    self.depends_on.insert(
      link.target_id(),
      link
    );
  }
}

impl Hash for TargetNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.the_unique_id.hash(state);
  }
}

impl PartialEq for TargetNode {
  fn eq(&self, other: &Self) -> bool {
    self.unique_target_id() == other.unique_target_id()
  }
}

impl Eq for TargetNode { }

// NOTE: It's fine to keep this clone implementation, since it only clones the contained RCs. 
#[derive(Clone)]
enum ProjectWrapper {
  NormalProject(Rc<FinalProjectData>),
  GCMakeDependency(Rc<FinalGCMakeDependency>),
  PredefinedDependency(Rc<FinalPredefinedDependencyConfig>)
}

enum CycleCheckResult {
  Cycle(Vec<RcRefcHashWrapper<TargetNode>>),
  AllUsedTargets(HashSet<RcRefcHashWrapper<TargetNode>>)
}

struct DependencyGraphInfoWrapper {
  dep_graph: Rc<DependencyGraph>,
  sorted_info: OrderedTargetInfo
}

// TODO: Allow predefined dependencies to influence target ordering. For instance, SFML
// targets must be linked in a certain order to work. The SFML predefined configuration
// should be allowed to specify that its 'window' target depends on 'system', and so on.
// Essentially, the configuration should be able to contain a graph-like representation
// of how the dependency's targets depend on each other. After ensuring the graph is correct,
// targets in that library which depend on other targets will be sorted lower in the list
// than the targets they depend on.
pub struct DependencyGraph {
  parent: Option<Weak<DependencyGraph>>,
  toplevel: Weak<DependencyGraph>,
  current_graph_ref: Weak<DependencyGraph>,

  graph_id: ProjectId,
  
  // Test projects should have the same group ID as their parent.
  // This is necessary when sorting targets because we sometimes need to traverse
  // upward to find all targets in a project group which don't depend on any other targets made by
  // that project group. This ensures that all targets in a project are iterated through in one pass,
  // which is important. 
  project_group_id: ProjectGroupId,

  project_wrapper: ProjectWrapper,
  targets: RefCell<HashMap<String, Rc<RefCell<TargetNode>>>>,
  pre_build_wrapper: Option<Rc<RefCell<TargetNode>>>,

  subprojects: HashMap<String, Rc<DependencyGraph>>,
  test_projects: HashMap<String, Rc<DependencyGraph>>,
  gcmake_deps: HashMap<String, Rc<DependencyGraph>>,

  predefined_deps: HashMap<String, Rc<DependencyGraph>>
}

impl Hash for DependencyGraph {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.graph_id.hash(state);
  }
}

impl PartialEq for DependencyGraph {
  fn eq(&self, other: &Self) -> bool {
    self.graph_id == other.graph_id
  }
}

impl Eq for DependencyGraph { }

impl DependencyGraph {
  pub fn new_info_from_toplevel(
    toplevel_project: &Rc<FinalProjectData>
  ) -> Result<DependencyGraphInfoWrapper, LinkResolutionFailureReason> {
    let mut target_id_counter: TargetId = 0;
    let mut toplevel_tree_id_counter: ProjectId = 0;

    let mut full_graph: Rc<Self> = Self::recurse_root_project(
      &mut target_id_counter,
      &mut toplevel_tree_id_counter,
      toplevel_project
    );

    full_graph.make_given_link_associations(&mut target_id_counter)?;
    full_graph.make_auto_inner_project_link_associations()?;
    full_graph.ensure_proper_predefined_dep_links()?;

    let all_used_targets: HashSet<RcRefcHashWrapper<TargetNode>>;
    
    match full_graph.find_cycle() {
      CycleCheckResult::AllUsedTargets(all_used) => all_used,
      CycleCheckResult::Cycle(cycle_vec) => {
        return Err(LinkResolutionFailureReason::DependencyCycle(
          cycle_vec
            .into_iter()
            .map(|wrapped_target_node| wrapped_target_node.unwrap())
            .collect()
        ))
      }
    };

    // Now we have the full set of used nodes, and know for sure there are no cycles.

    // TODO: Find all DAG subgraphs, then use those to produce a correctly sorted target
    // list and project ordering.

    return Ok(DependencyGraphInfoWrapper {
      sorted_info: sorted_target_info(&all_used_targets),
      dep_graph: full_graph
    });
  }


  // See the python prototype project for details.
  // D:\Personal_Projects\Coding\prototyping\python\dependency-graph-sorting
  fn find_cycle(&self) -> CycleCheckResult {
    let mut visited: HashSet<RcRefcHashWrapper<TargetNode>> = HashSet::new();
    let mut stack: Vec<RcRefcHashWrapper<TargetNode>> = Vec::new();

    if let Some(cycle_vec) = self.do_find_cycle(&mut visited, &mut stack) {
      return CycleCheckResult::Cycle(cycle_vec);
    }

    // As of this point, 'visited' contains the set of all targets used in the entire build tree.
    return CycleCheckResult::AllUsedTargets(visited);
  }

  fn do_find_cycle(
    &self,
    visited: &mut HashSet<RcRefcHashWrapper<TargetNode>>,
    stack: &mut Vec<RcRefcHashWrapper<TargetNode>>
  ) -> Option<Vec<RcRefcHashWrapper<TargetNode>>> {
    if let Some(pre_build) = &self.pre_build_wrapper {
      if let Some(cycle_vec) = self.do_find_cycle_helper(pre_build, &mut visited, &mut stack) {
        return Some(cycle_vec);
      }
    }

    for (_, target_node) in self.targets.borrow().iter() {
      let wrapped_target_node: RcRefcHashWrapper<TargetNode> = RcRefcHashWrapper(Rc::clone(target_node));

      if !visited.contains(&wrapped_target_node) {
        if let Some(cycle_vec) = self.do_find_cycle_helper(target_node, &mut visited, &mut stack) {
          return Some(cycle_vec);
        }
      }
    }

    for (_, test_project) in &self.test_projects {
      if let Some(cycle_vec) = self.do_find_cycle(&mut visited, &mut stack) {
        return Some(cycle_vec);
      }
    }

    for (_, subproject) in &self.subprojects {
      if let Some(cycle_vec) = self.do_find_cycle(&mut visited, &mut stack) {
        return Some(cycle_vec);
      }
    }

    for (_, gcmake_dep) in &self.gcmake_deps {
      if let Some(cycle_vec) = self.do_find_cycle(&mut visited, &mut stack) {
        return Some(cycle_vec);
      }
    }

    return None;
  }

  fn do_find_cycle_helper(
    &self,
    node: &Rc<RefCell<TargetNode>>,
    visited: &mut HashSet<RcRefcHashWrapper<TargetNode>>,
    stack: &mut Vec<RcRefcHashWrapper<TargetNode>>
  ) -> Option<Vec<RcRefcHashWrapper<TargetNode>>> {
    stack.push(RcRefcHashWrapper(Rc::clone(node)));
    visited.insert(RcRefcHashWrapper(Rc::clone(node)));

    for (_, dep_link) in &node.as_ref().borrow().depends_on {
      let dependency_node: RcRefcHashWrapper<TargetNode> =
        RcRefcHashWrapper(Weak::upgrade(&dep_link.target).unwrap());

      if visited.contains(&dependency_node) && stack.contains(&dependency_node) {
        return Some(stack.clone());
      }
      else if let Some(cycle_vec) = self.do_find_cycle_helper(&dependency_node, &mut visited, &mut stack) {
        return Some(stack.clone());
      }
    }

    stack.pop();
    return None;
  }

  fn root_graph_id(&self) -> usize {
    return Weak::upgrade(&self.toplevel).unwrap().graph_id;
  }

  fn has_any_pre_build(&self) -> bool {
    return self.pre_build_wrapper.is_some();
  }

  fn pre_build_script_name() -> &'static str {
    "Pre-build script"
  }

  /*
    TODO: After making associations, ensure correct predefined dependency inclusion for all
    targets (tests exes, project outputs, and pre-build script) for all non-predefined-dependency projects.
    Although it shouldn't be possible to create cycles while doing this, do it before cycle detection
    anyways just in case. It doesn't hurt to have that extra layer of checking.

    For example: several wxWidgets targets depend directly on the wxWidgets 'base' target. However, it is 
    possible specify a link to those targets without specifying a link to wxWidgets 'base'.
    For library projects it is also possible to specify a link to wxWidgets 'base' using the wrong
    link inheritance category (i.e.  specifying 'base' as a private link, when 'core' (requires 'base')
    is specified as a public one).

    From here on out, let's call these transitively needed  predefined dependency targets "requirements"
    and their dependents "predefined dependents".
    
    For each linked 'predefined dependent' in a target, recurse through the requirements of that dependency.
      - If the user didn't specify a link to the requirement in the target, then create a link from the
          target to the requirement using the same link category permissions (PUBLIC, PRIVATE, INTERFACE)
          as the predefined dependent link.
      - If the requirement link already exists because it was added by code (see above), then modify its link
          category to be the more permissive of the category of [the existing requirement link, or the
          requirement link which is about to be created].
      - If the requirement link already exists because it was given by the user, make sure its link category
          is equally or more permissive than the link which would be created otherwise. Return an error
          message if this is not the case.
  */
  fn ensure_proper_predefined_dep_links(&self) -> Result<(), LinkResolutionFailureReason> {
    for (_, target_rc) in self.targets.borrow().iter() {
      let project_output_target: &mut TargetNode = &mut target_rc.as_ref().borrow_mut();

      // This is necessary because adding links to the project target inside the loop could mess with
      // the list's iteration. 
      let mut links_to_add: HashMap<TargetId, Link> = HashMap::new();
    
      for (_, link) in &project_output_target.depends_on {
        let link_target = Weak::upgrade(&link.target).unwrap().as_ref().borrow();
        let link_target_graph = Weak::upgrade(&link_target.contained_in_graph).unwrap();

        if let ProjectWrapper::PredefinedDependency(_) = &link_target_graph.project_wrapper {
          let mut checked_predef_targets: HashMap<TargetId, Rc<RefCell<TargetNode>>> = HashMap::new();
          let mut predef_targets_checking_stack: Vec<(TargetId, Rc<RefCell<TargetNode>>)> = Vec::new();

          for (predef_target_id, predef_requirement_target) in &link_target.depends_on {
            predef_targets_checking_stack.push(
              (
                *predef_target_id,
                Weak::upgrade(&predef_requirement_target.target).unwrap()
              )
            );
          }

          while let Some((target_checking_id, target_checking_rc)) = predef_targets_checking_stack.pop() {
            let predef_target_checking: &TargetNode = &target_checking_rc.as_ref().borrow();

            checked_predef_targets.insert(
              target_checking_id,
              Rc::clone(&target_checking_rc)
            );

            // Essentially recurse nested requirements
            for (nested_requirement_id, nested_requirement) in &predef_target_checking.depends_on {
              let is_requirement_in_stack: bool = predef_targets_checking_stack.iter()
                .find(|(id_finding, _)| id_finding == nested_requirement_id)
                .is_some();

              if !is_requirement_in_stack && !checked_predef_targets.contains_key(nested_requirement_id) {
                predef_targets_checking_stack.push(
                  (
                    *nested_requirement_id,
                    Weak::upgrade(&nested_requirement.target).unwrap()
                  )
                );
              }
            }

            if let Some(existing_link_to_add) = links_to_add.get_mut(&target_checking_id) {
              // The link already exists and was added by code. Use the most permissive of the two
              // link modes.
              existing_link_to_add.link_mode = LinkMode::more_permissive(
                existing_link_to_add.link_mode,
                link.link_mode.clone()
              );
            }
            else if let Some(existing_link) = project_output_target.depends_on.get(&target_checking_id) {
              // The link already exists and was added by the user. Return an error if the existing link mode
              // is not the same as the one which would be created.
              if existing_link.link_mode != link.link_mode {
                return Err(LinkResolutionFailureReason::WrongUserGivenPredefLinkMode {
                  current_link_mode: existing_link.link_mode.clone(),
                  needed_link_mode: link.link_mode.clone(),
                  target: Weak::upgrade(&link.target).unwrap(),
                  dependency: Rc::clone(&target_rc)
                });
              }
            }
            else {
              // The link is not present. Just add it to links_to_add.
              links_to_add.insert(
                target_checking_id,
                Link::new(
                  predef_target_checking.name.clone(),
                  Rc::downgrade(&target_checking_rc),
                  link.link_mode.clone()
                )
              );
            }
          }
        }
      }

      let mut mut_target: &mut TargetNode = &mut target_rc.as_ref().borrow_mut();

      for (_, link) in links_to_add {
        mut_target.insert_link(link);
      }
    }
    
    for (_, subproject) in &self.subprojects {
      subproject.ensure_proper_predefined_dep_links()?;
    }

    for (_, test_project) in &self.test_projects {
      test_project.ensure_proper_predefined_dep_links()?;
    }

    for (_, gcmake_dep) in &self.gcmake_deps {
      gcmake_dep.ensure_proper_predefined_dep_links()?;
    }

    // This is not done for predefined dependencies because their interdependent target
    // links are already made upon being loaded into the map.

    Ok(())
  }

  // Makes these associations within a project:
  //    Tests -> project outputs -> pre-build
  // Which ensures that pre-build scripts are built before project outputs, and project outputs are
  // built before tests. Also ensures that all tests in a project depend on all immediate outputs
  // of the project.
  fn make_auto_inner_project_link_associations(&self) -> Result<(), LinkResolutionFailureReason> {
    if let Some(pre_build_target) = &self.pre_build_wrapper {
      // All project output targets must depend on the project's pre-build script in order
      // for project targets to be ordered and checked for cycles correctly.
      for (_, project_output_rc) in self.targets.borrow().iter() {
        let project_output_target: &mut TargetNode = &mut project_output_rc.as_ref().borrow_mut();

        project_output_target.insert_link(Link::new(
          Self::pre_build_script_name().to_string(),
          Rc::downgrade(pre_build_target),
          LinkMode::Private
        ));
      }
    }

    for (_, test_project) in &self.test_projects {
      // Each target in a test project must depend on every target output from the project.
      // This is because tests should be able to make use of all code used by executables in the
      // project (or for libraries, make use of the library). As a result, all tests must be built after
      // project output.
      for (_, test_target_rc) in test_project.targets.borrow().iter() {
        let test_target: &mut TargetNode = &mut test_target_rc.as_ref().borrow_mut();

        for (project_output_name, project_output_rc) in self.targets.borrow().iter() {
          test_target.insert_link(Link::new(
            project_output_name.to_string(),
            Rc::downgrade(project_output_rc),
            LinkMode::Private
          ));
        }
      }

      test_project.make_auto_inner_project_link_associations()?;
    }

    for (_, subproject) in &self.subprojects {
      subproject.make_auto_inner_project_link_associations()?;
    }

    for (_, gcmake_dep_project) in &self.gcmake_deps {
      gcmake_dep_project.make_auto_inner_project_link_associations()?;
    }

    Ok(())
  }

  fn make_given_link_associations(
    &self,
    // Needed for creating placeholder targets in gcmake dependency projects which haven't been cloned yet.
    target_id_counter: &mut i32
  ) -> Result<(), LinkResolutionFailureReason> {
    for (link_receiver_name, target_container) in self.targets.borrow().iter() {
      self.resolve_and_apply_target_links(
        target_id_counter,
        Rc::clone(target_container),
        link_receiver_name,
        false
      )?;
    }

    if let Some(pre_build_target) = &self.pre_build_wrapper {
      self.resolve_and_apply_target_links(
        target_id_counter,
        Rc::clone(pre_build_target),
        Self::pre_build_script_name(),
        true
      )?;
    }

    for (_, subproject) in &self.subprojects {
      subproject.make_given_link_associations(target_id_counter)?;
    }

    for (_, test_project) in &self.test_projects {
      test_project.make_given_link_associations(target_id_counter)?;
    }

    // This allows links for an entire GCMake project tree to be checked, including
    // dependencies. This means the available GCMake dependencies can also have their
    // CMake configurations written, although this is not done currently. It probably
    // should be though.
    for (_, gcmake_dep) in &self.gcmake_deps {
      gcmake_dep.make_given_link_associations(target_id_counter)?;
    }

    return Ok(());
  }

  fn resolve_and_apply_target_links(
    &self,
    target_id_counter: &mut i32,
    target_container: Rc<RefCell<TargetNode>>,
    link_receiver_name: &str,
    is_link_receiver_pre_build_script: bool
  ) -> Result<(), LinkResolutionFailureReason> {
    if let Some(link_specs) = target_container.as_ref().borrow().raw_link_specifiers {
      let link_receiver: &mut TargetNode = &mut target_container.as_ref().borrow_mut();

      let public_links: HashSet<Link> = self.resolve_links(
        target_id_counter,
        &link_specs.cmake_public,
        &LinkMode::Public,
      )?;

      self.apply_link_set_to_target(
        link_receiver,
        link_receiver_name,
        public_links,
        is_link_receiver_pre_build_script
      )?;

      let interface_links: HashSet<Link> = self.resolve_links(
        target_id_counter,
        &link_specs.cmake_interface,
        &LinkMode::Interface
      )?;

      self.apply_link_set_to_target(
        link_receiver,
        link_receiver_name,
        interface_links,
        is_link_receiver_pre_build_script
      )?;

      let private_links: HashSet<Link> = self.resolve_links(
        target_id_counter,
        &link_specs.cmake_private,
        &LinkMode::Private
      )?;

      self.apply_link_set_to_target(
        link_receiver,
        link_receiver_name,
        private_links,
        is_link_receiver_pre_build_script
      );
    }

    return Ok(());
  }

  fn apply_link_set_to_target(
    &self,
    link_receiver: &mut TargetNode,
    link_receiver_name: &str,
    link_set: HashSet<Link>,
    is_receiver_pre_build_script: bool
  ) -> Result<(), LinkResolutionFailureReason> {
    for link in link_set {
      let link_giver: &TargetNode = &Weak::upgrade(&link.target).unwrap().as_ref().borrow();
      let link_giver_graph: Rc<DependencyGraph> = Weak::upgrade(&link_giver.contained_in_graph).unwrap();
      let link_receiver_graph: Rc<DependencyGraph> = Weak::upgrade(&link_receiver.contained_in_graph).unwrap();

      // Targets cannot link to themselves.
      if link_receiver.unique_target_id() == link_giver.unique_target_id() { 
        return Err(LinkResolutionFailureReason::LinkedToSelf {
          project: self.project_wrapper.clone(),
          target_name: link_receiver_name.to_string()
        });
      }
      else if let Some(existing_link) = link_receiver.depends_on.get(&link.target_id()) {
        // Targets can only be linked in a single categry. I.E. it doesn't make sense
        // to link a target as both PUBLIC and INTERFACE.
        if existing_link.link_mode != link.link_mode {
          return Err(LinkResolutionFailureReason::LinkedInMultipleCategories {
            current_link_mode: existing_link.link_mode,
            attempted_link_mode: link.link_mode,
            link_giver_project: link_giver_graph.project_wrapper.clone(),
            link_giver_name: link.target_name.to_string(),
            link_receiver_project: link_receiver_graph.project_wrapper.clone(),
            link_receiver_name: link_receiver_name.to_string()
          });
        }
      }
    }

    Ok(())
  }

  fn resolve_links(
    &self,
    target_id_counter: &mut i32,
    link_specs: &Vec<LinkSpecifier>,
    link_mode: &LinkMode
  ) -> Result<HashSet<Link>, LinkResolutionFailureReason> {
    /*
      Resolution scenarios:
        - root::{target, names}
        - parent::{target, names}
          -> Useful when nested subprojects need to depend on each other
        - dependency_name::{target, names}
          -> 'dependency_name' is a placeholder for any string other than 'root' and 'parent'.
    */

    let link_set: HashSet<Link> = HashSet::new();

    for link_spec in link_specs {
      let mut namespace_stack: Vec<String> = link_spec.get_namespace_stack().clone();

      let resolved_links: HashSet<Link> = self.resolve_namespace_helper(
        target_id_counter,
        &mut namespace_stack,
        link_spec.get_target_list(),
        link_mode,
        link_spec.get_access_mode(),
        false
      )?;

      // NOTE: We don't check for conflicting link categories here because this function only resolves
      // one link specifier at a time. Checking for conflicting link categories is likely done from
      // the function which calls this one.
      for link in resolved_links {
        // Prioritize the first specified instance of a link target. When the link specifier contains
        // duplicate target names, the index of the first specified instance of that target is used.
        if !link_set.contains(&link) {
          link_set.insert(link);
        }
      }
    }

    return Ok(link_set);
  }

  fn resolve_namespace_helper(
    &self,
    target_id_counter: &mut i32,
    namespace_stack: &mut Vec<String>,
    target_list: &Vec<String>,
    link_mode: &LinkMode,
    access_mode: &LinkAccessMode,
    is_outside_original_project_context: bool
  ) -> Result<HashSet<Link>, LinkResolutionFailureReason> {
    if namespace_stack.is_empty() {
      let mut accumulated_link_set: HashSet<Link> = HashSet::new();

      for target_name in target_list {
        let resolved_link: Link = self.resolve_target_into_link(
          target_id_counter,
          target_name,
          link_mode,
          access_mode
        )?;

        accumulated_link_set.insert(resolved_link);
      }

      return Ok(accumulated_link_set);
    }
    else {
      // Needed for namespace resolution because only the root project can define predefined_dependencies
      // and gcmake_dependencies.
      let root_project: Rc<DependencyGraph> = Weak::upgrade(&self.toplevel).unwrap();
      let next_namespace: String = namespace_stack.pop().unwrap();

      let next_graph: Rc<DependencyGraph>;
      let will_be_outside_original_project_context: bool;

      match &next_namespace[..] {
        "root" => {
          next_graph = Weak::upgrade(&self.toplevel).unwrap();
          assert!(
            self.root_graph_id() == next_graph.root_graph_id(),
            "Root project tree should not change when resolving to the 'root' graph."
          );
          will_be_outside_original_project_context = false;
        },
        "parent" => match &self.parent {
          Some(parent_graph) => {
            next_graph = Weak::upgrade(parent_graph).unwrap();
            assert!(
              self.root_graph_id() == next_graph.root_graph_id(),
              "Root project tree should not change when resolving to the 'parent' graph. This is because project root graphs (including dependency projects) are not given a parent."
            );
            // Dependency project roots never have a parent graph, and therefore referencing a
            // project's "parent" will never resolve to a context outside that project root's tree.
            will_be_outside_original_project_context = false;
          },
          None => return Err(LinkResolutionFailureReason::PointsToInvalidOrNonexistentProject)
        },
        namespace_to_resolve => {
          if is_outside_original_project_context {
            return Err(LinkResolutionFailureReason::NestedNamespaceInOtherProjectContext)
          }
          else if let Some(matching_subproject) = self.subprojects.get(namespace_to_resolve) {
            next_graph = Rc::clone(matching_subproject);
            assert!(
              self.root_graph_id() == next_graph.root_graph_id(),
              "Root project tree should not change when resolving to a subproject graph."
            );
            will_be_outside_original_project_context = false;
          }
          else if let Some(matching_predef_dep) = root_project.predefined_deps.get(namespace_to_resolve) {
            next_graph = Rc::clone(matching_predef_dep);
            assert!(
              self.root_graph_id() != next_graph.root_graph_id(),
              "Root project tree must change when resolving to a predefined_dependency graph."
            );
            will_be_outside_original_project_context = true;
          }
          else if let Some(gcmake_dep) = root_project.gcmake_deps.get(namespace_to_resolve) {
            next_graph = Rc::clone(gcmake_dep);
            assert!(
              self.root_graph_id() != next_graph.root_graph_id(),
              "Root project tree must change when resolving to a gcmake_dependency graph."
            );
            will_be_outside_original_project_context = true;
          }
          else {
            return Err(LinkResolutionFailureReason::PointsToInvalidOrNonexistentProject) 
          }
        }
      }

      return next_graph.resolve_namespace_helper(
        target_id_counter,
        namespace_stack,
        target_list,
        link_mode,
        access_mode,
        will_be_outside_original_project_context
      );
    }
  }

  // This function only needs to worry about finding targets in the current project and subproject.
  fn resolve_target_into_link(
    &self,
    target_id_counter: &mut i32,
    target_name: &str,
    link_mode: &LinkMode,
    using_access_mode: &LinkAccessMode
  ) -> Result<Link, LinkResolutionFailureReason> {
    let toplevel_graph: Rc<DependencyGraph> = Weak::upgrade(&self.toplevel).unwrap();

    if let ProjectWrapper::GCMakeDependency(gcmake_dep) = &self.project_wrapper {
      if let GCMakeDependencyStatus::NotDownloaded(_) = gcmake_dep.project_status() {
        // Targets should be created on the fly.
        let mut target_map = self.targets.borrow_mut();
        let new_placeholder_target: Rc<RefCell<TargetNode>> = Rc::new(RefCell::new(TargetNode::new(
          target_id_counter,
          target_name,
          Weak::clone(&self.current_graph_ref),
          ContainedItem::PredefinedLibrary(target_name.to_string()),
          LinkAccessMode::UserFacing,
          true
        )));

        let the_link = Ok(Link::new(
          target_name.to_string(),
          Rc::downgrade(&new_placeholder_target),
          link_mode.clone()
        ));

        target_map.insert(target_name.to_string(), new_placeholder_target);
        return the_link;
      }
    }

    let maybe_resolved_link: Option<Link> = self.resolve_target_into_link_helper(
      target_id_counter,
      target_name,
      link_mode,
      using_access_mode
    )?;

    return match maybe_resolved_link {
      Some(resolved_link) => Ok(resolved_link),
      None => Err(LinkResolutionFailureReason::NotFound {
        project: self.project_wrapper.clone(),
        target_name: target_name.to_string()
      })
    }
  }

  fn resolve_target_into_link_helper(
    &self,
    target_id_counter: &mut i32,
    target_name: &str,
    link_mode: &LinkMode,
    using_access_mode: &LinkAccessMode
  ) -> Result<Option<Link>, LinkResolutionFailureReason> {
    if let Some(found_target) = self.targets.borrow().get(target_name) {
      if using_access_mode.satisfies(&found_target.as_ref().borrow().visibility) {
        return Ok(Some(Link::new(
          target_name.to_string(),
          Rc::downgrade(found_target),
          link_mode.clone()
        )))
      }
      else {
        return Err(LinkResolutionFailureReason::AccessNotAllowed {
          project: self.project_wrapper.clone(),
          given_access_mode: using_access_mode.clone(),
          target_name: target_name.to_string()
        });
      }
    }

    for (_, subproject) in &self.subprojects {
      let maybe_link: Option<Link> = subproject.resolve_target_into_link_helper(
        target_id_counter,
        target_name,
        link_mode,
        using_access_mode
      )?;

      if let Some(resolved_link) = maybe_link {
        return Ok(Some(resolved_link));
      }
    }

    return Ok(None);
  }

  fn recurse_root_project(
    target_id_counter: &mut TargetId,
    graph_id_counter: &mut ProjectId,
    project: &Rc<FinalProjectData>
  ) -> Rc<Self> {
    Self::recurse_project_helper(
      target_id_counter,
      graph_id_counter,
      project,
      None,
      None
    )
  }

  fn recurse_nested_project(
    target_id_counter: &mut i32,
    graph_id_counter: &mut usize,
    project: &Rc<FinalProjectData>,
    parent_graph: &Rc<DependencyGraph>,
    toplevel_graph: Weak<DependencyGraph>
  ) -> Rc<Self> {
    Self::recurse_project_helper(
      target_id_counter,
      graph_id_counter,
      project,
      Some(Rc::downgrade(parent_graph)),
      Some(toplevel_graph)
    )
  }

  fn recurse_project_helper(
    target_id_counter: &mut TargetId,
    graph_id_counter: &mut ProjectId,
    project: &Rc<FinalProjectData>,
    parent_graph: Option<Weak<DependencyGraph>>,
    toplevel_graph: Option<Weak<DependencyGraph>>
  ) -> Rc<Self> {
    match project.get_project_type() {
      FinalProjectType::Root => {
        assert!(
          toplevel_graph.is_none(),
          "A root project (this includes GCMake dependencies) should not be given a toplevel graph, since it it the toplevel graph."
        );
        assert!(
          parent_graph.is_none(),
          "A root project (this includes GCMake dependencies) should not have a parent graph."
        );
      },
      FinalProjectType::Subproject { .. }
        | FinalProjectType::Test { .. } =>
      {
        assert!(
          toplevel_graph.is_some(),
          "Subprojects and test projects must have a toplevel graph, since they are contained inside another project."
        );
        assert!(
          parent_graph.is_some(),
          "Subprojects and test projects must have a parent graph, since they are contained inside another project."
        );
      }
    }

    let mut graph: Rc<DependencyGraph> = Rc::new(Self {
      graph_id: *graph_id_counter,
      project_group_id: match project.get_project_type() {
        // Test projects are considered part of the same group as their parent graph.
        FinalProjectType::Test { .. } => Weak::upgrade(&parent_graph.unwrap()).unwrap().project_group_id.clone(),
        _ => ProjectGroupId(*graph_id_counter)
      },

      parent: parent_graph,
      toplevel: Weak::new(),
      current_graph_ref: Weak::new(),

      project_wrapper: ProjectWrapper::NormalProject(Rc::clone(project)),
      targets: RefCell::new(HashMap::new()),
      pre_build_wrapper: None,

      subprojects: HashMap::new(),
      test_projects: HashMap::new(),
      predefined_deps: HashMap::new(),
      gcmake_deps: HashMap::new()
    });

    *graph_id_counter += 1;

    match toplevel_graph {
      Some(existing_toplevel) => {
        graph.toplevel = existing_toplevel;
      },
      None => {
        graph.toplevel = Rc::downgrade(&graph);
      }
    }

    graph.toplevel = toplevel_graph.unwrap_or(Rc::downgrade(&graph));
    graph.current_graph_ref = Rc::downgrade(&graph);

    graph.pre_build_wrapper = project.get_prebuild_script().as_ref()
      .map(|pre_build_script| {
        Rc::new(RefCell::new(TargetNode::new(
          target_id_counter,
          Self::pre_build_script_name(),
          Rc::downgrade(&graph),
          ContainedItem::PreBuild(pre_build_script),
          LinkAccessMode::UserFacing,
          false
        )))
      });

    let target_map: HashMap<String, Rc<RefCell<TargetNode>>> = project.get_outputs()
      .iter()
      .map(|(target_name, output_item)| {
        let access_mode: LinkAccessMode = if output_item.is_executable_type()
          { LinkAccessMode::Internal }
          else { LinkAccessMode::UserFacing };

        let can_link_to: bool = match project.get_project_type() {
          FinalProjectType::Root => true,
          FinalProjectType::Subproject { .. } => true,
          FinalProjectType::Test { .. } => false
        };

        (
          target_name.to_string(),
          Rc::new(RefCell::new(TargetNode::new(
            target_id_counter,
            target_name,
            Rc::downgrade(&graph),
            ContainedItem::CompiledOutput(output_item),
            access_mode,
            can_link_to
          )))
        )
      })
      .collect();

    graph.targets = RefCell::new(target_map);

    graph.subprojects = project.get_subprojects()
      .iter()
      .map(|(subproject_name, subproject)| {
        (
          subproject_name.to_string(),
          Self::recurse_nested_project(
            target_id_counter,
            graph_id_counter,
            subproject,
            &graph,
            Weak::clone(&graph.toplevel)
          ) 
        )
      })
      .collect();

    graph.test_projects = project.get_test_projects()
      .iter()
      .map(|(test_project_name, test_project)| {
        (
          test_project_name.to_string(),
          Self::recurse_nested_project(
            target_id_counter,
            graph_id_counter,
            test_project,
            &graph,
            Weak::clone(&graph.toplevel)
          ) 
        )
      })
      .collect();

    graph.predefined_deps = project.get_predefined_dependencies()
      .iter()
      .map(|(predep_name, predef_dep)| {
        (
          predep_name.to_string(),
          Self::load_predefined_dependency(
            target_id_counter,
            graph_id_counter,
            predef_dep
          )
        )
      })
      .collect();

    graph.gcmake_deps = project.get_gcmake_dependencies()
      .iter()
      .map(|(gcmake_dep_name, gcmake_dep)| {
        (
          gcmake_dep_name.to_string(),
          Self::load_gcmake_dependency(
            target_id_counter,
            graph_id_counter,
            gcmake_dep
          )
        )
      })
      .collect();

    return graph;
  }

  fn load_predefined_dependency(
    target_id_counter: &mut i32,
    graph_id_counter: &mut usize,
    predef_dep: &Rc<FinalPredefinedDependencyConfig>
  ) -> Rc<DependencyGraph> {
    let mut graph: Rc<DependencyGraph> = Rc::new(DependencyGraph {
      graph_id: *graph_id_counter,
      project_group_id: ProjectGroupId(*graph_id_counter),
      toplevel: Weak::new(),
      parent: None,
      current_graph_ref: Weak::new(),
      project_wrapper: ProjectWrapper::PredefinedDependency(Rc::clone(predef_dep)),
      pre_build_wrapper: None,
      gcmake_deps: HashMap::new(),
      predefined_deps: HashMap::new(),
      subprojects: HashMap::new(),
      test_projects: HashMap::new(),
      targets: RefCell::new(HashMap::new())
    });

    *graph_id_counter += 1;

    graph.toplevel = Rc::downgrade(&graph);
    graph.current_graph_ref = Rc::downgrade(&graph);

    let targets: HashMap<String, Rc<RefCell<TargetNode>>> = predef_dep.target_name_set()
      .into_iter()
      .map(|target_name| {
        (
          target_name.clone(),
          Rc::new(RefCell::new(TargetNode::new(
            &mut target_id_counter,
            &target_name,
            Rc::downgrade(&graph),
            ContainedItem::PredefinedLibrary(target_name),
            LinkAccessMode::UserFacing,
            true
          )))
        )
      })
      .collect();

    for (target_name, target_rc) in &targets {
      let single_target: &mut TargetNode = &mut target_rc.as_ref().borrow_mut();

      let requirement_target_names = &predef_dep.get_target_config_map()
        .get(target_name)
        .unwrap()
        .requires;

      for requirement_name in requirement_target_names {
        assert!(
          targets.contains_key(requirement_name),
          "Required interdependent target names should always have a match in a predefined dependency project, since those are checked when the project itself is loaded."
        );

        single_target.insert_link(Link::new(
          requirement_name.to_string(),
          Rc::downgrade(targets.get(requirement_name).unwrap()),
          // Not sure if this will make a difference yet, since we probably won't be checking links
          // to predefined dependencies anyways.
          LinkMode::Public
        ));
      }
    }

    graph.targets = RefCell::new(targets);

    return graph;
  }

  fn load_gcmake_dependency(
    target_id_counter: &mut i32,
    graph_id_counter: &mut usize,
    gcmake_dep: &Rc<FinalGCMakeDependency>
  ) -> Rc<DependencyGraph> {
    return match gcmake_dep.project_status() {
      GCMakeDependencyStatus::Available(available_project) => {
        let mut resolved_project = Self::recurse_root_project(
          target_id_counter,
          graph_id_counter,
          available_project
        );

        resolved_project.project_wrapper = ProjectWrapper::GCMakeDependency(Rc::clone(gcmake_dep));
        resolved_project
      },
      GCMakeDependencyStatus::NotDownloaded(project_name) => {
        let mut graph: Rc<DependencyGraph> = Rc::new(DependencyGraph {
          graph_id: *graph_id_counter,
          project_group_id: ProjectGroupId(*graph_id_counter),
          toplevel: Weak::new(),
          parent: None,
          current_graph_ref: Weak::new(),
          project_wrapper: ProjectWrapper::GCMakeDependency(Rc::clone(gcmake_dep)),
          pre_build_wrapper: None,
          gcmake_deps: HashMap::new(),
          predefined_deps: HashMap::new(),
          subprojects: HashMap::new(),
          test_projects: HashMap::new(),
          // Targets are added on the fly during the link assignment step.
          // Links to an unavailable gcmake dependency project may be incorrect,
          // however we have no way of knowing that since the project isn't available
          // yet. This is the way we "disable checks" until the repo is cloned.
          // TODO: In the future, allow checks to be done using a 'predefined gcmake dependency'
          // document in the project-local .gcmake/ dir. Essentially, this would be a small yaml
          // file which describes the dependency information and lists its targets, just like
          // the regular predefined dependency files.
          targets: RefCell::new(HashMap::new())
        });

        *graph_id_counter += 1;

        graph.toplevel = Rc::downgrade(&graph);
        graph.current_graph_ref = Rc::downgrade(&graph);
        graph
      }
    }
  }
}

struct DAGSubGraph {
  // Head nodes are not depended on by any other nodes. At least one of these is guaranteed
  // to exist in a graph which has no cycles.
  pub head_nodes: HashSet<RcRefcHashWrapper<TargetNode>>,
  pub all_member_nodes: HashSet<RcRefcHashWrapper<TargetNode>>
}

struct OrderedTargetInfo {
  pub ordered_targets: Vec<RcRefcHashWrapper<TargetNode>>,
  pub project_order: Vec<Rc<DependencyGraph>>
}

impl OrderedTargetInfo {
  // Assumes the vec of targets is already correctly sorted.
  pub fn from_ordered(ordered_targets: Vec<RcRefcHashWrapper<TargetNode>>) -> Self {
    let mut project_indices: HashMap<Rc<DependencyGraph>, usize> = HashMap::new();

    for (target_index, target) in ordered_targets.iter().enumerate() {
      project_indices.entry(target.borrow().container_project())
        .and_modify(|index| *index = target_index)
        .or_insert(target_index);
    }

    let mut project_to_index_pairs: Vec<(Rc<DependencyGraph>, usize)> = project_indices
      .into_iter()
      .collect();
    
    project_to_index_pairs
      .sort_by_key(|(_, last_target_index)| last_target_index);

    return Self {
      ordered_targets,
      project_order: project_to_index_pairs
        .into_iter()
        .map(|(project, _)| project)
        .collect()
    }
  }
}

type DepMap = HashMap<
  RcRefcHashWrapper<TargetNode>,
  HashSet<RcRefcHashWrapper<TargetNode>>
>;

type InverseDepMap = HashMap<
  RcRefcHashWrapper<TargetNode>,
  HashSet<RcRefcHashWrapper<TargetNode>>
>;

type GroupIdMappedTargets = HashMap<ProjectGroupId, HashSet<RcRefcHashWrapper<TargetNode>>>;

fn make_dep_map(
  all_used_targets: &HashSet<RcRefcHashWrapper<TargetNode>>
) -> DepMap {
  let mut dep_map: DepMap = DepMap::new();

  for target_node in all_used_targets {
    for (_, dependency_link) in &target_node.borrow().depends_on {
      let dependency_target: Rc<RefCell<TargetNode>> = Weak::upgrade(&dependency_link.target).unwrap();

      dep_map.entry(target_node.clone())
        .and_modify(move |dependency_set| {
          dependency_set.insert(RcRefcHashWrapper(dependency_target));
        })
        .or_insert(HashSet::from([RcRefcHashWrapper(dependency_target)]));
    }
  }

  return dep_map;
}

fn make_inverse_dep_map(
  all_used_targets: &HashSet<RcRefcHashWrapper<TargetNode>>
) -> InverseDepMap {
  // target -> nodes which depend on target
  let mut inverse_map: InverseDepMap = InverseDepMap::new();

  for dependent_target in all_used_targets {
    for (_, dependency_link) in &dependent_target.borrow().depends_on {
      let target: Rc<RefCell<TargetNode>> = Weak::upgrade(&dependency_link.target).unwrap();

      inverse_map.entry(dependent_target.clone())
        .and_modify(move |dependent_set| {
          dependent_set.insert(RcRefcHashWrapper(target));
        })
        .or_insert(HashSet::from([RcRefcHashWrapper(target)]));
    }
  }

  return inverse_map;
}

fn nodes_mapped_by_project_group_id(
  all_used_targets: &HashSet<RcRefcHashWrapper<TargetNode>>
) -> GroupIdMappedTargets {
  let mut the_map: GroupIdMappedTargets = GroupIdMappedTargets::new();

  for node in all_used_targets {
    let node_ref_clone = node.clone();

    the_map.entry(node.borrow().container_project_group_id())
      .and_modify(|node_set| {
        node_set.insert(node_ref_clone);
      })
      .or_insert(HashSet::from([node_ref_clone]));
  }

  return the_map;
}

fn sorted_target_info(all_used_targets: &HashSet<RcRefcHashWrapper<TargetNode>>) -> OrderedTargetInfo {
  let dep_map: DepMap = make_dep_map(&all_used_targets);
  let inverse_dep_map: InverseDepMap = make_inverse_dep_map(&all_used_targets);
  let nodes_by_project: GroupIdMappedTargets = nodes_mapped_by_project_group_id(&all_used_targets);

  let mut sorted_node_list: Vec<RcRefcHashWrapper<TargetNode>> = Vec::new();
  let mut visited: HashSet<RcRefcHashWrapper<TargetNode>> = HashSet::new();

  for dag_subgraph in find_all_dag_subgraphs(&all_used_targets, &inverse_dep_map) {
    for head_node in dag_subgraph.head_nodes {
      recurse_sort_select(
        &head_node,
        &mut sorted_node_list,
        &mut visited,
        &dep_map,
        &inverse_dep_map,
        &nodes_by_project
      );
    }
  }

  return OrderedTargetInfo::from_ordered(sorted_node_list);
}

fn recurse_sort_select(
  node_checking: &RcRefcHashWrapper<TargetNode>,
  sorted_node_list: &mut Vec<RcRefcHashWrapper<TargetNode>>,
  visited: &mut HashSet<RcRefcHashWrapper<TargetNode>>,
  dep_map: &DepMap,
  inverse_dep_map: &InverseDepMap,
  nodes_by_project: &GroupIdMappedTargets
) {
  let project_head_nodes_iter = nodes_by_project.get(&node_checking.borrow().container_project_group_id())
    .unwrap()
    .iter()
    .filter(|node| {
      let is_depended_on_by_other_node_in_project_group: bool = inverse_dep_map.get(node)
        .unwrap()
        .iter()
        .any(|dependent|
          dependent.borrow().container_project_group_id() == node.borrow().container_project_group_id()
        );

      !is_depended_on_by_other_node_in_project_group && !visited.contains(node)
    });

  for node in project_head_nodes_iter {
    traverse_sort_nodes(
      node,
      sorted_node_list,
      visited,
      dep_map,
      inverse_dep_map,
      nodes_by_project
    );
  }
}

fn traverse_sort_nodes(
  // 'node' is guaranteed to be an uppermost unvisited node in the project.
  node: &RcRefcHashWrapper<TargetNode>,
  sorted_node_list: &mut Vec<RcRefcHashWrapper<TargetNode>>,
  visited: &mut HashSet<RcRefcHashWrapper<TargetNode>>,
  dep_map: &DepMap,
  inverse_dep_map: &InverseDepMap,
  nodes_by_project: &GroupIdMappedTargets
) {
  visited.insert(node.clone());

  for dependency_node in dep_map.get(node).unwrap() {
    if visited.contains(dependency_node) {
      continue;
    }

    if dependency_node.borrow().container_project_group_id() != node.borrow().container_project_group_id() {
      recurse_sort_select(
        dependency_node,
        sorted_node_list,
        visited,
        dep_map,
        inverse_dep_map,
        nodes_by_project
      );
    }
    else {
      traverse_sort_nodes(
        node,
        sorted_node_list,
        visited,
        dep_map,
        inverse_dep_map,
        nodes_by_project
      );
    }
  }

  sorted_node_list.push(node.clone());
}

fn find_all_dag_subgraphs(
  all_used_targets: &HashSet<RcRefcHashWrapper<TargetNode>>,
  inverse_dep_map: &InverseDepMap
) -> Vec<DAGSubGraph> {
  let mut all_visited: HashSet<RcRefcHashWrapper<TargetNode>> = HashSet::new();
  let mut dag_list: Vec<DAGSubGraph> = Vec::new();

  for node in all_used_targets {
    if !all_visited.contains(node) {
      let local_visited: HashSet<RcRefcHashWrapper<TargetNode>> = HashSet::new();
      let stack: Vec<RcRefcHashWrapper<TargetNode>> = vec![node.clone()];

      while let Some(node_checking) = stack.pop() {
        local_visited.insert(node_checking.clone());

        for (_, dep_link) in &node_checking.borrow().depends_on {
          let wrapped_link_target: RcRefcHashWrapper<TargetNode> = 
            RcRefcHashWrapper(Weak::upgrade(&dep_link.target).unwrap());

          if !local_visited.contains(&wrapped_link_target) {
            stack.push(wrapped_link_target);
          }
        }

        for dependent in inverse_dep_map.get(&node_checking).unwrap() {
          if !local_visited.contains(dependent) {
            stack.push(dependent.clone())
          }
        }
      }

      for visited_node in &local_visited {
        all_visited.insert(visited_node.clone());
      }

      dag_list.push(DAGSubGraph {
        head_nodes: local_visited.iter()
          .filter(|member_node|
            inverse_dep_map.get(member_node).unwrap().is_empty()
          )
          .map(|head_node| head_node.clone())
          .collect(),
        all_member_nodes: local_visited
      });
    }
  }

  assert!(
    all_visited.len() == all_used_targets.len(),
    "Number of visited nodes should equal the number of used nodes."
  );

  return dag_list;
}