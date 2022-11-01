use std::{cell::{RefCell}, rc::{Rc, Weak}, hash::{Hash, Hasher}, collections::{HashMap, HashSet, VecDeque}, borrow::Borrow, path::{Path, PathBuf}};

use crate::{project_info::{LinkMode, CompiledOutputItem, PreBuildScript, OutputItemLinks, final_project_data::FinalProjectData, final_dependencies::{FinalGCMakeDependency, FinalPredefinedDependencyConfig, GCMakeDependencyStatus, FinalRequirementSpecifier, FinalTargetConfig, FinalExternalRequirementSpecifier}, LinkSpecifier, FinalProjectType, parsers::{link_spec_parser::{LinkAccessMode, LinkSpecTargetList, LinkSpecifierTarget}, system_spec::platform_spec_parser::SystemSpecifierWrapper}, raw_data_in::{dependencies::internal_dep_config::raw_dep_common::RawEmscriptenConfig, OutputItemType}, FinalFeatureEnabler}, project_generator::configuration};

use super::hash_wrapper::RcRefcHashWrapper;
use colored::*;

pub struct BasicTargetSearchResult<'a> {
  pub searched_with: String,
  pub target_name_searched: String,
  pub searched_project: Rc<RefCell<DependencyGraph<'a>>>,
  pub target: Option<Rc<RefCell<TargetNode<'a>>>>
}

pub struct BasicProjectSearchResult<'a> {
  pub searched_with: String,
  pub project_name_searched: String,
  pub searched_project: Rc<RefCell<DependencyGraph<'a>>>,
  pub project: Option<Rc<RefCell<DependencyGraph<'a>>>>
}

#[derive(Clone)]
pub enum SimpleNodeOutputType {
  Executable,
  Library
}

type TargetId = i32;
type ProjectId = usize;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ProjectGroupId(usize);

#[derive(Clone)]
pub struct CycleNode<'a> {
  pub target: Rc<RefCell<TargetNode<'a>>>,
  pub project: Rc<RefCell<DependencyGraph<'a>>>
}

impl<'a> CycleNode<'a> {
  pub fn new(target: Rc<RefCell<TargetNode<'a>>>) -> Self {
    return Self {
      project: {
        let the_borrow = target.as_ref().borrow();
        the_borrow.container_project()
      },
      target
    };
  }
}

pub enum AdditionalConfigValidationFailureReason<'a> {
  EmscriptenHTMLPathPointsToNonexistent {
    target: Rc<RefCell<TargetNode<'a>>>,
    absolute_path_to_html_file: PathBuf,
    given_relative_path: PathBuf
  },
  WindowsIconPathPointsToNonexistent {
    target: Rc<RefCell<TargetNode<'a>>>,
    absolute_path_to_icon: PathBuf,
    given_relative_path: PathBuf
  },
  FeatureEnablerDependencyProjectNotFound {
    container_feature_name: String,
    gcmake_dep_name: String,
    feature_name_to_enable: String
  },
  FeatureEnablerDependencyFeatureNotFound {
    container_feature_name: String,
    gcmake_dep_name: String,
    feature_name_to_enable: String
  }
}

pub enum GraphLoadFailureReason<'a> {
  FailedAdditionalProjectValidation {
    project: Rc<RefCell<DependencyGraph<'a>>>,
    failure_reason: AdditionalConfigValidationFailureReason<'a>
  },
  SubprojectNameOverlapsDependency {
    subproject: Rc<RefCell<DependencyGraph<'a>>>,
    dependency_project: Rc<RefCell<DependencyGraph<'a>>>
  },
  DuplicateRootProjectIdentifier {
    project1: Rc<RefCell<DependencyGraph<'a>>>,
    project2: Rc<RefCell<DependencyGraph<'a>>>
  },
  DuplicateCMakeIdentifier {
    target1: Rc<RefCell<TargetNode<'a>>>,
    target1_project: Rc<RefCell<DependencyGraph<'a>>>,
    target2: Rc<RefCell<TargetNode<'a>>>,
    target2_project: Rc<RefCell<DependencyGraph<'a>>>
  },
  DuplicateYamlIdentifier {
    target1: Rc<RefCell<TargetNode<'a>>>,
    target1_project: Rc<RefCell<DependencyGraph<'a>>>,
    target2: Rc<RefCell<TargetNode<'a>>>,
    target2_project: Rc<RefCell<DependencyGraph<'a>>>
  },
  DuplicateLinkTarget {
    link_spec_container_target: Rc<RefCell<TargetNode<'a>>>,
    link_spec_container_project: Rc<RefCell<DependencyGraph<'a>>>,
    dependency_project: Rc<RefCell<DependencyGraph<'a>>>,
    dependency: Rc<RefCell<TargetNode<'a>>>,
  },
  LinkSystemRequirementImpossible {
    target: Rc<RefCell<TargetNode<'a>>>,
    target_container_project: Rc<RefCell<DependencyGraph<'a>>>,
    link_system_spec_info: SystemSpecifierWrapper,
    dependency: Rc<RefCell<TargetNode<'a>>>
  },
  LinkSystemSubsetMismatch {
    link_spec: Option<LinkSpecifier>,
    link_system_spec_info: SystemSpecifierWrapper,
    link_spec_container_target: Rc<RefCell<TargetNode<'a>>>,
    link_spec_container_project: Rc<RefCell<DependencyGraph<'a>>>,
    dependency_project: Rc<RefCell<DependencyGraph<'a>>>,
    dependency: Rc<RefCell<TargetNode<'a>>>,
    transitively_required_by: Option<Rc<RefCell<TargetNode<'a>>>>
  },
  LinkPointsToInvalidOrNonexistentProject {
    target: Rc<RefCell<TargetNode<'a>>>,
    project: Rc<RefCell<DependencyGraph<'a>>>,
    link_spec: LinkSpecifier
  },
  LinkNestedNamespaceInOtherProjectContext {
    target: Rc<RefCell<TargetNode<'a>>>,
    project: Rc<RefCell<DependencyGraph<'a>>>,
    link_spec: LinkSpecifier
  },
  DependencyCycle(Vec<CycleNode<'a>>),
  WrongUserGivenPredefLinkMode {
    current_link_mode: LinkMode,
    needed_link_mode: LinkMode,
    target: Rc<RefCell<TargetNode<'a>>>,
    target_project: Rc<RefCell<DependencyGraph<'a>>>,
    dependency: Rc<RefCell<TargetNode<'a>>>,
    dependency_project: Rc<RefCell<DependencyGraph<'a>>>
  },
  LinkedInMultipleCategories {
    current_link_mode: LinkMode,
    attempted_link_mode: LinkMode,
    link_receiver_project: Rc<RefCell<DependencyGraph<'a>>>,
    link_receiver_name: String,
    link_giver_project: Rc<RefCell<DependencyGraph<'a>>>,
    link_giver_name: String,
  },
  ComplexTargetRequirementNotSatisfied {
    target: Rc<RefCell<TargetNode<'a>>>,
    target_project: Rc<RefCell<DependencyGraph<'a>>>,
    dependency: Rc<RefCell<TargetNode<'a>>>,
    dependency_project: Rc<RefCell<DependencyGraph<'a>>>,
    failed_requirement: OwningComplexTargetRequirement<'a>
  },
  LinkedToSelf {
    project: Rc<RefCell<DependencyGraph<'a>>>,
    target_name: String
  },
  AccessNotAllowed {
    link_spec: LinkSpecifier,
    link_spec_container_target: Rc<RefCell<TargetNode<'a>>>,
    link_spec_container_project: Rc<RefCell<DependencyGraph<'a>>>,
    dependency_project: Rc<RefCell<DependencyGraph<'a>>>,
    dependency: Rc<RefCell<TargetNode<'a>>>,
    given_access_mode: LinkAccessMode,
    needed_access_mode: LinkAccessMode
  },
  LinkTargetNotFound {
    target: Rc<RefCell<TargetNode<'a>>>,
    target_container_project: Rc<RefCell<DependencyGraph<'a>>>,
    looking_in_project: Rc<RefCell<DependencyGraph<'a>>>,
    link_spec: LinkSpecifier,
    name_searching: String
  }
}

pub struct Link<'a> {
  target_name: String,
  system_spec_info: SystemSpecifierWrapper,
  link_mode: LinkMode,
  target: Weak<RefCell<TargetNode<'a>>>
}

impl<'a> Link<'a> {
  pub fn new(
    target_name: String,
    system_spec_info: SystemSpecifierWrapper,
    target: Weak<RefCell<TargetNode<'a>>>,
    link_mode: LinkMode
  ) -> Self {

    unsafe {
      (*Weak::upgrade(&target).unwrap().as_ptr()).linked_to_count += 1;
    }

    Self {
      target_name,
      system_spec_info,
      target,
      link_mode
    }
  }

  fn target_id(&self) -> TargetId {
    Weak::upgrade(&self.target).unwrap().as_ref().borrow().unique_target_id()
  }

  pub fn linked_target(&self) -> Rc<RefCell<TargetNode<'a>>> {
    Weak::upgrade(&self.target).unwrap()
  }

  pub fn get_link_mode(&self) -> LinkMode {
    self.link_mode.clone()
  }

  pub fn get_system_spec_info(&self) -> &SystemSpecifierWrapper {
    &self.system_spec_info
  }
}

impl<'a> Hash for Link<'a> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.target_id().hash(state);
  }
}

impl<'a> PartialEq for Link<'a> {
  fn eq(&self, other: &Self) -> bool {
    return self.target_id() == other.target_id();
  }
}

impl<'a> Eq for Link<'a> { }

