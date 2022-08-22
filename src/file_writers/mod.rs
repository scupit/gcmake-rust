mod cmake_writer;

use std::{io::{self, ErrorKind}, rc::Rc, cell::{RefCell, Ref}};
use crate::project_info::{final_project_data::{UseableFinalProjectDataGroup}, dependency_graph_mod::dependency_graph::{DependencyGraphInfoWrapper, DependencyGraph, GraphLoadFailureReason, TargetNode, OwningComplexTargetRequirement}, SystemSpecifierWrapper};

pub struct ProjectWriteConfiguration {
  name: String,
  config_func: fn(&DependencyGraphInfoWrapper) -> io::Result<()>,
}

fn borrow_target(target_node: &Rc<RefCell<TargetNode>>) -> Ref<TargetNode> {
  return target_node.as_ref().borrow();
}

fn borrow_project(project: &Rc<RefCell<DependencyGraph>>) -> Ref<DependencyGraph> {
  return project.as_ref().borrow();
}

pub fn write_configurations<'a, FBefore, FAfter>(
  project_data: &UseableFinalProjectDataGroup,
  before_write: FBefore,
  after_write: FAfter
) -> io::Result<()>
  where
    FBefore: Fn(&str),
    FAfter: Fn((&str, io::Result<()>))
{
  match DependencyGraph::new_info_from_root(&project_data.root_project) {
    Ok(dep_graph_info) => {
      let project_configurers = [
        ProjectWriteConfiguration {
          name: String::from("CMake"),
          config_func: cmake_writer::configure_cmake
        }
      ];

      for config in project_configurers {
        let config_name_str = config.name.as_str();
        before_write(config_name_str);

        let write_result = (config.config_func)(&dep_graph_info);
        after_write((config_name_str, write_result));
      }
    },
    // TODO: Improve these error messages. Especially figure out how to add a "project stack trace" to
    // better specify where targets come from.
    Err(graph_build_error) => match graph_build_error {
      GraphLoadFailureReason::LinkPointsToInvalidOrNonexistentProject { target, project, link_spec } => {
        let borrowed_target  = borrow_target(&target);

        return wrap_error_msg(format!(
          "Link specifier '{}' from target '{}' in project '{}' points to an invalid or nonexistent project.",
          link_spec.get_spec_string(),
          borrowed_target.get_name(),
          borrow_project(&project).project_name_for_error_messages()
        ));
      },
      GraphLoadFailureReason::LinkNestedNamespaceInOtherProjectContext { target, project, link_spec } => {
        let borrowed_target = borrow_target(&target);

        return wrap_error_msg(format!(
          "Link specifier '{}' from target '{}' in project '{}' tries to access nested namespaces in a dependency project, which is forbidden.",
          link_spec.get_spec_string(),
          borrowed_target.get_name(),
          borrow_project(&project).project_name_for_error_messages()
        ));
      },
      GraphLoadFailureReason::LinkTargetNotFound { target, target_container_project, looking_in_project, link_spec, name_searching } => {
        let borrowed_target = borrow_target(&target);

        return wrap_error_msg(format!(
          "Unable to find target '{}' in project '{}'.\n\tUsing link specifier '{}' from target '{}' in project '{}'.",
          name_searching,
          borrow_project(&looking_in_project).project_name_for_error_messages(),
          link_spec.get_spec_string(),
          borrowed_target.get_name(),
          borrow_project(&target_container_project).project_name_for_error_messages()
        ));
      },
      GraphLoadFailureReason::DependencyCycle(mut cycle_vec) => {
        // Cyclic dependency
        //    firstproject::target
        //    -> someproject::target
        //    -> otherproject::target
        //    -> firstproject::target
        //    -> ...
        cycle_vec.push(cycle_vec.get(0).unwrap().clone());
        let cycle_str: String = cycle_vec
          .iter()
          .map(|cycle| format!(
            "{}::{}",
            borrow_project(&cycle.project).project_name_for_error_messages(),
            borrow_target(&cycle.target).get_name()
          ))
          .collect::<Vec<String>>()
          .join("\n-> ");

        return wrap_error_msg(format!(
          "Dependency cycle detected:\n{}\n-> ...",
          cycle_str
        ));
      },
      GraphLoadFailureReason::LinkedToSelf { project, target_name } => {
        return wrap_error_msg(format!(
          "Target '{}' in project '{}' tries to link to itself.",
          target_name,
          borrow_project(&project).project_name_for_error_messages()
        ));
      },
      GraphLoadFailureReason::AccessNotAllowed {
        link_spec,
        ref link_spec_container_target,
        link_spec_container_project,
        dependency_project: target_project,
        dependency: ref target,
        given_access_mode: _,
        needed_access_mode: _
      } => {
        return wrap_error_msg(format!(
          "Link specifier '{}' from target '{}' in project '{}' points points to a target '{}' which exists in project '{}', but cannot be linked to. The target is either an executable or an internally created target.",
          link_spec.get_spec_string(),
          borrow_target(link_spec_container_target).get_name(),
          borrow_project(&link_spec_container_project).project_name_for_error_messages(),
          borrow_target(target).get_name(),
          borrow_project(&target_project).project_name_for_error_messages()
        ));
      },
      GraphLoadFailureReason::WrongUserGivenPredefLinkMode {
        current_link_mode,
        needed_link_mode,
        ref target,
        target_project,
        ref dependency,
        dependency_project
      } => {
        return wrap_error_msg(format!(
          "In target '{}' in project '{}':\n A {} link is specified to target '{}' in project '{}', however it should be linked as {}.",
          borrow_target(target).get_name(),
          borrow_project(&target_project).project_name_for_error_messages(),
          current_link_mode.to_str(),
          borrow_target(dependency).get_name(),
          borrow_project(&dependency_project).project_name_for_error_messages(),
          needed_link_mode.to_str()
        ));
      },
      GraphLoadFailureReason::LinkedInMultipleCategories {
        current_link_mode,
        attempted_link_mode,
        link_receiver_project,
        link_receiver_name,
        link_giver_project,
        link_giver_name
      } => {
        return wrap_error_msg(format!(
          "Target '{}' in project '{}' specifies both a {} and {} link to target '{}' in project '{}'. Links to a target should only be specified in one category.",
          link_receiver_name,
          borrow_project(&link_receiver_project).project_name_for_error_messages(),
          current_link_mode.to_str(),
          attempted_link_mode.to_str(),
          link_giver_name,
          borrow_project(&link_giver_project).project_name_for_error_messages()
        ));
      },
      GraphLoadFailureReason::ComplexTargetRequirementNotSatisfied {
        ref target,
        ref target_project,
        ref dependency,
        ref dependency_project,
        ref failed_requirement
      } => {
        let base_message: String = format!(
          "Target '{}' in project '{}' failed to satisfy a requirement of its linked dependency target '{}':\n\t",
          borrow_target(target).get_name(),
          borrow_project(target_project).project_name_for_error_messages(),
          borrow_target(dependency).get_yaml_namespaced_target_name()
        );

        let requirement_specific_message: String = match failed_requirement {
          OwningComplexTargetRequirement::OneOf(target_list) => {
            let target_list_str: String = target_list
              .iter()
              .map(|needed_target|
                borrow_target(needed_target)
                  .get_yaml_namespaced_target_name()
                  .to_string()
              )
              .collect::<Vec<String>>()
              .join(", ");

            format!(
              "The target must link to one of: ({}) from '{}'",
              target_list_str,
              borrow_project(dependency_project).project_name_for_error_messages()
            )
          },
          OwningComplexTargetRequirement::ExclusiveFrom(excluded_target) => {
            format!(
              "The target links to both '{}' and '{}' from '{}', but '{}' and '{}' are mutually exclusive. You can link to one or the other, but not both at once.",
              borrow_target(dependency).get_name(),
              borrow_target(excluded_target).get_name(),
              borrow_project(dependency_project).project_name_for_error_messages(),
              borrow_target(dependency).get_yaml_namespaced_target_name(),
              borrow_target(excluded_target).get_yaml_namespaced_target_name()
            )
          }
        };

        return wrap_error_msg(format!("{}{}", base_message, requirement_specific_message));
      },
      GraphLoadFailureReason::DuplicateLinkTarget {
        ref link_spec_container_target,
        link_spec_container_project: _,
        ref dependency_project,
        ref dependency
      } => {
        return wrap_error_msg(format!(
          "Target '{}' from project '{}' specifies multiple links to '{}'. Please remove any duplicate links.",
          borrow_target(link_spec_container_target).get_name(),
          borrow_project(dependency_project).project_name_for_error_messages(),
          borrow_target(dependency).get_yaml_namespaced_target_name(),
        ));
      },
      GraphLoadFailureReason::LinkSystemSubsetMismatch {
        ref link_spec,
        link_system_spec_info: link_target_spec_info,
        ref link_spec_container_target,
        ref link_spec_container_project,
        dependency_project: _,
        ref dependency,
        ref transitively_required_by
      } => {
        let borrowed_dependency = borrow_target(dependency);


        let transitive_target_str: String = match transitively_required_by {
          None => String::from(""),
          Some(middle_dep_node) => {
            format!(
              " (as a transitive dependency required by {})",
              borrow_target(middle_dep_node).get_yaml_namespaced_target_name()
            )
          }
        };

        let link_spec_str: String = match link_spec {
          None => String::from(""),
          Some(used_link_spec) => {
            format!(
              " using link spec '{}'",
              used_link_spec.get_spec_string()
            )
          }
        };

        return wrap_error_msg(format!(
          "Target '{}' in project '{}' links to dependency '{}' on {}{}{}, but '{}' is only supported on {}.",
          borrow_target(link_spec_container_target).get_name(),
          borrow_project(link_spec_container_project).project_name_for_error_messages(),
          borrowed_dependency.get_yaml_namespaced_target_name(),
          systems_string(&link_target_spec_info),
          link_spec_str,
          transitive_target_str,
          borrowed_dependency.get_yaml_namespaced_target_name(),
          systems_string(borrowed_dependency.get_system_spec_info())
        ));
      },
      GraphLoadFailureReason::LinkSystemRequirementImpossible {
        ref target,
        ref target_container_project,
        ref link_system_spec_info,
        ref dependency
      } => {
        return wrap_error_msg(format!(
          "Target '{}' on {} in project '{}' links on {} to dependency '{}', which is available on {}. This association is impossible.",
          borrow_target(target).get_name(),
          systems_string(borrow_target(target).get_system_spec_info()),
          borrow_project(target_container_project).project_name_for_error_messages(),
          systems_string(link_system_spec_info),
          borrow_target(dependency).get_yaml_namespaced_target_name(),
          systems_string(borrow_target(dependency).get_system_spec_info())
        ));
      }
    }
  }

  return Ok(());
}

fn wrap_error_msg(msg: impl AsRef<str>) -> io::Result<()> {
  return Err(io::Error::new(
    ErrorKind::Other,
    format!("Error: {}", msg.as_ref().to_string())
  ));
}

fn systems_string(system_spec_info: &SystemSpecifierWrapper) -> String {
  return if system_spec_info.includes_all() {
    String::from("all systems")
  }
  else {
    format!(
      "a subset of {}",
      system_spec_info.unwrap_specific_ref().to_string()
    )
  }
}