#[derive(Clone)]
pub enum ContainedItem<'a> {
  CompiledOutput(&'a CompiledOutputItem),
  PredefinedLibrary {
    target_name: String,
    external_requirements: Vec<FinalExternalRequirementSpecifier>
  },
  PreBuild(&'a PreBuildScript)
}

impl<'a> ContainedItem<'a> {
  pub fn is_predefined_lib(&self) -> bool {
    match self {
      Self::PredefinedLibrary { .. } => true,
      _ => false
    }
  }
}

// Target which might not have been imported into the project, but still needs to be referenced
// in errors. For example, external_requirements of a predefined dependency's target might not
// even be populated into the project at all even if the target is used, but we still need to know
// which external_requirements the target would need to have. We can't just delete the requirement
// even though the library isn't present.
pub enum MaybePresentNonOwningTarget<'a> {
  NotImported {
    namespaced_yaml_name: String
  },
  Populated(Weak<RefCell<TargetNode<'a>>>)
}

// Weak references to existing nodes.
enum NonOwningComplexTargetRequirement<'a> {
  OneOf(Vec<MaybePresentNonOwningTarget<'a>>),
  ExclusiveFrom(Weak<RefCell<TargetNode<'a>>>)
}

pub enum MaybePresentOwningTarget<'a> {
  NotImported {
    namespaced_yaml_name: String
  },
  Populated(Rc<RefCell<TargetNode<'a>>>)
}

// Strong references to existing nodes.
pub enum OwningComplexTargetRequirement<'a> {
  OneOf(Vec<MaybePresentOwningTarget<'a>>),
  ExclusiveFrom(Rc<RefCell<TargetNode<'a>>>)
}

impl<'a> OwningComplexTargetRequirement<'a> {
  fn make_owned_target_list(weak_target_list: &Vec<MaybePresentNonOwningTarget<'a>>) -> Vec<MaybePresentOwningTarget<'a>> {
    return weak_target_list.iter()
      .map(|non_owning_target| match non_owning_target {
        MaybePresentNonOwningTarget::NotImported { namespaced_yaml_name } => {
          MaybePresentOwningTarget::NotImported { namespaced_yaml_name: namespaced_yaml_name.clone() }
        },
        MaybePresentNonOwningTarget::Populated(weak_target) => {
          MaybePresentOwningTarget::Populated(
            Weak::upgrade(weak_target).unwrap()
          )
        }
      })
      .collect();
  }

  fn new_from(weak_requirement: &NonOwningComplexTargetRequirement<'a>) -> Self {
    match weak_requirement {
      NonOwningComplexTargetRequirement::OneOf(weak_target_list) => {
        Self::OneOf(Self::make_owned_target_list(weak_target_list))
      },
      NonOwningComplexTargetRequirement::ExclusiveFrom(target) => {
        Self::ExclusiveFrom(Weak::upgrade(target).unwrap())
      }
    }
  }
}

#[derive(Clone, PartialEq, Eq)]
pub struct EmscriptenLinkFlagInfo {
  pub full_flag_expression: String,
  pub supports_link_time_only: bool
}

enum NodeType {
  PreBuild,
  ProjectOutput,
  PredefinedLib
}

pub struct TargetNode<'a> {
  node_type: NodeType,
  locator_name: String,

  // Name of the output target item
  output_target_name: String,

  // Name of the target which will receive links
  internal_receiver_name: String,

  // Namespaced linkable name for the target.
  // NOTE: For predefined components modules, this could be a CMake variable (ex: ${wxWidgets_LIBRARIES}).
  // When writing CMakeLists, make sure to eliminate duplicates before writing, so this variable isn't linked
  // more than once.
  namespaced_cmake_target_name: String,
  namespaced_yaml_target_name: String,

  system_specifier_info: SystemSpecifierWrapper,

  requires_custom_install_if_linked_to_output_lib: bool,
  is_linked_to_output_lib: bool,

  the_unique_id: TargetId,
  linked_to_count: i32,
  contained_in_graph: Weak<RefCell<DependencyGraph<'a>>>,
  output_type: SimpleNodeOutputType,
  visibility: LinkAccessMode,
  // depends_on: HashSet<Link>,
  depends_on: HashMap<TargetId, Link<'a>>,
  complex_requirements: Vec<NonOwningComplexTargetRequirement<'a>>,
  // TODO: This doesn't need to be a copy. This is just easier to use, for now.
  raw_link_specifiers: Option<OutputItemLinks>,
  contained_item: ContainedItem<'a>
}

impl<'a> TargetNode<'a> {
  fn new(
    id_var: &mut TargetId,
    locator_name: impl AsRef<str>,
    system_specifier_info: SystemSpecifierWrapper,
    output_target_name: String,
    internal_receiver_name: String,
    namespaced_output_target_name: String,
    namespaced_yaml_target_name: String,
    should_install_if_linked_to_output_library: bool,
    parent_graph: Weak<RefCell<DependencyGraph<'a>>>,
    contained_item: ContainedItem<'a>,
    visibility: LinkAccessMode,
    _can_link_to: bool
  ) -> Self {
    let unique_id: TargetId = *id_var;
    *id_var = unique_id + 1;

    let output_type: SimpleNodeOutputType;
    let raw_link_specifiers: Option<OutputItemLinks>;
    let node_type: NodeType;

    match contained_item {
      ContainedItem::PredefinedLibrary { .. } => {
        raw_link_specifiers = None;
        output_type = SimpleNodeOutputType::Library;
        node_type = NodeType::PredefinedLib;
      },
      ContainedItem::CompiledOutput(output_item) => {
        raw_link_specifiers = Some(output_item.get_links().clone());
        output_type = if output_item.is_library_type()
          { SimpleNodeOutputType::Library }
          else { SimpleNodeOutputType::Executable };
        node_type = NodeType::ProjectOutput;
      },
      ContainedItem::PreBuild(pre_build) => match pre_build {
        PreBuildScript::Exe(pre_build_exe) => {
          raw_link_specifiers = Some(pre_build_exe.get_links().clone());
          output_type = SimpleNodeOutputType::Executable;
          node_type = NodeType::PreBuild;
        },
        PreBuildScript::Python(_) => {
          raw_link_specifiers = None;
          // This is just a placeholder. Not sure if this will cause issues yet, but it shouldn't.
          output_type = SimpleNodeOutputType::Executable;
          node_type = NodeType::PreBuild;
        }
      }
    }
    
    return Self {
      contained_item,
      the_unique_id: unique_id,
      node_type,
      locator_name: locator_name.as_ref().to_string(),
      system_specifier_info,

      output_target_name,
      internal_receiver_name,
      namespaced_cmake_target_name: namespaced_output_target_name,
      namespaced_yaml_target_name,

      requires_custom_install_if_linked_to_output_lib: should_install_if_linked_to_output_library,
      is_linked_to_output_lib: false,
      
      contained_in_graph: parent_graph,
      output_type,
      visibility,
      // depends_on: HashSet::new(),
      depends_on: HashMap::new(),
      complex_requirements: Vec::new(),
      raw_link_specifiers,
      linked_to_count: 0
    }
  }

  pub fn get_depends_on(&self) -> &HashMap<TargetId, Link<'a>> {
    &self.depends_on
  }

  pub fn simple_output_type(&self) -> SimpleNodeOutputType {
    return self.output_type.clone();
  }

  pub fn must_be_additionally_installed(&self) -> bool {
    return self.requires_custom_install_if_linked_to_output_lib && self.is_linked_to_output_lib;
  }

  // When targets are public/interface linked by libraries produced by the current project, 
  pub fn should_be_searched_in_package_config(&self) -> bool {
    return self.is_linked_to_output_lib;
  }

  pub fn is_pre_build(&self) -> bool {
    match &self.node_type {
      NodeType::PreBuild => true,
      _ => false
    }
  }

  pub fn is_regular_node(&self) -> bool {
    match &self.node_type {
      NodeType::ProjectOutput => true,
      NodeType::PredefinedLib => true,
      NodeType::PreBuild => false
    }
  }

  fn is_predefined_lib(&self) -> bool {
    match &self.node_type {
      NodeType::PredefinedLib => true,
      _ => false
    }
  }

  pub fn get_name(&self) -> &str {
    &self.locator_name
  }

  pub fn unique_target_id(&self) -> TargetId {
    self.the_unique_id
  }

  pub fn get_cmake_target_base_name(&self) -> &str {
    &self.output_target_name
  }

  pub fn get_internal_receiver_name(&self) -> &str {
    &self.internal_receiver_name
  }

  pub fn get_yaml_namespaced_target_name(&self) -> &str {
    &self.namespaced_yaml_target_name
  }

  pub fn get_cmake_namespaced_target_name(&self) -> &str {
    &self.namespaced_cmake_target_name
  }

  pub fn get_system_spec_info(&self) -> &SystemSpecifierWrapper {
    &self.system_specifier_info
  }

  pub fn emscripten_link_flag(&self) -> Option<EmscriptenLinkFlagInfo> {
    return match self.container_project().as_ref().borrow().project_wrapper().emscripten_config() {
      None => None,
      Some(config) => match &config.link_flag {
        None => None,
        Some(link_flag_str) => Some(EmscriptenLinkFlagInfo {
          full_flag_expression: link_flag_str.to_string(),
          supports_link_time_only: config.is_flag_link_time_only.unwrap_or(false)
        })
      }
    }
  }

  pub fn is_internally_supported_by_emscripten(&self) -> bool {
    return match self.container_project().as_ref().borrow().project_wrapper() {
      ProjectWrapper::NormalProject(_) => false,
      ProjectWrapper::GCMakeDependencyRoot(_) => false,
      ProjectWrapper::PredefinedDependency(predef_dep) =>
        predef_dep.predefined_dep_info().is_internally_supported_by_emscripten()
    }
  }

  pub fn has_links(&self) -> bool {
    let num_regular_links: usize = self.depends_on.iter()
      .filter(|(_, link)| Weak::upgrade(&link.target).unwrap().as_ref().borrow().is_regular_node())
      .collect::<Vec<_>>()
      .len();

    return num_regular_links > 0;
  }

  pub fn get_link_by_id(&self, target_id: TargetId) -> Option<&Link<'a>> {
    return self.depends_on.get(&target_id)
  }

  pub fn maybe_regular_output(&self) -> Option<&CompiledOutputItem> {
    return match &self.contained_item {
      ContainedItem::CompiledOutput(output) => Some(output),
      _ => None
    }
  }

  pub fn get_contained_item(&self) -> &ContainedItem {
    &self.contained_item
  }

  pub fn container_project_id(&self) -> ProjectId {
    self.container_project().as_ref().borrow().graph_id
  }

  pub fn container_project_group_id(&self) -> ProjectGroupId {
    self.container_project().as_ref().borrow().project_group_id.clone()
  }

  pub fn container_project(&self) -> Rc<RefCell<DependencyGraph<'a>>> {
    return Weak::upgrade(&self.contained_in_graph).unwrap();
  }

  fn insert_link(&mut self, link: Link<'a>) {
    // If this node is a regular node and is PUBLIC or INTERFACE linking to its dependency,
    // mark the dependency node as public/private linked.
    if let NodeType::ProjectOutput = &self.node_type {
      if let SimpleNodeOutputType::Library = &self.output_type {
        // I think this is already recursively for predefined dependency 'requirement dependencies' since
        // an error is thrown if interdependent 'requirements' are not linked with the same access modifier.
        let target = Weak::upgrade(&link.target).unwrap();

        if target.as_ref().borrow().is_predefined_lib() {
          unsafe {
            (*target.as_ptr()).is_linked_to_output_lib = true;
          }
        }
      }
    }

    self.depends_on.insert(
      link.target_id(),
      link
    );
  }

  fn add_complex_requirement(&mut self, requirement: NonOwningComplexTargetRequirement<'a>) {
    self.complex_requirements.push(requirement);
  }
}

impl<'a> Hash for TargetNode<'a> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.the_unique_id.hash(state);
  }
}

impl<'a> PartialEq for TargetNode<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.unique_target_id() == other.unique_target_id()
  }
}

impl<'a> Eq for TargetNode<'a> { }

// NOTE: It's fine to keep this clone implementation, since it only clones the contained RCs. 
#[derive(Clone)]
pub enum ProjectWrapper {
  NormalProject(Rc<FinalProjectData>),
  GCMakeDependencyRoot(Rc<FinalGCMakeDependency>),
  PredefinedDependency(Rc<FinalPredefinedDependencyConfig>)
}

impl ProjectWrapper {
  pub fn internal_project_name(&self) -> &str {
    match self {
      Self::NormalProject(project_info) => project_info.get_project_base_name(),
      Self::GCMakeDependencyRoot(gcmake_dep) => gcmake_dep.project_base_name(),
      Self::PredefinedDependency(predef_dep) => predef_dep.get_name()
    }
  }

  pub fn identifier_name(&self) -> &str {
    match self {
      Self::NormalProject(project_info) => project_info.get_project_base_name(),
      Self::GCMakeDependencyRoot(gcmake_dep) => gcmake_dep.given_dependency_name(),
      Self::PredefinedDependency(predef_dep) => predef_dep.get_name()
    }
  }

  pub fn mangled_name(&self) -> &str {
    match self {
      Self::NormalProject(project_info) => project_info.get_full_namespaced_project_name(),
      Self::GCMakeDependencyRoot(gcmake_dep) => gcmake_dep.given_dependency_name(),
      Self::PredefinedDependency(predef_dep) => predef_dep.get_name()
    }
  }

  pub fn name_for_error_messages(&self) -> &str {
    match self {
      Self::NormalProject(project_info) => project_info.get_name_for_error_messages(),
      Self::GCMakeDependencyRoot(gcmake_dep) => gcmake_dep.given_dependency_name(),
      Self::PredefinedDependency(predef_dep) => predef_dep.get_name()
    }
  }

  pub fn contains_predef_dep(&self) -> bool {
    self.maybe_predef_dep().is_some()
  }

  pub fn maybe_predef_dep(&self) -> Option<&Rc<FinalPredefinedDependencyConfig>> {
    return match self {
      Self::PredefinedDependency(predef_dep) => Some(predef_dep),
      _ => None
    }
  }

  // pub fn contains_available_normal_project(&)
  pub fn unwrap_predef_dep(self) -> Rc<FinalPredefinedDependencyConfig> {
    return match self {
      Self::PredefinedDependency(predef_dep) => predef_dep,
      _ => panic!("Tried to unwrap a ProjectWrapper as a predefined dependency when the wrapper doesn't contain a predefined dependency.")
    }
  }

  pub fn unwrap_normal_project(self) -> Rc<FinalProjectData> {
    return match self.maybe_normal_project() {
      Some(project_info) => Rc::clone(project_info),
      None => panic!("Tried to unwrap a ProjectWrapper as a normal project, but the wrapper did not contain an available FinalProjectData.")
    }
  }

  pub fn maybe_gcmake_dep(&self) -> Option<&Rc<FinalGCMakeDependency>> {
    return match self {
      Self::GCMakeDependencyRoot(gcmake_dep) => Some(gcmake_dep),
      _ => None
    }
  }

  pub fn maybe_normal_project(&self) -> Option<&Rc<FinalProjectData>> {
    return match self {
      Self::NormalProject(project_info) => Some(project_info),
      Self::GCMakeDependencyRoot(gcmake_dep) => match gcmake_dep.project_status() {
        GCMakeDependencyStatus::Available(project_info) => Some(project_info),
        GCMakeDependencyStatus::NotDownloaded(_) => None
      }
      _ => None
    }
  }

  pub fn emscripten_config(&self) -> Option<&RawEmscriptenConfig> {
    return match self {
      Self::PredefinedDependency(predef_dep) => predef_dep.emscripten_config(),
      _ => None
    }
  }
}

enum CycleCheckResult<'a> {
  Cycle(Vec<RcRefcHashWrapper<TargetNode<'a>>>),
  AllUsedTargets(HashSet<RcRefcHashWrapper<TargetNode<'a>>>)
}

pub struct DependencyGraphInfoWrapper<'a> {
  pub root_dep_graph: Rc<RefCell<DependencyGraph<'a>>>,
  pub sorted_info: OrderedTargetInfo<'a>
}

pub enum DependencyGraphWarningMode {
  All,
  Off
}

// TODO: Allow predefined dependencies to influence target ordering. For instance, SFML
// targets must be linked in a certain order to work. The SFML predefined configuration
// should be allowed to specify that its 'window' target depends on 'system', and so on.
// Essentially, the configuration should be able to contain a graph-like representation
// of how the dependency's targets depend on each other. After ensuring the graph is correct,
// targets in that library which depend on other targets will be sorted lower in the list
// than the targets they depend on.
pub struct DependencyGraph<'a> {
  parent: Option<Weak<RefCell<DependencyGraph<'a>>>>,
  toplevel: Weak<RefCell<DependencyGraph<'a>>>,
  current_graph_ref: Weak<RefCell<DependencyGraph<'a>>>,

  graph_id: ProjectId,
  
  // Test projects should have the same group ID as their parent.
  // This is necessary when sorting targets because we sometimes need to traverse
  // upward to find all targets in a project group which don't depend on any other targets made by
  // that project group. This ensures that all targets in a project are iterated through in one pass,
  // which is important. 
  project_group_id: ProjectGroupId,

  _project_wrapper: ProjectWrapper,
  targets: RefCell<HashMap<String, Rc<RefCell<TargetNode<'a>>>>>,
  pre_build_wrapper: Option<Rc<RefCell<TargetNode<'a>>>>,

  subprojects: HashMap<String, Rc<RefCell<DependencyGraph<'a>>>>,
  test_projects: HashMap<String, Rc<RefCell<DependencyGraph<'a>>>>,
  gcmake_deps: HashMap<String, Rc<RefCell<DependencyGraph<'a>>>>,

  predefined_deps: HashMap<String, Rc<RefCell<DependencyGraph<'a>>>>
}

impl<'a> Hash for DependencyGraph<'a> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.graph_id.hash(state);
  }
}

impl<'a> PartialEq for DependencyGraph<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.graph_id == other.graph_id
  }
}

impl<'a> Eq for DependencyGraph<'a> { }

impl<'a> DependencyGraph<'a> {
  pub fn new_info_from_root(
    toplevel_project: &'a Rc<FinalProjectData>,
    warning_mode: DependencyGraphWarningMode
  ) -> Result<DependencyGraphInfoWrapper<'a>, GraphLoadFailureReason<'a>> {
    let mut target_id_counter: TargetId = 0;
    let mut toplevel_tree_id_counter: ProjectId = 0;

    let full_graph: Rc<RefCell<DependencyGraph>> = Self::recurse_root_project(
      &mut target_id_counter,
      &mut toplevel_tree_id_counter,
      toplevel_project
    );

    {
      let borrowed_graph = full_graph.as_ref().borrow();

      borrowed_graph.make_given_link_associations(&mut target_id_counter)?;
      borrowed_graph.make_auto_inner_project_link_associations()?;
      borrowed_graph.ensure_proper_predefined_dep_links()?;
      borrowed_graph.ensure_valid_system_spec_links(&warning_mode)?;
      borrowed_graph.ensure_no_duplicate_identifiers()?;
      borrowed_graph.do_additional_project_checks()?;
    }

    let all_used_targets: HashSet<RcRefcHashWrapper<TargetNode>> = match full_graph.as_ref().borrow().find_cycle() {
      CycleCheckResult::AllUsedTargets(all_used) => all_used,
      CycleCheckResult::Cycle(cycle_vec) => {
        return Err(GraphLoadFailureReason::DependencyCycle(
          cycle_vec
            .into_iter()
            .map(|wrapped_target_node| {
              CycleNode::new(wrapped_target_node.unwrap())
            })
            .collect()
        ))
      }
    };

    return Ok(DependencyGraphInfoWrapper {
      sorted_info: sorted_target_info(&all_used_targets),
      root_dep_graph: full_graph
    });
  }

  pub fn project_mangled_name(&self) -> &str {
    self.project_wrapper().mangled_name()
  }

  pub fn internal_project_name(&self) -> &str {
    self.project_wrapper().internal_project_name()
  }

  pub fn project_identifier_name(&self) -> &str {
    self.project_wrapper().identifier_name()
  }

  pub fn project_debug_name(&self) -> &str {
    self.project_wrapper().name_for_error_messages()
  }

  pub fn project_id(&self) -> ProjectId {
    self.graph_id
  }

  pub fn root_project_id(&self) -> usize {
    return self.root_project().as_ref().borrow().graph_id;
  }

  pub fn root_project(&self) -> Rc<RefCell<DependencyGraph<'a>>> {
    return Weak::upgrade(&self.toplevel).unwrap();
  }

  pub fn get_pre_build_node(&self) -> &Option<Rc<RefCell<TargetNode<'a>>>> {
    &self.pre_build_wrapper
  }

  pub fn get_test_projects(&self) -> &HashMap<String, Rc<RefCell<DependencyGraph<'a>>>> {
    &self.test_projects
  }

  pub fn get_gcmake_dependencies(&self) -> &HashMap<String, Rc<RefCell<DependencyGraph<'a>>>> {
    &self.gcmake_deps
  }

  pub fn get_subprojects(&self) -> &HashMap<String, Rc<RefCell<DependencyGraph<'a>>>> {
    &self.subprojects
  }

  pub fn get_predefined_dependencies(&self) -> &HashMap<String, Rc<RefCell<DependencyGraph<'a>>>> {
    &self.predefined_deps
  }

  pub fn project_wrapper(&self) -> &ProjectWrapper {
    &self._project_wrapper
  }

  // See the python prototype project for details.
  // D:\Personal_Projects\Coding\prototyping\python\dependency-graph-sorting
  fn find_cycle(&self) -> CycleCheckResult<'a> {
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
    visited: &mut HashSet<RcRefcHashWrapper<TargetNode<'a>>>,
    stack: &mut Vec<RcRefcHashWrapper<TargetNode<'a>>>
  ) -> Option<Vec<RcRefcHashWrapper<TargetNode<'a>>>> {
    if let Some(pre_build) = &self.pre_build_wrapper {
      if let Some(cycle_vec) = self.do_find_cycle_helper(pre_build, visited, stack) {
        return Some(cycle_vec);
      }
    }

    for (_, target_node) in self.targets.borrow().iter() {
      let wrapped_target_node: RcRefcHashWrapper<TargetNode> = RcRefcHashWrapper(Rc::clone(target_node));

      if !visited.contains(&wrapped_target_node) {
        if let Some(cycle_vec) = self.do_find_cycle_helper(target_node, visited, stack) {
          return Some(cycle_vec);
        }
      }
    }

    for (_, test_project) in &self.test_projects {
      if let Some(cycle_vec) = test_project.as_ref().borrow().do_find_cycle(visited, stack) {
        return Some(cycle_vec);
      }
    }

    for (_, subproject) in &self.subprojects {
      if let Some(cycle_vec) = subproject.as_ref().borrow().do_find_cycle(visited, stack) {
        return Some(cycle_vec);
      }
    }

    for (_, gcmake_dep) in &self.gcmake_deps {
      if let Some(cycle_vec) = gcmake_dep.as_ref().borrow().do_find_cycle(visited, stack) {
        return Some(cycle_vec);
      }
    }

    return None;
  }

  fn do_find_cycle_helper(
    &self,
    node: &Rc<RefCell<TargetNode<'a>>>,
    visited: &mut HashSet<RcRefcHashWrapper<TargetNode<'a>>>,
    stack: &mut Vec<RcRefcHashWrapper<TargetNode<'a>>>
  ) -> Option<Vec<RcRefcHashWrapper<TargetNode<'a>>>> {
    stack.push(RcRefcHashWrapper(Rc::clone(node)));
    visited.insert(RcRefcHashWrapper(Rc::clone(node)));

    for (_, dep_link) in &node.as_ref().borrow().depends_on {
      let dependency_node: RcRefcHashWrapper<TargetNode> =
        RcRefcHashWrapper(Weak::upgrade(&dep_link.target).unwrap());

      if visited.contains(&dependency_node) && stack.contains(&dependency_node) {
        return Some(stack.clone());
      }
      else if let Some(cycle_vec) = self.do_find_cycle_helper(&dependency_node, visited, stack) {
        return Some(cycle_vec);
      }
    }

    stack.pop();
    return None;
  }

  // TODO: Refactor all these as_ref()s and borrow()s a bit.
  fn check_target_system_spec_links_are_valid(
    target: &Rc<RefCell<TargetNode<'a>>>,
    warning_mode: &DependencyGraphWarningMode
  ) -> Result<(), GraphLoadFailureReason<'a>> {
    for (_, link) in &target.as_ref().borrow().depends_on {
      let dependency: Rc<RefCell<TargetNode>> = Weak::upgrade(&link.target).unwrap();

      // println!(
      //   "{}: {:?}\n\tLinked using: {:?}\n\tto {}: {:?}",
      //   target.as_ref().borrow().get_yaml_namespaced_target_name(),
      //   target.as_ref().borrow().system_specifier_info.explicit_name_list(),
      //   link.system_spec_info.explicit_name_list(),
      //   dependency.as_ref().borrow().get_yaml_namespaced_target_name(),
      //   dependency.as_ref().borrow().system_specifier_info.explicit_name_list()
      // );

      // NOTE: Keep this var. It will be used when system-specifier validation checks have been implemented.
      // let target_systems: SystemSpecifierWrapper = target.as_ref().borrow().system_specifier_info.clone();
      let link_systems: SystemSpecifierWrapper = link.system_spec_info.clone();
      let dependency_systems: SystemSpecifierWrapper = dependency.as_ref().borrow().system_specifier_info.clone();

      if let SystemSpecifierWrapper::Specific(dependency_spec_tree) = dependency_systems {
        let current_platform_spec_str: String = match link_systems {
          SystemSpecifierWrapper::All => String::from(""),
          SystemSpecifierWrapper::Specific(current_platform_spec_tree) =>
            format!("Current: {}", current_platform_spec_tree.to_string())
        };

        if let DependencyGraphWarningMode::All = warning_mode {
          println!(
            "\n{}Platform-specific subset validation at GCMake configuration time has not been implemented yet. However, the correct CMake generator expressions will still be written.",
            "Warning: ".yellow()
          );

          println!(
            "-- target '{}' in project '{}' links to '{}', which has a platform specifier '{}'. Please make sure the link to {} is prefixed with a platform specifier that is a subset of {}. {}",
            target.as_ref().borrow().get_name(),
            target.as_ref().borrow().container_project().as_ref().borrow().project_debug_name(),
            dependency.as_ref().borrow().get_yaml_namespaced_target_name(),
            dependency_spec_tree.to_string(),
            dependency.as_ref().borrow().get_yaml_namespaced_target_name(),
            dependency_spec_tree.to_string(),
            current_platform_spec_str
          );
        }
      }

      // TODO: Re-enable these once I figure out the checking system.
      // if !link_systems.is_subset_of(&dependency_systems) {
      //   return Err(GraphLoadFailureReason::LinkSystemSubsetMismatch {
      //     link_spec: None,
      //     link_spec_container_target: Rc::clone(target),
      //     link_spec_container_project: Rc::clone(target).as_ref().borrow().container_project(),
      //     dependency: Rc::clone(&dependency),
      //     dependency_project: dependency.as_ref().borrow().container_project(),
      //     link_system_spec_info: link.system_spec_info.clone(),
      //     transitively_required_by: None
      //   })
      // }
      // else if !link_systems.is_subset_of(&target_systems) {
      //   return Err(GraphLoadFailureReason::LinkSystemRequirementImpossible {
      //     target: Rc::clone(target),
      //     target_container_project: target.as_ref().borrow().container_project(),
      //     link_system_spec_info: link.system_spec_info.clone(),
      //     dependency
      //   });
      // }
      // else if !(link_systems.intersection(&dependency_systems)).is_subset_of(&target_systems) {
      //   return Err(GraphLoadFailureReason::LinkSystemRequirementImpossible {
      //     target: Rc::clone(target),
      //     target_container_project: target.as_ref().borrow().container_project(),
      //     link_system_spec_info: link.system_spec_info.clone(),
      //     dependency
      //   });
      // }
    }

    Ok(())
  }

  fn do_additional_project_checks(&self) -> Result<(), GraphLoadFailureReason<'a>> {
    if let Some(root_project) = self.root_project().as_ref().borrow().project_wrapper().maybe_normal_project() {
      // Ensure that any transitively enabled dependency features actually exist.
      // TODO: Refactor this mess. Thing is, I can't really see a good way to refactor it.
      if self.root_project_id() == self.project_id() {
        if let Some(normal_project_config) = self.project_wrapper().maybe_normal_project() {
          for (container_feature_name, feature_config) in normal_project_config.get_features() {
            for FinalFeatureEnabler { dep_name, feature_name } in &feature_config.enables {
              if let Some(dep_name_str) = dep_name {
                match self.root_project().as_ref().borrow().gcmake_deps.get(dep_name_str) {
                  None => {
                    return Err(GraphLoadFailureReason::FailedAdditionalProjectValidation {
                      project: Weak::upgrade(&self.current_graph_ref).unwrap(),
                      failure_reason: AdditionalConfigValidationFailureReason::FeatureEnablerDependencyProjectNotFound {
                        container_feature_name: container_feature_name.to_string(),
                        gcmake_dep_name: dep_name_str.to_string(),
                        feature_name_to_enable: feature_name.to_string()
                      }
                    });
                  }
                  Some(gcmake_dep) => {
                    if let Some(available_gcmake_dep) = gcmake_dep.as_ref().borrow().project_wrapper().maybe_normal_project() {
                      if !available_gcmake_dep.get_features().contains_key(feature_name) {
                        return Err(GraphLoadFailureReason::FailedAdditionalProjectValidation {
                          project: Weak::upgrade(&self.current_graph_ref).unwrap(),
                          failure_reason: AdditionalConfigValidationFailureReason::FeatureEnablerDependencyFeatureNotFound {
                            container_feature_name: container_feature_name.to_string(),
                            gcmake_dep_name: dep_name_str.to_string(),
                            feature_name_to_enable: feature_name.to_string()
                          }
                        });
                      }
                    }
                  },
                }
              }
            }
          }
        }
      }

      // Validate paths resolved relative to the project root. Current examples are Windows Executable icon
      // and custom Emscripten HTML file.
      let absolute_project_root: &Path = Path::new(root_project.get_absolute_project_root());

      for (_, target_rc) in self.targets.borrow().iter() {
        let target: &TargetNode = &target_rc.as_ref().borrow();

        let output_executable: Option<&CompiledOutputItem> = match &target.contained_item {
          ContainedItem::PredefinedLibrary { .. } => None,
          ContainedItem::PreBuild(pre_build) => match pre_build {
            PreBuildScript::Exe(pre_build_exe) => Some(pre_build_exe),
            _ => None
          }
          ContainedItem::CompiledOutput(output) => match output.get_output_type() {
            OutputItemType::Executable => Some(output),
            _ => None
          },
        };

        let maybe_windows_icon_relative_path: Option<&Path>;
        let maybe_emscripten_html_relative_path: Option<&Path>;

        match output_executable {
          Some(exe_config) => {
            maybe_windows_icon_relative_path = exe_config.windows_icon_relative_to_root_project.as_ref().map(Path::new);
            maybe_emscripten_html_relative_path = exe_config.emscripten_html_shell_relative_to_project_root.as_ref().map(Path::new);
          },
          None => {
            maybe_windows_icon_relative_path = None;
            maybe_emscripten_html_relative_path = None;
          }
        }

        if let Some(windows_icon_relative_path) = maybe_windows_icon_relative_path {
          let absolute_path_to_icon: PathBuf = absolute_project_root.join(windows_icon_relative_path);

          // TODO: Make sure the final icon path (after normalization) is inside the project root.
          if !absolute_path_to_icon.is_file() {
            return Err(GraphLoadFailureReason::FailedAdditionalProjectValidation {
              project: Weak::upgrade(&self.current_graph_ref).unwrap(),
              failure_reason: AdditionalConfigValidationFailureReason::WindowsIconPathPointsToNonexistent {
                target: Rc::clone(target_rc),
                absolute_path_to_icon,
                given_relative_path: windows_icon_relative_path.to_path_buf()
              }
            });
          }
        }

        if let Some(emscripten_html_relative_path) = maybe_emscripten_html_relative_path {
          let absolute_path_to_html_file: PathBuf = absolute_project_root.join(emscripten_html_relative_path);

          if !absolute_path_to_html_file.is_file() {
            return Err(GraphLoadFailureReason::FailedAdditionalProjectValidation {
              project: Weak::upgrade(&self.current_graph_ref).unwrap(),
              failure_reason: AdditionalConfigValidationFailureReason::EmscriptenHTMLPathPointsToNonexistent {
                target: Rc::clone(target_rc),
                absolute_path_to_html_file,
                given_relative_path: emscripten_html_relative_path.to_path_buf()
              }
            });
          }
        }
      }
    }

    for (_, subproject) in &self.subprojects {
      subproject.as_ref().borrow().do_additional_project_checks()?;
    }

    for (_, test_project) in &self.test_projects {
      test_project.as_ref().borrow().do_additional_project_checks()?;
    }

    for (_, gcmake_dep) in &self.gcmake_deps {
      gcmake_dep.as_ref().borrow().do_additional_project_checks()?;
    }

    Ok(())
  }

  fn ensure_valid_system_spec_links(&self, warning_mode: &DependencyGraphWarningMode) -> Result<(), GraphLoadFailureReason<'a>> {
    for (_, predef_dep_graph) in &self.predefined_deps {
      predef_dep_graph.as_ref().borrow().ensure_valid_system_spec_links(warning_mode)?;
    }

    if let Some(pre_build_target) = &self.pre_build_wrapper {
      Self::check_target_system_spec_links_are_valid(pre_build_target, warning_mode)?;
    }

    for (_, target) in self.targets.borrow().iter() {
      Self::check_target_system_spec_links_are_valid(target, warning_mode)?;
    }

    for (_, test_project) in &self.test_projects {
      test_project.as_ref().borrow().ensure_valid_system_spec_links(warning_mode)?;
    }

    for (_, subproject_graph) in &self.subprojects {
      subproject_graph.as_ref().borrow().ensure_valid_system_spec_links(warning_mode)?;
    }

    for (_, gcmake_dep_graph) in &self.gcmake_deps {
      gcmake_dep_graph.as_ref().borrow().ensure_valid_system_spec_links(warning_mode)?;
    }

    Ok(())
  }

  fn ensure_no_duplicate_identifiers(&self) -> Result<(), GraphLoadFailureReason<'a>> {
    return self.ensure_no_duplicate_identifiers_helper(
      &mut HashMap::new(),
      &mut HashMap::new(),
      &mut HashMap::new()
    );
  }

  // TODO: Make a warning system, then warn when target names are too similar (i.e. match
  // when case-insensitive).
  fn err_if_target_ident_is_duplicate(
    target: &Rc<RefCell<TargetNode<'a>>>,
    cmake_identifiers: &mut HashMap<String, Rc<RefCell<TargetNode<'a>>>>,
    yaml_identifiers: &mut HashMap<String, Rc<RefCell<TargetNode<'a>>>>
  ) -> Result<(), GraphLoadFailureReason<'a>> {
    let borrowed_target = target.as_ref().borrow();

    let target_cmake_ident: &str = borrowed_target.get_cmake_target_base_name();
    let target_yaml_ident: &str = borrowed_target.get_name();

    match yaml_identifiers.get(target_yaml_ident) {
      Some(matching_target) => {
        // Only err if two different targets share the same identifier. Obviously
        // a target has the same identifier as itself. This check is important because
        // dependency targets are checked once for each root project tree they are a part of.
        if matching_target.as_ref().borrow().ne(borrowed_target.borrow()) {
          return Err(GraphLoadFailureReason::DuplicateYamlIdentifier {
            target1: Rc::clone(target),
            target1_project: borrowed_target.container_project(),
            target2: Rc::clone(matching_target),
            target2_project: matching_target.as_ref().borrow().container_project()
          });
        }
      },
      None => {
        yaml_identifiers.insert(target_yaml_ident.to_string(), Rc::clone(target));
      }
    }

    match cmake_identifiers.get(target_cmake_ident) {
      Some(matching_target) => {
        if matching_target.as_ref().borrow().ne(borrowed_target.borrow()) {
          return Err(GraphLoadFailureReason::DuplicateCMakeIdentifier {
            target1: Rc::clone(target),
            target1_project: borrowed_target.container_project(),
            target2: Rc::clone(matching_target),
            target2_project: matching_target.as_ref().borrow().container_project()
          });
        }
      },
      None => {
        cmake_identifiers.insert(target_cmake_ident.to_string(), Rc::clone(target));
      }
    }
    
    Ok(())
  }

  // TODO: Make a warning system, then warn when project names are too similar (i.e. match
  // when case-insensitive).
  fn err_if_root_project_ident_is_duplicate(
    &self, 
    root_project_name_idents: &mut HashMap<String, Rc<RefCell<DependencyGraph<'a>>>>
  ) -> Result<(), GraphLoadFailureReason<'a>> {
    match root_project_name_idents.get(self.project_identifier_name()) {
      Some(matching_project) => {
        if matching_project.as_ref().borrow().ne(self) {
          return Err(GraphLoadFailureReason::DuplicateRootProjectIdentifier {
            project1: Weak::upgrade(&self.current_graph_ref).unwrap(),
            project2: Rc::clone(matching_project)
          });
        }
      },
      None => {
        root_project_name_idents.insert(
          self.project_identifier_name().to_string(),
          Weak::upgrade(&self.current_graph_ref).unwrap()
        );
      }
    }

    Ok(())
  }

  // TODO: Make a warning system, then warn when project names are too similar (i.e. match
  // when case-insensitive).
  fn err_if_subproject_overlaps_dependency_name(
    subproject_graph: &Rc<RefCell<DependencyGraph<'a>>>
  ) -> Result<(), GraphLoadFailureReason<'a>> {
    let subproject_name: String = subproject_graph.as_ref().borrow().project_identifier_name().to_string();
    let root_project: Rc<RefCell<DependencyGraph>> = Weak::upgrade(&subproject_graph.as_ref().borrow().toplevel).unwrap();

    if let Some(matching_predep) = root_project.as_ref().borrow().predefined_deps.get(&subproject_name) {
      return Err(GraphLoadFailureReason::SubprojectNameOverlapsDependency {
        subproject: Rc::clone(subproject_graph),
        dependency_project: Rc::clone(matching_predep)
      });
    }

    if let Some(matching_gcmake_dep) = root_project.as_ref().borrow().gcmake_deps.get(&subproject_name) {
      return Err(GraphLoadFailureReason::SubprojectNameOverlapsDependency {
        subproject: Rc::clone(subproject_graph),
        dependency_project: Rc::clone(matching_gcmake_dep)
      });
    }

    Ok(())
  }

  fn ensure_no_duplicate_identifiers_helper(
    &self,
    cmake_identifiers: &mut HashMap<String, Rc<RefCell<TargetNode<'a>>>>,
    yaml_identifiers: &mut HashMap<String, Rc<RefCell<TargetNode<'a>>>>,
    root_project_name_idents: &mut HashMap<String, Rc<RefCell<DependencyGraph<'a>>>>
  ) -> Result<(), GraphLoadFailureReason<'a>> {
    match self.project_wrapper() {
      ProjectWrapper::GCMakeDependencyRoot(_)
        | ProjectWrapper::PredefinedDependency(_) =>
      {
        self.err_if_root_project_ident_is_duplicate(root_project_name_idents)?;
      },
      ProjectWrapper::NormalProject(normal_project) => {
        if normal_project.is_root_project() {
          self.err_if_root_project_ident_is_duplicate(root_project_name_idents)?;
        }
      }
    }

    for (_, predef_dep_graph) in &self.predefined_deps {
      predef_dep_graph.as_ref().borrow().ensure_no_duplicate_identifiers_helper(
        cmake_identifiers,
        yaml_identifiers,
        root_project_name_idents
      )?;
    }

    if let Some(pre_build_target) = &self.pre_build_wrapper {
      Self::err_if_target_ident_is_duplicate(
        pre_build_target,
        cmake_identifiers,
        yaml_identifiers
      )?;
    }

    for (_, target) in self.targets.borrow().iter() {
      Self::err_if_target_ident_is_duplicate(
        target,
        cmake_identifiers,
        yaml_identifiers
      )?;
    }

    for (_, test_project) in &self.test_projects {
      test_project.as_ref().borrow().ensure_no_duplicate_identifiers_helper(
        cmake_identifiers,
        yaml_identifiers,
        root_project_name_idents
      )?;
    }

    for (_, subproject_graph) in &self.subprojects {
      subproject_graph.as_ref().borrow().ensure_no_duplicate_identifiers_helper(
        cmake_identifiers,
        yaml_identifiers,
        root_project_name_idents
      )?;

      // Ensure no subproject names overlap with the root project's predefined dependencies.
      // This overlap doesn't cause issues in the CMakeLists because the project( ... ) name written
      // is "mangled" (prefixed with the parent project's mangled name). However, it may cause issues
      // when linking from the parent project because the used namespace becomes ambiguous.
      // For example, with a subproject SFML, we can't tell at a glance whether linking to SFML::system
      // links to a target in subproject SFML or predefined dependency SFML.
      Self::err_if_subproject_overlaps_dependency_name(subproject_graph)?;
    }

    for (_, gcmake_dep_graph) in &self.gcmake_deps {
      gcmake_dep_graph.as_ref().borrow().ensure_no_duplicate_identifiers_helper(
        cmake_identifiers,
        yaml_identifiers,
        root_project_name_idents
      )?;
    }

    Ok(())
  }

  /*
    After making associations, ensure correct predefined dependency inclusion for all
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
  fn ensure_proper_predefined_dep_links(&self) -> Result<(), GraphLoadFailureReason<'a>> {
    // "External requirements" for predefined dependencies must be loaded here because
    // we know that all predefined dependencies used by the project have been loaded at this point.
    for (_, predefined_dep) in &self.predefined_deps {
      for (_, target_config) in predefined_dep.as_ref().borrow().targets.borrow().iter() {
        let predep_target: &mut TargetNode = &mut target_config.as_ref().borrow_mut();

        if let ContainedItem::PredefinedLibrary { external_requirements, .. } = predep_target.get_contained_item().clone() {
          for external_requirement_spec in external_requirements {
            match external_requirement_spec {
              FinalExternalRequirementSpecifier::OneOf(link_spec_list) => {
                let mut one_of_target_list: Vec<MaybePresentNonOwningTarget<'a>> = Vec::new();

                for link_spec in link_spec_list {
                  // Each of these link specifiers is guaranteed to have only a single non-nested namespace
                  // and a single library name. The predefined dependency pointed to by the namespace is
                  // guaranteed to exist, but NOT guaranteed to have been imported by the project. The
                  // target name is also guaranteed to exist in that project, but as a result can only
                  // be retrieved if that project has also been imported.
                  let required_predep_name: &str = link_spec.get_namespace_queue().iter().nth(0).unwrap();
                  let required_lib_name: &str = link_spec.get_target_list().iter().nth(0).unwrap().get_name();

                  match self.predefined_deps.get(required_predep_name) {
                    None => {
                      one_of_target_list.push(MaybePresentNonOwningTarget::NotImported {
                        namespaced_yaml_name: link_spec.original_spec_str().to_string()
                      });
                    },
                    Some(required_predep_project) => {
                      let matching_target = required_predep_project
                        .as_ref().borrow()
                        .get_target_in_current_namespace(required_lib_name)
                        .unwrap();

                      one_of_target_list.push(MaybePresentNonOwningTarget::Populated(
                        Rc::downgrade(&matching_target)
                      ));
                    }
                  }
                }

                predep_target.add_complex_requirement(
                  NonOwningComplexTargetRequirement::OneOf(one_of_target_list)
                );
              }
            }
          }
        }
      }
    }

    for (_, project_output_target_rc) in self.targets.borrow().iter() {
      let project_output_target: &mut TargetNode = &mut project_output_target_rc.as_ref().borrow_mut();

      // This is necessary because adding links to the project target inside the loop could mess with
      // the list's iteration. 
      let mut links_to_add: HashMap<TargetId, Link> = HashMap::new();
      let mut all_checked_predef_targets: HashMap<TargetId, Rc<RefCell<TargetNode>>> = HashMap::new();
    
      for (_, link) in &project_output_target.depends_on {
        let upgraded_target = Weak::upgrade(&link.target).unwrap();
        let link_target = upgraded_target.as_ref().borrow();
        let upgraded_target_graph = Weak::upgrade(&link_target.contained_in_graph).unwrap();
        let link_target_graph = upgraded_target_graph.as_ref().borrow();

        if let ProjectWrapper::PredefinedDependency(_) = &link_target_graph._project_wrapper {
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
                existing_link_to_add.link_mode.clone(),
                link.link_mode.clone()
              );

              existing_link_to_add.system_spec_info = existing_link_to_add.system_spec_info.union(&link.system_spec_info);
            }
            else if let Some(existing_link) = project_output_target.depends_on.get(&target_checking_id) {
              // The link already exists and was added by the user. Return an error if the existing link mode
              // is not the same as the one which would be created.
              if existing_link.link_mode != link.link_mode {
                let dependency: Rc<RefCell<TargetNode>> = Weak::upgrade(&link.target).unwrap();
                return Err(GraphLoadFailureReason::WrongUserGivenPredefLinkMode {
                  current_link_mode: existing_link.link_mode.clone(),
                  needed_link_mode: link.link_mode.clone(),
                  target_project: project_output_target.container_project(),
                  target: Rc::clone(&project_output_target_rc),
                  dependency_project: dependency.as_ref().borrow().container_project(),
                  dependency: Rc::clone(&dependency),
                });
              }
            }
            else {
              // The link is not present. Just add it to links_to_add.
              links_to_add.insert(
                target_checking_id,
                Link::new(
                  predef_target_checking.locator_name.clone(),
                  // predef_target_checking.system_specifier_info.clone(),
                  predef_target_checking.system_specifier_info.intersection(link_target.get_system_spec_info()),
                  Rc::downgrade(&target_checking_rc),
                  link.link_mode.clone()
                )
              );
            }
          }

          for (predef_target_id, predef_target) in checked_predef_targets {
            all_checked_predef_targets.entry(predef_target_id)
              .or_insert(predef_target);
          }
        }
      }

      for (_, link) in links_to_add {
        project_output_target.insert_link(link);
      }

      // At this point, all basic single requirements for the target have been met. Any links
      // which could be automatically added to the target to satisfy the single interdependencies
      // have been. Now we can check for any complex requirements.

      // NOTE: Right now, these associations/requirements are only checked on depenendencies of a target,
      // not on the target itself. This is fine since these requirements are only specified for predefined
      // dependencies. However, if a change is made which allows for these additional interdependent
      // requirements (like mutual exclusion) to be specified for normal gcmake projects, then this will have
      // to be checked per target as well.
      for (_, link) in &project_output_target.depends_on {
        let link_target: Rc<RefCell<TargetNode>> = Weak::upgrade(&link.target).unwrap();

        for complex_requirement in &link_target.as_ref().borrow().complex_requirements {
          match complex_requirement {
            NonOwningComplexTargetRequirement::OneOf(target_list) => {
              let has_requirement_target: bool = target_list
                .iter()
                .any(|maybe_populated_target| match maybe_populated_target {
                  MaybePresentNonOwningTarget::NotImported { .. } => false,
                  MaybePresentNonOwningTarget::Populated(maybe_needed_target) => {
                    let id_searching: TargetId = Weak::upgrade(maybe_needed_target).unwrap().as_ref().borrow().unique_target_id();
                    project_output_target.depends_on.contains_key(&id_searching)
                  }
                });

              if !has_requirement_target {
                return Err(GraphLoadFailureReason::ComplexTargetRequirementNotSatisfied {
                  target: Rc::clone(project_output_target_rc),
                  target_project: project_output_target.container_project(),
                  dependency_project: link_target.as_ref().borrow().container_project(),
                  dependency: Rc::clone(&link_target),
                  failed_requirement: OwningComplexTargetRequirement::new_from(complex_requirement)
                });
              }
            },
            NonOwningComplexTargetRequirement::ExclusiveFrom(exclusion_target_weak) => {
              let exclusion_target_id: TargetId = Weak::upgrade(exclusion_target_weak).unwrap()
                .as_ref()
                .borrow()
                .unique_target_id();

              if project_output_target.depends_on.contains_key(&exclusion_target_id) {
                return Err(GraphLoadFailureReason::ComplexTargetRequirementNotSatisfied {
                  target: Rc::clone(project_output_target_rc),
                  target_project: project_output_target.container_project(),
                  dependency_project: link_target.as_ref().borrow().container_project(),
                  dependency: Rc::clone(&link_target),
                  failed_requirement: OwningComplexTargetRequirement::new_from(complex_requirement)
                })
              }
            }
          }
        }
      }
    }
    
    for (_, subproject) in &self.subprojects {
      subproject.as_ref().borrow().ensure_proper_predefined_dep_links()?;
    }

    for (_, test_project) in &self.test_projects {
      test_project.as_ref().borrow().ensure_proper_predefined_dep_links()?;
    }

    for (_, gcmake_dep) in &self.gcmake_deps {
      gcmake_dep.as_ref().borrow().ensure_proper_predefined_dep_links()?;
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
  fn make_auto_inner_project_link_associations(&self) -> Result<(), GraphLoadFailureReason<'a>> {
    if let Some(pre_build_target) = &self.pre_build_wrapper {
      // All project output targets must depend on the project's pre-build script in order
      // for project targets to be ordered and checked for cycles correctly.
      for (_, project_output_rc) in self.targets.borrow().iter() {
        let project_output_target: &mut TargetNode = &mut project_output_rc.as_ref().borrow_mut();
        let pre_build_name: String = self._project_wrapper.maybe_normal_project()
          .map(|project_info| project_info.prebuild_script_name())
          .unwrap_or(String::from("Pre-build script"));

        project_output_target.insert_link(Link::new(
          pre_build_name,
          SystemSpecifierWrapper::default_include_all(),
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
      for (_, test_target_rc) in test_project.as_ref().borrow().targets.borrow().iter() {
        let test_target: &mut TargetNode = &mut test_target_rc.as_ref().borrow_mut();

        for (project_output_name, project_output_rc) in self.targets.borrow().iter() {
          let spec_info_clone: SystemSpecifierWrapper = {
            project_output_rc.as_ref().borrow().system_specifier_info.clone()
          };

          test_target.insert_link(Link::new(
            project_output_name.to_string(),
            spec_info_clone,
            Rc::downgrade(project_output_rc),
            LinkMode::Private
          ));
        }
      }

      test_project.as_ref().borrow().make_auto_inner_project_link_associations()?;
    }

    for (_, subproject) in &self.subprojects {
      subproject.as_ref().borrow().make_auto_inner_project_link_associations()?;
    }

    for (_, gcmake_dep_project) in &self.gcmake_deps {
      gcmake_dep_project.as_ref().borrow().make_auto_inner_project_link_associations()?;
    }

    Ok(())
  }

  fn make_given_link_associations(
    &self,
    // Needed for creating placeholder targets in gcmake dependency projects which haven't been cloned yet.
    target_id_counter: &mut i32
  ) -> Result<(), GraphLoadFailureReason<'a>> {
    for (link_receiver_name, target_container) in self.targets.borrow().iter() {
      self.resolve_and_apply_target_links(
        target_id_counter,
        Rc::clone(target_container),
        &mut target_container.as_ref().borrow_mut(),
        link_receiver_name
      )?;
    }

    if let Some(pre_build_target) = &self.pre_build_wrapper {
      let borrowed_target: &mut TargetNode = &mut pre_build_target.as_ref().borrow_mut();

      self.resolve_and_apply_target_links(
        target_id_counter,
        Rc::clone(pre_build_target),
        borrowed_target,
        &borrowed_target.get_name().to_string()
      )?;
    }

    for (_, subproject) in &self.subprojects {
      subproject.as_ref().borrow().make_given_link_associations(target_id_counter)?;
    }

    for (_, test_project) in &self.test_projects {
      test_project.as_ref().borrow().make_given_link_associations(target_id_counter)?;
    }

    // This allows links for an entire GCMake project tree to be checked, including
    // dependencies. This means the available GCMake dependencies can also have their
    // CMake configurations written, although this is not done currently. It probably
    // should be though.
    for (_, gcmake_dep) in &self.gcmake_deps {
      gcmake_dep.as_ref().borrow().make_given_link_associations(target_id_counter)?;
    }

    return Ok(());
  }

  fn resolve_and_apply_target_links(
    &self,
    target_id_counter: &mut i32,
    target_container: Rc<RefCell<TargetNode<'a>>>,
    mut_target_node: &mut TargetNode<'a>,
    link_receiver_name: &str
  ) -> Result<(), GraphLoadFailureReason<'a>> {
    // let mut_target_node: &mut TargetNode = &mut target_container.as_ref().borrow_mut();

    if let Some(link_specs) = &mut_target_node.raw_link_specifiers.clone() {
      let public_links: HashSet<Link> = self.resolve_links(
        target_id_counter,
        &target_container,
        mut_target_node,
        &link_specs.cmake_public,
        &LinkMode::Public,
      )?;

      self.apply_link_set_to_target(
        mut_target_node,
        link_receiver_name,
        public_links
      )?;

      let interface_links: HashSet<Link> = self.resolve_links(
        target_id_counter,
        &target_container,
        mut_target_node,
        &link_specs.cmake_interface,
        &LinkMode::Interface
      )?;

      self.apply_link_set_to_target(
        mut_target_node,
        link_receiver_name,
        interface_links
      )?;

      let private_links: HashSet<Link> = self.resolve_links(
        target_id_counter,
        &target_container,
        mut_target_node,
        &link_specs.cmake_private,
        &LinkMode::Private
      )?;

      self.apply_link_set_to_target(
        mut_target_node,
        link_receiver_name,
        private_links
      )?;
    }

    return Ok(());
  }

  fn apply_link_set_to_target(
    &self,
    mut_target_node: &mut TargetNode<'a>,
    link_receiver_name: &str,
    link_set: HashSet<Link<'a>>
  ) -> Result<(), GraphLoadFailureReason<'a>> {
    let link_receiver = mut_target_node;

    for link in link_set {
      let borrowed_target = Weak::upgrade(&link.target).unwrap();
      let link_giver: &TargetNode = &borrowed_target.as_ref().borrow();
      let link_giver_graph: Rc<RefCell<DependencyGraph>> = Weak::upgrade(&link_giver.contained_in_graph).unwrap();
      let link_receiver_graph: Rc<RefCell<DependencyGraph>> = Weak::upgrade(&link_receiver.contained_in_graph).unwrap();

      // Targets cannot link to themselves.
      if link_receiver.unique_target_id() == link_giver.unique_target_id() { 
        return Err(GraphLoadFailureReason::LinkedToSelf {
          project: link_receiver.container_project(),
          target_name: link_receiver_name.to_string()
        });
      }
      else if let Some(existing_link) = link_receiver.depends_on.get(&link.target_id()) {
        // Targets can only be linked in a single categry. I.E. it doesn't make sense
        // to link a target as both PUBLIC and INTERFACE.
        if existing_link.link_mode != link.link_mode {
          return Err(GraphLoadFailureReason::LinkedInMultipleCategories {
            current_link_mode: existing_link.link_mode.clone(),
            attempted_link_mode: link.link_mode,
            link_giver_project: link_giver_graph,
            link_giver_name: link.target_name.to_string(),
            link_receiver_project: link_receiver_graph,
            link_receiver_name: link_receiver_name.to_string()
          });
        }
      }
      else {
        link_receiver.insert_link(link);
      }
    }

    Ok(())
  }

  fn resolve_links(
    &self,
    target_id_counter: &mut i32,
    target_container: &Rc<RefCell<TargetNode<'a>>>,
    mut_target_node: &mut TargetNode<'a>,
    link_specs: &Vec<LinkSpecifier>,
    link_mode: &LinkMode
  ) -> Result<HashSet<Link<'a>>, GraphLoadFailureReason<'a>> {
    /*
      Resolution scenarios:
        - root::{target, names}
        - parent::{target, names}
          -> Useful when nested subprojects need to depend on each other
        - dependency_name::{target, names}
          -> 'dependency_name' is a placeholder for any string other than 'root' and 'parent'.
    */

    let mut link_set: HashSet<Link> = HashSet::new();

    for link_spec in link_specs {
      let mut namespace_queue: VecDeque<String> = link_spec.get_namespace_queue().clone();

      let resolved_links: HashSet<Link> = self.resolve_namespace_helper(
        link_spec,
        target_container,
        mut_target_node,
        target_id_counter,
        &mut namespace_queue,
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
        
        if link_set.contains(&link) {
          return Err(GraphLoadFailureReason::DuplicateLinkTarget {
            dependency: Weak::upgrade(&link.target).unwrap(),
            dependency_project: Weak::upgrade(&link.target).unwrap().as_ref().borrow().container_project(),
            link_spec_container_project: mut_target_node.container_project(),
            link_spec_container_target: Rc::clone(target_container)
          })
        }
        else {
          link_set.insert(link);
        }
      }
    }

    return Ok(link_set);
  }

  fn resolve_namespace_helper(
    &self,
    whole_link_spec: &LinkSpecifier,
    link_spec_container_target: &Rc<RefCell<TargetNode<'a>>>,
    mut_target_node: &mut TargetNode<'a>,
    target_id_counter: &mut i32,
    namespace_queue: &mut VecDeque<String>,
    target_list: &LinkSpecTargetList,
    link_mode: &LinkMode,
    access_mode: &LinkAccessMode,
    is_outside_original_project_context: bool
  ) -> Result<HashSet<Link<'a>>, GraphLoadFailureReason<'a>> {
    if namespace_queue.is_empty() {
      let mut accumulated_link_set: HashSet<Link> = HashSet::new();

      for link_target_spec in target_list {
        let resolved_link: Link = self.resolve_target_into_link(
          whole_link_spec,
          link_spec_container_target,
          mut_target_node,
          target_id_counter,
          link_target_spec,
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
      let next_namespace: String = namespace_queue.pop_front().unwrap();

      let next_graph: Rc<RefCell<DependencyGraph>>;
      let will_be_outside_original_project_context: bool;

      match &next_namespace[..] {
        "root" => {
          next_graph = Weak::upgrade(&self.toplevel).unwrap();
          assert!(
            self.root_project_id() == next_graph.as_ref().borrow().root_project_id(),
            "Root project tree should not change when resolving to the 'root' graph."
          );
          will_be_outside_original_project_context = false;
        },
        "parent" | "super" => match &self.parent {
          Some(parent_graph) => {
            next_graph = Weak::upgrade(parent_graph).unwrap();
            assert!(
              self.root_project_id() == next_graph.as_ref().borrow().root_project_id(),
              "Root project tree should not change when resolving to the 'parent' graph. This is because project root graphs (including dependency projects) are not given a parent."
            );
            // Dependency project roots never have a parent graph, and therefore referencing a
            // project's "parent" will never resolve to a context outside that project root's tree.
            will_be_outside_original_project_context = false;
          },
          None => return Err(GraphLoadFailureReason::LinkPointsToInvalidOrNonexistentProject {
            target: Rc::clone(link_spec_container_target),
            project: mut_target_node.container_project(),
            link_spec: whole_link_spec.clone()
          })
        },
        namespace_to_resolve => {
          let root_project: Rc<RefCell<DependencyGraph>> = Weak::upgrade(&self.toplevel).unwrap();

          if is_outside_original_project_context {
            return Err(GraphLoadFailureReason::LinkNestedNamespaceInOtherProjectContext {
              target: Rc::clone(link_spec_container_target),
              project: mut_target_node.container_project(),
              link_spec: whole_link_spec.clone()
            })
          }
          else if let Some(matching_subproject) = self.subprojects.get(namespace_to_resolve) {
            next_graph = Rc::clone(matching_subproject);
            assert!(
              self.root_project_id() == next_graph.as_ref().borrow().root_project_id(),
              "Root project tree should not change when resolving to a subproject graph."
            );
            will_be_outside_original_project_context = false;
          }
          else if let Some(matching_predef_dep) = root_project.as_ref().borrow().predefined_deps.get(namespace_to_resolve) {
            next_graph = Rc::clone(matching_predef_dep);
            assert!(
              self.root_project_id() != next_graph.as_ref().borrow().root_project_id(),
              "Root project tree must change when resolving to a predefined_dependency graph."
            );
            will_be_outside_original_project_context = true;
          }
          else if let Some(gcmake_dep) = root_project.as_ref().borrow().gcmake_deps.get(namespace_to_resolve) {
            next_graph = Rc::clone(gcmake_dep);
            assert!(
              self.root_project_id() != next_graph.as_ref().borrow().root_project_id(),
              "Root project tree must change when resolving to a gcmake_dependency graph."
            );
            will_be_outside_original_project_context = true;
          }
          else {
            return Err(GraphLoadFailureReason::LinkPointsToInvalidOrNonexistentProject {
              target: Rc::clone(link_spec_container_target),
              project: mut_target_node.container_project(),
              link_spec: whole_link_spec.clone()
            }) 
          }
        }
      }

      return next_graph.as_ref().borrow().resolve_namespace_helper(
        whole_link_spec,
        link_spec_container_target,
        mut_target_node,
        target_id_counter,
        namespace_queue,
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
    whole_link_spec: &LinkSpecifier,
    link_spec_container_target: &Rc<RefCell<TargetNode<'a>>>,
    mut_target_node: &mut TargetNode<'a>,
    target_id_counter: &mut i32,
    link_target_spec: &LinkSpecifierTarget,
    link_mode: &LinkMode,
    using_access_mode: &LinkAccessMode
  ) -> Result<Link<'a>, GraphLoadFailureReason<'a>> {
    if let ProjectWrapper::GCMakeDependencyRoot(gcmake_dep) = &self._project_wrapper {
      if let GCMakeDependencyStatus::NotDownloaded(_) = gcmake_dep.project_status() {
        // Targets should be created on the fly.
        let mut target_map = self.targets.borrow_mut();
        let linkable_name: String = gcmake_dep.get_linkable_target_name(link_target_spec.get_name());

        let new_placeholder_target: Rc<RefCell<TargetNode>> = Rc::new(RefCell::new(TargetNode::new(
          target_id_counter,
          link_target_spec.get_name(),
          // Placeholder targets are assumed to work on all platforms and systems
          SystemSpecifierWrapper::default_include_all(),
          linkable_name.clone(),
          linkable_name.clone(),
          linkable_name.clone(),
          linkable_name,
          false,
          Weak::clone(&self.current_graph_ref),
          ContainedItem::PredefinedLibrary {
            target_name: link_target_spec.get_name().to_string(),
            external_requirements: Vec::new()
          },
          LinkAccessMode::UserFacing,
          true
        )));

        let the_link = Ok(Link::new(
          link_target_spec.get_name().to_string(),
          link_target_spec.get_system_spec_info().clone(),
          Rc::downgrade(&new_placeholder_target),
          link_mode.clone()
        ));

        target_map.insert(link_target_spec.get_name().to_string(), new_placeholder_target);
        return the_link;
      }
    }

    let maybe_resolved_link: Option<Link> = self.resolve_target_into_link_helper(
      whole_link_spec,
      link_spec_container_target,
      mut_target_node,
      target_id_counter,
      link_target_spec,
      link_mode,
      using_access_mode
    )?;

    return match maybe_resolved_link {
      Some(resolved_link) => Ok(resolved_link),
      None => Err(GraphLoadFailureReason::LinkTargetNotFound {
        target: Rc::clone(link_spec_container_target),
        link_spec: whole_link_spec.clone(),
        looking_in_project: Weak::upgrade(&self.current_graph_ref).unwrap(),
        target_container_project: mut_target_node.container_project(),
        name_searching: link_target_spec.get_name().to_string()
      })
    }
  }

  fn resolve_target_into_link_helper(
    &self,
    whole_link_spec: &LinkSpecifier,
    link_spec_container_target: &Rc<RefCell<TargetNode<'a>>>,
    mut_target_node: &mut TargetNode<'a>,
    target_id_counter: &mut i32,
    link_target_spec: &LinkSpecifierTarget,
    link_mode: &LinkMode,
    using_access_mode: &LinkAccessMode
  ) -> Result<Option<Link<'a>>, GraphLoadFailureReason<'a>> {
    if let Some(found_target) = self.targets.borrow().get(link_target_spec.get_name()) {
      if !using_access_mode.satisfies(&found_target.as_ref().borrow().visibility) {
        return Err(GraphLoadFailureReason::AccessNotAllowed {
          dependency: Rc::clone(found_target),
          dependency_project: found_target.as_ref().borrow().container_project(),
          link_spec: whole_link_spec.clone(),
          link_spec_container_project: mut_target_node.container_project(),
          link_spec_container_target: Rc::clone(link_spec_container_target),
          given_access_mode: using_access_mode.clone(),
          needed_access_mode: found_target.as_ref().borrow().visibility.clone()
        });
      }
      // TODO: Re-enable once system spec expression subset checks have been implemented.
      // else if !link_target_spec.get_system_spec_info().is_subset_of(&found_target.as_ref().borrow().system_specifier_info) {
      //   return Err(GraphLoadFailureReason::LinkSystemSubsetMismatch {
      //     dependency: Rc::clone(found_target),
      //     dependency_project: found_target.as_ref().borrow().container_project(),
      //     link_spec: Some(whole_link_spec.clone()),
      //     link_system_spec_info: link_target_spec.clone().get_system_spec_info().clone(),
      //     link_spec_container_project: mut_target_node.container_project(),
      //     link_spec_container_target: Rc::clone(link_spec_container_target),
      //     transitively_required_by: None
      //   })
      // }
      else {
        return Ok(Some(Link::new(
          link_target_spec.get_name().to_string(),
          link_target_spec.get_system_spec_info().clone(),
          Rc::downgrade(found_target),
          link_mode.clone()
        )))
      }
    }

    for (_, subproject) in &self.subprojects {
      let maybe_link: Option<Link> = subproject.as_ref().borrow().resolve_target_into_link_helper(
        whole_link_spec,
        link_spec_container_target,
        mut_target_node,
        target_id_counter,
        link_target_spec,
        link_mode,
        using_access_mode
      )?;

      if let Some(resolved_link) = maybe_link {
        return Ok(Some(resolved_link));
      }
    }

    return Ok(None);
  }

  pub fn find_single_target_by_name(&self, target_name: impl AsRef<str>) -> BasicTargetSearchResult<'a> {
    return BasicTargetSearchResult {
      target: self.get_target_in_current_namespace(target_name.as_ref()),
      searched_project: Weak::upgrade(&self.current_graph_ref).unwrap(),
      searched_with: target_name.as_ref().to_string(),
      target_name_searched: target_name.as_ref().to_string()
    };
  }

  pub fn find_targets_using_name_list(
    &self,
    target_list: &Vec<impl AsRef<str>>
  ) -> Vec<BasicTargetSearchResult<'a>> {
    return target_list.iter()
      .map(|target_name| self.find_single_target_by_name(target_name))
      .collect();
  }

  pub fn find_projects_using_name_list(
    &self,
    project_list: &Vec<impl AsRef<str>>
  ) -> Result<Vec<BasicProjectSearchResult<'a>>, String> {
    return project_list.iter()
      .map(|project_name| {
        let project_search_result = self.get_project_in_current_namespace(project_name.as_ref(), true)?;

        Ok(BasicProjectSearchResult {
          project: project_search_result,
          searched_project: Weak::upgrade(&self.current_graph_ref).unwrap(),
          searched_with: project_name.as_ref().to_string(),
          project_name_searched: project_name.as_ref().to_string()
        })
      })
      .collect();
  }

  pub fn find_targets_using_link_spec(
    &self,
    in_current_project_only: bool,
    link_spec: &LinkSpecifier
  ) -> Result<Vec<BasicTargetSearchResult<'a>>, String> {
    let mut search_results: Vec<Option<Rc<RefCell<TargetNode>>>> = Vec::new();
    let mut namespace_queue = link_spec.get_namespace_queue().clone();
    let combined_stack_string: String = LinkSpecifier::join_some_namespace_queue(&namespace_queue);

    assert!(
      !namespace_queue.is_empty(),
      "When searching for targets using a link specifier, the namespace queue must contain at least one value."
    );

    let project_from_namespace = self.basic_namespace_resolve(
      in_current_project_only,
      &mut namespace_queue,
      link_spec
    )?;

    for target in link_spec.get_target_list() {
      search_results.push(
        project_from_namespace.as_ref().borrow().get_target_in_current_namespace(target.get_name())
      );
    }

    return Ok(
      search_results.into_iter()
        .enumerate()
        .map(|(index, target)| {
          let target_name_searched: String = link_spec.get_target_list()[index].get_name().to_string();

          BasicTargetSearchResult {
            target,
            searched_project: Weak::upgrade(&self.current_graph_ref).unwrap(),
            searched_with: format!(
              "{}::{}",
              combined_stack_string,
              &target_name_searched
            ),
            target_name_searched
          }
        })
        .collect()
    )
  }

  pub fn find_projects_using_link_spec(
    &self,
    in_current_project_only: bool,
    link_spec: &LinkSpecifier
  ) -> Result<Vec<BasicProjectSearchResult<'a>>, String> {
    let mut search_results: Vec<Option<Rc<RefCell<DependencyGraph<'a>>>>> = Vec::new();
    let mut namespace_queue = link_spec.get_namespace_queue().clone();
    let combined_stack_string: String = LinkSpecifier::join_some_namespace_queue(&namespace_queue);

    assert!(
      !namespace_queue.is_empty(),
      "When searching for projects using a link specifier, the namespace queue must contain at least one value."
    );

    let project_resolving_from: Rc<RefCell<DependencyGraph<'a>>> = if in_current_project_only {
      Weak::upgrade(&self.current_graph_ref).unwrap()
    }
    else {
      let project_name: String = namespace_queue.pop_front().unwrap();

      match self.get_project_in_current_namespace(&project_name, true)? {
        Some(matching_project) => matching_project,
        None => return Err(format!(
          "Unable to find a subproject, test project, or dependency named '{}' associated with [{}]",
          project_name,
          self.project_debug_name()
        ))
      }
    };

    let project_from_namespace = project_resolving_from.as_ref().borrow().basic_namespace_resolve(
      true,
      &mut namespace_queue,
      link_spec
    )?;

    for target in link_spec.get_target_list() {
      search_results.push(
        project_from_namespace.as_ref().borrow().get_project_in_current_namespace(target.get_name(), false)?
      );
    }

    return Ok(
      search_results.into_iter()
        .enumerate()
        .map(|(index, project)| {
          let project_name_searched: String = link_spec.get_target_list()[index].get_name().to_string();

          BasicProjectSearchResult {
            project,
            searched_project: Weak::upgrade(&self.current_graph_ref).unwrap(),
            searched_with: format!(
              "{}::{}",
              combined_stack_string,
              &project_name_searched
            ),
            project_name_searched
          }
        })
        .collect()
    )
  }


  fn basic_namespace_resolve(
    &self,
    in_current_project_only: bool,
    namespace_queue: &mut VecDeque<String>,
    whole_link_spec: &LinkSpecifier
  ) -> Result<Rc<RefCell<DependencyGraph<'a>>>, String> {
    if namespace_queue.is_empty() {
      return Ok(Weak::upgrade(&self.current_graph_ref).unwrap());
    }
    else {
      match namespace_queue.pop_front().unwrap().as_str() {
        "self" => {
          return self.basic_namespace_resolve(
            in_current_project_only,
            namespace_queue,
            whole_link_spec
          );
        },
        "root" => {
          return self.root_project().as_ref().borrow().basic_namespace_resolve(
            in_current_project_only,
            namespace_queue,
            whole_link_spec
          );
        },
        namespace_ident @ ("super" | "parent") => match &self.parent {
          Some(parent_graph) => {
            return Weak::upgrade(parent_graph).unwrap().as_ref().borrow().basic_namespace_resolve(
              in_current_project_only,
              namespace_queue,
              whole_link_spec
            );
          },
          None => {
            return Err(format!(
              "Error while trying to resolve namespace '{}' ({}::) in [{}]:\n\t[{}] doesn't have a parent project.",
              namespace_ident,
              LinkSpecifier::join_some_namespace_queue(whole_link_spec.get_namespace_queue()),
              self.project_debug_name(),
              self.project_identifier_name()
            ));
          }
        },
        namespace_ident => {
          let next_project_checking: &Rc<RefCell<DependencyGraph>> = if let Some(matching_subproject) = self.subprojects.get(namespace_ident) {
            matching_subproject
          }
          else if let Some(matching_test_project) = self.test_projects.get(namespace_ident) {
            matching_test_project
          }
          else if let Some(matching_gcmake_dep) = self.gcmake_deps.get(namespace_ident) {
            if in_current_project_only {
              let root_project_debug_name: String = self.root_project().as_ref().borrow().project_debug_name().to_string();
              return Err(format!(
                "Error while trying to resolve namespace '{}' ({}::) in [{}]:\n\t'{}' points to a valid GCMake dependency project of the project root [{}], however the search is currently limited to outputs of [{}] (not dependencies).",
                namespace_ident,
                LinkSpecifier::join_some_namespace_queue(whole_link_spec.get_namespace_queue()),
                self.project_debug_name(),
                namespace_ident,
                &root_project_debug_name,
                &root_project_debug_name
              ));
            }
            matching_gcmake_dep
          }
          else if let Some(matching_predef_dep) = self.predefined_deps.get(namespace_ident) {
            if in_current_project_only {
              let root_project_debug_name: String = self.root_project().as_ref().borrow().project_debug_name().to_string();
              return Err(format!(
                "Error while trying to resolve namespace '{}' ({}::) in [{}]:\n\t'{}' points to a valid predefined dependency project of the project root [{}], however the search is currently limited to outputs of [{}] (not dependencies).",
                namespace_ident,
                LinkSpecifier::join_some_namespace_queue(whole_link_spec.get_namespace_queue()),
                self.project_debug_name(),
                namespace_ident,
                &root_project_debug_name,
                &root_project_debug_name
              ));
            }
            matching_predef_dep
          }
          else {
            return Err(format!(
              "Error while trying to resolve namespace '{}' ({}::) in [{}]:\n\t- '{}' doesn't point to any existing subproject, test project, or dependency.",
              namespace_ident,
              LinkSpecifier::join_some_namespace_queue(whole_link_spec.get_namespace_queue()),
              self.project_debug_name(),
              namespace_ident
            ));
          };

          return next_project_checking.as_ref().borrow().basic_namespace_resolve(
            in_current_project_only,
            namespace_queue,
            whole_link_spec
          );
        }
      }
    }
  }

  fn get_target_in_current_namespace(&self, target_name: &str) -> Option<Rc<RefCell<TargetNode<'a>>>> {
    if target_name == "pre-build" {
      return self.pre_build_wrapper.clone();
    }

    if let Some(matching_target) = self.targets.borrow().get(target_name) {
      return Some(Rc::clone(matching_target));
    }

    for (_, subproject_graph) in &self.subprojects {
      if let Some(matching_target) = subproject_graph.as_ref().borrow().get_target_in_current_namespace(target_name) {
        return Some(matching_target);
      }
    }

    for (_, test_project_graph) in &self.test_projects {
      if let Some(matching_target) = test_project_graph.as_ref().borrow().get_target_in_current_namespace(target_name) {
        return Some(matching_target)
      }
    }

    None
  }

  fn get_project_in_current_namespace(
    &self,
    project_name: &str,
    is_finding_initial_namespace: bool
  ) -> Result<Option<Rc<RefCell<DependencyGraph<'a>>>>, String> {
    if project_name == "self" || project_name == self.project_identifier_name() {
      return Ok(Some(Weak::upgrade(&self.current_graph_ref).unwrap()));
    }

    if is_finding_initial_namespace {
      match project_name {
        namespace_ident @ ("super" | "parent") => match &self.parent {
          None => return Err(format!(
            "Error resolving initial project '{}' in [{}]: [{}] doesn't have a parent project.",
            namespace_ident,
            self.project_debug_name(),
            self.project_identifier_name()
          )),
          Some(parent_graph) => return Ok(Some(Weak::upgrade(parent_graph).unwrap()))
        },
        "root" => {
          return Ok(Some(self.root_project()))
        },
        _ => ()
      }
    }

    for (_, subproject_graph) in &self.subprojects {
      let search_result = subproject_graph.as_ref().borrow().get_project_in_current_namespace(project_name, false)?;
      if let Some(matching_project) = search_result {
        return Ok(Some(matching_project));
      }
    }

    for (_, test_project_graph) in &self.test_projects {
      let search_result = test_project_graph.as_ref().borrow().get_project_in_current_namespace(project_name, false)?;
      if let Some(matching_project) = search_result {
        return Ok(Some(matching_project));
      }
    }

    if is_finding_initial_namespace {
      // Dependencies are only stored in the project root.
      let project_root_rc = self.root_project();
      let project_root = project_root_rc.as_ref().borrow();

      if let Some(matching_project) = project_root.predefined_deps.get(project_name) {
        return Ok(Some(Rc::clone(matching_project)));
      }
      else if let Some(matching_project) = project_root.gcmake_deps.get(project_name) {
        return Ok(Some(Rc::clone(matching_project)));
      }
    }

    Ok(None)
  }

  pub fn find_using_project_data(&self, project_data: &Rc<FinalProjectData>) -> Option<Rc<RefCell<DependencyGraph<'a>>>> {
    if let Some(normal_project) = self.project_wrapper().maybe_normal_project() {
      if normal_project.get_absolute_project_root() == project_data.get_absolute_project_root() {
        return Some(Weak::upgrade(&self.current_graph_ref).unwrap());
      }
    }

    let project_iter = self.subprojects.iter()
      .chain(self.test_projects.iter())
      .chain(self.gcmake_deps.iter());

    for (_, project_checking) in project_iter {
      if let Some(matching_project) = project_checking.as_ref().borrow().find_using_project_data(project_data) {
        return Some(matching_project);
      }
    }

    return None;
  }

  fn recurse_root_project(
    target_id_counter: &mut TargetId,
    graph_id_counter: &mut ProjectId,
    project: &'a Rc<FinalProjectData>
  ) -> Rc<RefCell<DependencyGraph<'a>>> {
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
    project: &'a Rc<FinalProjectData>,
    parent_graph: &mut DependencyGraph<'a>,
    toplevel_graph: Weak<RefCell<DependencyGraph<'a>>>
  ) -> Rc<RefCell<DependencyGraph<'a>>> {
    Self::recurse_project_helper(
      target_id_counter,
      graph_id_counter,
      project,
      Some(parent_graph),
      Some(toplevel_graph)
    )
  }

  fn recurse_project_helper(
    target_id_counter: &mut TargetId,
    graph_id_counter: &mut ProjectId,
    project: &'a Rc<FinalProjectData>,
    parent_graph: Option<&mut DependencyGraph<'a>>,
    toplevel_graph: Option<Weak<RefCell<DependencyGraph<'a>>>>
  ) -> Rc<RefCell<DependencyGraph<'a>>> {
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

    let graph: Rc<RefCell<DependencyGraph>> = Rc::new(RefCell::new(Self {
      graph_id: *graph_id_counter,
      project_group_id: match project.get_project_type() {
        // Test projects are considered part of the same group as their parent graph.
        FinalProjectType::Test { .. } => parent_graph.as_ref().unwrap().project_group_id.clone(),
        _ => ProjectGroupId(*graph_id_counter)
      },

      parent: parent_graph
        .map(|pg| Weak::clone(&pg.current_graph_ref)),
      toplevel: Weak::new(),
      current_graph_ref: Weak::new(),

      _project_wrapper: ProjectWrapper::NormalProject(Rc::clone(project)),
      targets: RefCell::new(HashMap::new()),
      pre_build_wrapper: None,

      subprojects: HashMap::new(),
      test_projects: HashMap::new(),
      predefined_deps: HashMap::new(),
      gcmake_deps: HashMap::new()
    }));

    *graph_id_counter += 1;

    let mut mut_graph_borrow =  graph.as_ref().borrow_mut();
    let mut_graph: &mut DependencyGraph = &mut mut_graph_borrow;

    match &toplevel_graph {
      Some(existing_toplevel) => {
        mut_graph.toplevel = Weak::clone(existing_toplevel);
      },
      None => {
        mut_graph.toplevel = Rc::downgrade(&graph);
      }
    }

    mut_graph.toplevel = toplevel_graph.unwrap_or(Rc::downgrade(&graph));
    mut_graph.current_graph_ref = Rc::downgrade(&graph);

    mut_graph.pre_build_wrapper = project.get_prebuild_script().as_ref()
      .map(|pre_build_script| {
        let pre_build_name: String = project.prebuild_script_name();

        Rc::new(RefCell::new(TargetNode::new(
          target_id_counter,
          &pre_build_name,
          SystemSpecifierWrapper::default_include_all(),
          pre_build_name.clone(),
          project.receiver_lib_name(&pre_build_name),
          project.prefix_with_project_namespace(&pre_build_name),
          project.prefix_with_project_namespace(&pre_build_name),
          false,
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

        let target_output_name: String = if project.is_test_project()
          { project.full_test_name(target_name) }
          else { target_name.clone() };

        (
          target_name.to_string(),
          Rc::new(RefCell::new(TargetNode::new(
            target_id_counter,
            target_name,
            output_item.system_specifier.clone(),
            target_output_name.clone(),
            project.receiver_lib_name(&target_output_name),
            project.prefix_with_project_namespace(&target_output_name),
            project.prefix_with_project_namespace(&target_output_name),
            false,
            Rc::downgrade(&graph),
            ContainedItem::CompiledOutput(output_item),
            access_mode,
            can_link_to
          )))
        )
      })
      .collect();

    mut_graph.targets = RefCell::new(target_map);

    mut_graph.subprojects = project.get_subprojects()
      .iter()
      .map(|(subproject_name, subproject)| {
        (
          subproject_name.to_string(),
          Self::recurse_nested_project(
            target_id_counter,
            graph_id_counter,
            subproject,
            mut_graph,
            Weak::clone(&mut_graph.toplevel)
          ) 
        )
      })
      .collect();

    mut_graph.test_projects = project.get_test_projects()
      .iter()
      .map(|(test_project_name, test_project)| {
        (
          test_project_name.to_string(),
          Self::recurse_nested_project(
            target_id_counter,
            graph_id_counter,
            test_project,
            mut_graph,
            Weak::clone(&mut_graph.toplevel)
          ) 
        )
      })
      .collect();

    mut_graph.predefined_deps = project.get_predefined_dependencies()
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

    mut_graph.gcmake_deps = project.get_gcmake_dependencies()
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

    drop(mut_graph_borrow);

    return graph;
  }

  fn load_predefined_dependency(
    target_id_counter: &mut i32,
    graph_id_counter: &mut usize,
    predef_dep: &Rc<FinalPredefinedDependencyConfig>
  ) -> Rc<RefCell<DependencyGraph<'a>>> {
    let graph: Rc<RefCell<DependencyGraph>> = Rc::new(RefCell::new(DependencyGraph {
      graph_id: *graph_id_counter,
      project_group_id: ProjectGroupId(*graph_id_counter),
      toplevel: Weak::new(),
      parent: None,
      current_graph_ref: Weak::new(),
      _project_wrapper: ProjectWrapper::PredefinedDependency(Rc::clone(predef_dep)),
      pre_build_wrapper: None,
      gcmake_deps: HashMap::new(),
      predefined_deps: HashMap::new(),
      subprojects: HashMap::new(),
      test_projects: HashMap::new(),
      targets: RefCell::new(HashMap::new())
    }));

    *graph_id_counter += 1;

    let mut mut_graph = graph.as_ref().borrow_mut();

    mut_graph.toplevel = Rc::downgrade(&graph);
    mut_graph.current_graph_ref = Rc::downgrade(&graph);

    let targets: HashMap<String, Rc<RefCell<TargetNode>>> = predef_dep.get_target_config_map()
      .iter()
      .map(|(target_name, target_config)| {
        
        (
          target_name.clone(),
          Rc::new(RefCell::new(TargetNode::new(
            target_id_counter,
            target_name.clone(),
            target_config.system_spec_info.clone(),
            target_config.cmakelists_name.clone(),
            String::from("RECEIVER LIB NAME NOT USED FOR PREDEFINED LIBRARIES"),
            predef_dep.get_cmake_namespaced_target_name(target_name).unwrap(),
            predef_dep.get_yaml_namespaced_target_name(target_name).unwrap(),
            predef_dep.should_install_if_linked_to_output_library(),
            Rc::downgrade(&graph),
            ContainedItem::PredefinedLibrary {
              target_name: target_name.clone(),
              external_requirements: target_config.external_requirements_set.clone()
            },
            LinkAccessMode::UserFacing,
            true
          )))
        )
      })
      .collect();

    for (target_name, target_rc) in &targets {
      let single_target: &mut TargetNode = &mut target_rc.as_ref().borrow_mut();

      let final_target_config: &FinalTargetConfig = predef_dep.get_target_config_map()
        .get(target_name)
        .unwrap();

      // NOTE: External requirements can't be loaded here because we can't guarantee that the needed
      // dependencies have been loaded yet. We'll load the external requirements when ensuring
      // links are correct, before cycle checking. That way we know that all dependencies the project
      // uses have already been loaded.
      for requirement_spec in &final_target_config.requirements_set {
        match requirement_spec {
          FinalRequirementSpecifier::Single(requirement_name) => {
            assert!(
              targets.contains_key(requirement_name),
              "Required interdependent target names should always have a match in a predefined dependency project, since those are checked when the project itself is loaded."
            );

            let requirement_target: &Rc<RefCell<TargetNode>> = targets.get(requirement_name).unwrap();

            single_target.insert_link(Link::new(
              requirement_name.to_string(),
              single_target.system_specifier_info.clone(),
              Rc::downgrade(requirement_target),
              // Not sure if this will make a difference yet, since we probably won't be checking links
              // to predefined dependencies anyways.
              LinkMode::Public
            ));
          },
          FinalRequirementSpecifier::OneOf(req_names) => {
            for requirement_name in req_names {
              assert!(
                targets.contains_key(requirement_name),
                "Required interdependent target names should always have a match in a predefined dependency project, since those are checked when the project itself is loaded."
              );
            }

            single_target.add_complex_requirement(NonOwningComplexTargetRequirement::OneOf(
              req_names.iter()
                .map(|lib_name| 
                  MaybePresentNonOwningTarget::Populated(
                    Rc::downgrade(targets.get(lib_name).unwrap())
                  )
                )
                .collect()
            ))
          },
          FinalRequirementSpecifier::MutuallyExclusive(exclusion_name) => {
            single_target.add_complex_requirement(NonOwningComplexTargetRequirement::ExclusiveFrom(
              Rc::downgrade(targets.get(exclusion_name).unwrap())
            ))
          }
        }
      }
    }

    mut_graph.targets = RefCell::new(targets);

    drop(mut_graph);

    return graph;
  }

  fn load_gcmake_dependency(
    target_id_counter: &mut i32,
    graph_id_counter: &mut usize,
    gcmake_dep: &'a Rc<FinalGCMakeDependency>
  ) -> Rc<RefCell<DependencyGraph<'a>>> {
    return match gcmake_dep.project_status() {
      GCMakeDependencyStatus::Available(available_project) => {
        let resolved_project = Self::recurse_root_project(
          target_id_counter,
          graph_id_counter,
          available_project
        );

        resolved_project.as_ref().borrow_mut()._project_wrapper = ProjectWrapper::GCMakeDependencyRoot(Rc::clone(gcmake_dep));
        resolved_project
      },
      GCMakeDependencyStatus::NotDownloaded(_) => {
        let graph: Rc<RefCell<DependencyGraph>> = Rc::new(RefCell::new(DependencyGraph {
          graph_id: *graph_id_counter,
          project_group_id: ProjectGroupId(*graph_id_counter),
          toplevel: Weak::new(),
          parent: None,
          current_graph_ref: Weak::new(),
          _project_wrapper: ProjectWrapper::GCMakeDependencyRoot(Rc::clone(gcmake_dep)),
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
        }));

        let mut mut_graph = graph.as_ref().borrow_mut();

        *graph_id_counter += 1;

        mut_graph.toplevel = Rc::downgrade(&graph);
        mut_graph.current_graph_ref = Rc::downgrade(&graph);

        drop(mut_graph);

        graph
      }
    }
  }
}

struct DAGSubGraph<'a> {
  // Head nodes are not depended on by any other nodes. At least one of these is guaranteed
  // to exist in a graph which has no cycles.
  pub head_nodes: HashSet<RcRefcHashWrapper<TargetNode<'a>>>,
  pub _all_member_nodes: HashSet<RcRefcHashWrapper<TargetNode<'a>>>
}

pub struct OrderedTargetInfo<'a> {
  pub targets_in_build_order: Vec<RcRefcHashWrapper<TargetNode<'a>>>,
  pub project_order: Vec<RcRefcHashWrapper<DependencyGraph<'a>>>
}

impl<'a> OrderedTargetInfo<'a> {
  // Assumes the vec of targets is already correctly sorted.
  pub fn from_ordered(ordered_targets: Vec<RcRefcHashWrapper<TargetNode<'a>>>) -> Self {
    let mut project_indices: HashMap<RcRefcHashWrapper<DependencyGraph>, usize> = HashMap::new();

    for (target_index, target) in ordered_targets.iter().enumerate() {
      project_indices.entry(RcRefcHashWrapper(target.as_ref().borrow().container_project()))
        .and_modify(|index| *index = target_index)
        .or_insert(target_index);
    }

    let mut project_to_index_pairs: Vec<(RcRefcHashWrapper<DependencyGraph>, usize)> = project_indices
      .into_iter()
      .collect();
    
    project_to_index_pairs
      .sort_by_key(|(_, last_target_index)| last_target_index.clone());

    return Self {
      targets_in_build_order: ordered_targets,
      project_order: project_to_index_pairs
        .into_iter()
        .map(|(project, _)| project)
        .collect()
    }
  }

  pub fn targets_in_link_order(&self) -> impl Iterator<Item=&RcRefcHashWrapper<TargetNode<'a>>> {
    return self.targets_in_build_order.iter().rev();
  }

  pub fn all_targets_with_root_project_id(&self, root_project_id: ProjectId) -> Vec<RcRefcHashWrapper<TargetNode<'a>>> {
    return self.targets_in_build_order
      .iter()
      .filter(|target|
        target.as_ref().borrow()
        .container_project().as_ref().borrow()
        .root_project_id() == root_project_id
      )
      .map(|wrapped_target| wrapped_target.clone())
      .collect();
  }

  // Excludes pre-build targets
  pub fn regular_targets_with_root_project_id(&self, root_project_id: ProjectId) -> Vec<RcRefcHashWrapper<TargetNode<'a>>> {
    return self.targets_in_build_order
      .iter()
      .filter(|target| {
        let borrowed_target = target.as_ref().borrow();
        let ids_match: bool = borrowed_target.container_project().as_ref().borrow().root_project_id() == root_project_id;

        ids_match && borrowed_target.is_regular_node()
      })
      .map(|wrapped_target| wrapped_target.clone())
      .collect();
  }

  pub fn targets_with_project_id(&self, project_id: ProjectId) -> Vec<RcRefcHashWrapper<TargetNode<'a>>> {
    return self.targets_in_build_order
      .iter()
      .filter(|target| target.as_ref().borrow().container_project_id() == project_id)
      .map(|wrapped_target| wrapped_target.clone())
      .collect();
  }

  pub fn regular_targets_with_project_id(&self, project_id: ProjectId) -> Vec<RcRefcHashWrapper<TargetNode<'a>>> {
    return self.targets_with_project_id(project_id)
      .into_iter()
      .filter(|target| target.as_ref().borrow().is_regular_node())
      .collect();
  }

  pub fn regular_dependencies_by_mode(&self, dependent_target: &Rc<RefCell<TargetNode<'a>>>) -> HashMap<LinkMode, Vec<RcRefcHashWrapper<TargetNode<'a>>>> {
    let dependencies: HashMap<RcRefcHashWrapper<TargetNode>, LinkMode> = dependent_target.as_ref().borrow().depends_on
      .iter()
      .map(|(_, link)| {
        (
          RcRefcHashWrapper(Weak::upgrade(&link.target).unwrap()),
          link.link_mode.clone()
        )
      })
      .filter(|(node, _)| node.as_ref().borrow().is_regular_node())
      .collect();

    let mut link_map: HashMap<LinkMode, Vec<RcRefcHashWrapper<TargetNode>>> = HashMap::new();
    
    for some_target in &self.targets_in_build_order {
      if let Some((dependency_target, link_mode)) = dependencies.get_key_value(some_target) {
        link_map.entry(link_mode.clone())
          .and_modify(|dep_vec| dep_vec.push(dependency_target.clone()))
          .or_insert(vec![dependency_target.clone()]);
      }
    }
    
    return link_map;
  }
}

type DepMap<'a> = HashMap<
  RcRefcHashWrapper<TargetNode<'a>>,
  HashSet<RcRefcHashWrapper<TargetNode<'a>>>
>;

type InverseDepMap<'a> = HashMap<
  RcRefcHashWrapper<TargetNode<'a>>,
  HashSet<RcRefcHashWrapper<TargetNode<'a>>>
>;

type GroupIdMappedTargets<'a> = HashMap<ProjectGroupId, HashSet<RcRefcHashWrapper<TargetNode<'a>>>>;

fn make_dep_map<'a>(
  all_used_targets: &HashSet<RcRefcHashWrapper<TargetNode<'a>>>
) -> DepMap<'a> {
  let mut dep_map: DepMap = DepMap::new();

  let mut unvisited_targets: Vec<RcRefcHashWrapper<TargetNode>> = all_used_targets
    .iter()
    .map(|wrapped_target| wrapped_target.clone())
    .collect();

  let mut visited_targets: HashSet<RcRefcHashWrapper<TargetNode>> = HashSet::new();

  while let Some(target_node) = unvisited_targets.pop() {
    visited_targets.insert(target_node.clone());
    let entry = dep_map.entry(target_node.clone())
      .or_insert(HashSet::new());

    for (_, dependency_link) in &target_node.as_ref().borrow().depends_on {
      let dependency_target: Rc<RefCell<TargetNode>> = Weak::upgrade(&dependency_link.target).unwrap();
      entry.insert(RcRefcHashWrapper(dependency_target));
    }

    for complex_requirement in &target_node.as_ref().borrow().complex_requirements {
      match complex_requirement {
        NonOwningComplexTargetRequirement::ExclusiveFrom(_) => {
          // Not added here. This is checked when resolving predefined dependency requirements, meaning
          // that if the code reaches this point, we can assume the requirement has been excluded from
          // the target and shouldn't appear in the dep list at this point (for this target).
        }
        NonOwningComplexTargetRequirement::OneOf(maybe_dependency_list) => {
          // Specify each optional node as a dependency as well. This done so that requirement targets
          // show up in the inverse dep map, which is important for correct ordering.
          for maybe_populated_dependency in maybe_dependency_list {
            match maybe_populated_dependency {
              MaybePresentNonOwningTarget::NotImported { .. } => (),
              MaybePresentNonOwningTarget::Populated(maybe_used_dependency) => {
                let wrapped_maybe_dep: RcRefcHashWrapper<TargetNode> = RcRefcHashWrapper(Weak::upgrade(maybe_used_dependency).unwrap());
                
                if !visited_targets.contains(&wrapped_maybe_dep) && !unvisited_targets.contains(&wrapped_maybe_dep) {
                  unvisited_targets.push(wrapped_maybe_dep.clone());
                }

                entry.insert(wrapped_maybe_dep);
              }
            }
          }
        }
      }
    }
  }

  assert!(
    dep_map.len() >= all_used_targets.len(),
    "A dep map should contain one entry for every used target."
  );

  return dep_map;
}

fn make_inverse_dep_map<'a>(
  all_used_targets: &HashSet<RcRefcHashWrapper<TargetNode<'a>>>,
  dep_map: &DepMap<'a>
) -> InverseDepMap<'a> {
  // target -> nodes which depend on target
  let mut inverse_map: InverseDepMap = InverseDepMap::new();

  for (dependent_target, dependency_set) in dep_map {
    for dependency in dependency_set {
      inverse_map.entry(dependency.clone())
        .and_modify(|dependent_target_set| {
          dependent_target_set.insert(dependent_target.clone());
        })
        .or_insert(HashSet::from([dependent_target.clone()]));
    }
  }

  let map_key_set: HashSet<RcRefcHashWrapper<TargetNode>> = inverse_map.keys()
    .map(|key| key.clone())
    .collect();

  for unused_key in all_used_targets.difference(&map_key_set) {
    inverse_map.insert(unused_key.clone(), HashSet::new());
  }

  assert!(
    inverse_map.len() >= all_used_targets.len(),
    "The inverse dep map should have an entry for every used target."
  );

  return inverse_map;
}

fn nodes_mapped_by_project_group_id<'a>(
  all_used_targets: &HashSet<RcRefcHashWrapper<TargetNode<'a>>>
) -> GroupIdMappedTargets<'a> {
  let mut the_map: GroupIdMappedTargets = GroupIdMappedTargets::new();

  for node in all_used_targets {
    let owned_node_ref = node.clone();
    let node_ref_clone = node.clone();

    the_map.entry(node.as_ref().borrow().container_project_group_id())
      .and_modify(|node_set| {
        node_set.insert(node_ref_clone);
      })
      .or_insert(HashSet::from([owned_node_ref]));
  }

  return the_map;
}

// NOTE: When sorting, use 'dep_map' and 'inverse_dep_map' to resolve node dependencies instead of
// node.depends_on. The dep maps may contain nodes which are optionally required by targets, but are
// not found in node.depends_on.
fn sorted_target_info<'a>(all_used_targets: &HashSet<RcRefcHashWrapper<TargetNode<'a>>>) -> OrderedTargetInfo<'a> {
  let dep_map: DepMap = make_dep_map(&all_used_targets);
  let inverse_dep_map: InverseDepMap = make_inverse_dep_map(&all_used_targets, &dep_map);
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

  return OrderedTargetInfo::from_ordered(
    sorted_node_list
      .into_iter()
      .filter(|node| all_used_targets.contains(node))
      .collect()
  );
}

fn recurse_sort_select<'a>(
  node_checking: &RcRefcHashWrapper<TargetNode<'a>>,
  sorted_node_list: &mut Vec<RcRefcHashWrapper<TargetNode<'a>>>,
  visited: &mut HashSet<RcRefcHashWrapper<TargetNode<'a>>>,
  dep_map: &DepMap<'a>,
  inverse_dep_map: &InverseDepMap<'a>,
  used_nodes_by_project: &GroupIdMappedTargets<'a>
) {
  let project_head_nodes_iter: Vec<&RcRefcHashWrapper<TargetNode>> = used_nodes_by_project.get(&node_checking.as_ref().borrow().container_project_group_id())
    .unwrap()
    .iter()
    .filter(|node| {
      let is_depended_on_by_other_node_in_project_group: bool = inverse_dep_map.get(node)
        .unwrap()
        .iter()
        .any(|dependent|
          dependent.as_ref().borrow().container_project_group_id() == node.as_ref().borrow().container_project_group_id()
        );

      !is_depended_on_by_other_node_in_project_group && !visited.contains(node)
    })
    .collect();

  // for &node in &project_head_nodes_iter {
  //   visited.insert(node.clone());
  // }

  for node in project_head_nodes_iter {
    traverse_sort_nodes(
      node,
      sorted_node_list,
      visited,
      dep_map,
      inverse_dep_map,
      used_nodes_by_project
    );
  }
}

fn traverse_sort_nodes<'a>(
  // 'node' is guaranteed to be an uppermost unvisited node in the project.
  node: &RcRefcHashWrapper<TargetNode<'a>>,
  sorted_node_list: &mut Vec<RcRefcHashWrapper<TargetNode<'a>>>,
  visited: &mut HashSet<RcRefcHashWrapper<TargetNode<'a>>>,
  dep_map: &DepMap<'a>,
  inverse_dep_map: &InverseDepMap<'a>,
  nodes_by_project: &GroupIdMappedTargets<'a>
) {
  visited.insert(node.clone());

  for dependency_node in dep_map.get(node).unwrap() {
    if visited.contains(dependency_node) {
      continue;
    }

    if dependency_node.as_ref().borrow().container_project_group_id() != node.as_ref().borrow().container_project_group_id() {
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
        dependency_node,
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

fn find_all_dag_subgraphs<'a>(
  all_used_targets: &HashSet<RcRefcHashWrapper<TargetNode<'a>>>,
  inverse_dep_map: &InverseDepMap<'a>
) -> Vec<DAGSubGraph<'a>> {
  let mut all_visited: HashSet<RcRefcHashWrapper<TargetNode>> = HashSet::new();
  let mut dag_list: Vec<DAGSubGraph> = Vec::new();

  for node in all_used_targets {
    if !all_visited.contains(node) {
      let mut local_visited: HashSet<RcRefcHashWrapper<TargetNode>> = HashSet::new();
      let mut stack: Vec<RcRefcHashWrapper<TargetNode>> = vec![node.clone()];

      while let Some(node_checking) = stack.pop() {
        local_visited.insert(node_checking.clone());

        for (_, dep_link) in &node_checking.as_ref().borrow().depends_on {
          let wrapped_link_target: RcRefcHashWrapper<TargetNode> = 
            RcRefcHashWrapper(Weak::upgrade(&dep_link.target).unwrap());

          if !local_visited.contains(&wrapped_link_target) {
            stack.push(wrapped_link_target);
          }
        }

        assert!(
          inverse_dep_map.get(&node_checking).is_some(),
          "The inverse dep map should always contain an entry for a used target."
        );

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
        _all_member_nodes: local_visited
      });
    }
  }

  assert!(
    all_visited.len() == all_used_targets.len(),
    "Number of visited nodes should equal the number of used nodes."
  );

  return dag_list;
}