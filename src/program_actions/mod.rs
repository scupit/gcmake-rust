mod create_project;
mod code_file_creator;
mod manage_dependencies;

pub use create_project::*;
pub use code_file_creator::*;
pub use manage_dependencies::*;
use std::{io, path::PathBuf, fs, cell::RefCell, rc::Rc, borrow::Borrow};

use crate::{cli_config::{clap_cli_config::{UseFilesCommand, CreateFilesCommand, UpdateDependencyConfigsCommand, TargetInfoCommand}, CLIProjectGenerationInfo, CLIProjectTypeGenerating}, common::prompt::prompt_until_boolean, logger::exit_error_log, project_info::{raw_data_in::dependencies::internal_dep_config::AllRawPredefinedDependencies, final_project_data::{UseableFinalProjectDataGroup, ProjectLoadFailureReason, FinalProjectData, ProjectConstructorConfig}, path_manipulation::absolute_path, dep_graph_loader::load_graph, dependency_graph_mod::dependency_graph::{DependencyGraphInfoWrapper, DependencyGraph, TargetNode, BasicTargetSearchResult, ContainedItem}, LinkSpecifier}, file_writers::write_configurations, project_generator::GeneralNewProjectInfo};

fn parse_project_info(
  project_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  just_created_project_at: Option<String>
) -> Result<UseableFinalProjectDataGroup, ProjectLoadFailureReason> {
  FinalProjectData::new(
    project_root_dir,
    dep_config,
    ProjectConstructorConfig {
      just_created_library_project_at: just_created_project_at,
    }
  )
    .map_err(|failure_reason| failure_reason.map_message(|err_message|{
      format!(
        "When loading project using path '{}':\n\n{}",
        absolute_path(project_root_dir).unwrap().to_str().unwrap(),
        err_message
      )
    }))
}

fn get_project_info_or_exit(
  project_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  just_created_project_at: Option<String>
) -> UseableFinalProjectDataGroup {
  match parse_project_info(project_root_dir, dep_config, just_created_project_at) {
    Ok(project_group) => project_group,
    Err(failure_reason) => exit_error_log(failure_reason.extract_message())
  }
}

struct RootAndOperatingGraphs<'a> {
  root_graph: Rc<RefCell<DependencyGraph<'a>>>,
  operating_on: Option<Rc<RefCell<DependencyGraph<'a>>>>
}

fn get_project_graph_or_exit<'a>(
  project_group: &'a UseableFinalProjectDataGroup
) -> RootAndOperatingGraphs<'a> {
  match load_graph(&project_group) {
    Ok(graph_info) => {
      return RootAndOperatingGraphs {
        operating_on: project_group.operating_on
          .as_ref()
          .map(|operatng_on_project|
            graph_info.root_dep_graph
              .as_ref()
              .borrow()
              .find_using_project_data(&operatng_on_project)
              .unwrap()
          ),
        root_graph: graph_info.root_dep_graph
      }
    },
    Err(err_msg) => exit_error_log(err_msg)
  }
}

pub fn print_target_info(
  command: &TargetInfoCommand,
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  just_created_project_at: Option<String>
) {
  if command.selectors.is_empty() {
    // TODO: When no selector is provided, just print the info for all project targets.
    exit_error_log("Must provide at least one target selector. Example: self::main_exe");
  }

  let project_group: UseableFinalProjectDataGroup =
    get_project_info_or_exit(given_root_dir, dep_config, just_created_project_at);
 
  let graph_info: RootAndOperatingGraphs = get_project_graph_or_exit(&project_group);

  match &graph_info.operating_on {
    None => exit_error_log(format!(
      "Tried to print target data when operating on project at '{}', however the project could not be found.",
      given_root_dir
    )),
    Some(operating_on) => {
      let result_list: Vec<Vec<BasicTargetSearchResult>> = command.selectors
        .iter()
        .map(|selector| {
          Ok(operating_on.as_ref().borrow().find_targets_using_link_spec(
            false,
            &LinkSpecifier::parse_with_full_permissions(selector)?
          )?)
        })
        .collect::<Result<_, String>>()
        .unwrap_or_else(|err_msg| exit_error_log(err_msg));

      for list_from_selector in result_list {
        for search_result in list_from_selector {
          match search_result.target {
            None => println!("{} not found", search_result.searched_with),
            Some(target_rc) => {
              let target = target_rc.as_ref().borrow();
              println!("\n========== {} ==========", target.get_yaml_namespaced_target_name());

              if command.export_header {

                match target.get_contained_item() {
                  ContainedItem::CompiledOutput(output) if output.is_compiled_library_type() => {
                    println!(
                      "Export header:\n\t\"{}/{}_export.h\"",
                      target.container_project().as_ref().borrow().project_wrapper().clone().unwrap_normal_project().get_full_include_prefix(),
                      target.get_name()
                    )
                  },
                  _ => println!("No export header")
                }
              }
            }
          }
        }
      }
    }
  }
}

pub fn copy_default_file(
  command: &UseFilesCommand,
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  just_created_project_at: Option<String>
) -> io::Result<()> {
  let file_name_str: &str = command.file.to_file_name();
  let project_info: UseableFinalProjectDataGroup =
    get_project_info_or_exit(given_root_dir, dep_config, just_created_project_at);

  let mut file_path: PathBuf = gcmake_config_root_dir();
  file_path.push(file_name_str);

  let mut project_root_file_path: PathBuf = PathBuf::from(project_info.root_project.get_absolute_project_root());
  project_root_file_path.push(file_name_str);

  if file_path.is_symlink() {
    println!(
      "{} is a symlink. Resolving...",
      file_path.to_str().unwrap()
    );

    file_path = fs::read_link(&file_path)?;

    println!(
      "The {} symlink points to '{}'. Using that path instead.\n",
      file_name_str,
      file_path.to_str().unwrap()
    );
  }

  if file_path.is_file() {
    if project_root_file_path.exists() {
      let prompt: String = format!(
        "A {} already exists in the root project. Overwrite it? [y or n]: ",
        file_name_str
      );

      if !prompt_until_boolean(&prompt)? {
        println!("File copy canceled.");
        return Ok(())
      }
      else {
        println!();
      }
    }

    return match fs::copy(&file_path, &project_root_file_path) {
      Ok(_) => {
        println!(
          "{} copied successfully.",
          file_name_str
        );
        Ok(())
      },
      Err(err) => Err(err)
    }
  }
  else {
    println!(
      "Could not copy \"{}\" because the file doesn't exist.",
      file_path.to_str().unwrap()
    );
    return Ok(())
  }
}

pub fn do_generate_project_configs(
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  just_created_project_at: Option<String>
) {
  let project_data_group: UseableFinalProjectDataGroup =
    get_project_info_or_exit(&given_root_dir, &dep_config, just_created_project_at);

  match load_graph(&project_data_group) {
    Err(err_msg) => exit_error_log(err_msg.to_string()),
    Ok(root_graph_info) => {
      let config_write_result: io::Result<()> = write_configurations(
        &root_graph_info,
        |config_name| println!("\nBeginning {} configuration step...", config_name),
        |(config_name, config_result)| match config_result {
          Ok(_) => println!("{} configuration written successfully!", config_name),
          Err(err) => {
            println!("Writing {} configuration failed with error:", config_name);
            println!("{:?}", err)
          }
        }
      ); 
      
      if let Err(err) = config_write_result {
        exit_error_log(err.to_string());
      }
    }
  }
  // print_project_info(project_data_group);
}

pub fn do_new_files_subcommand(
  command: CreateFilesCommand,
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  just_created_project_at: Option<String>
) {
  let project_data_group: UseableFinalProjectDataGroup =
    get_project_info_or_exit(&given_root_dir, &dep_config, just_created_project_at);

  // print_project_info(project_data_group);
  if let None = project_data_group.operating_on {
    exit_error_log("Tried to create files while not operating on a project. Make sure you are inside a project directory containing a cmake_data.yaml file.")
  }

  match handle_create_files(&project_data_group.operating_on.unwrap(), &command) {
    Ok(_) => println!("Files written successfully!"),
    Err(error_message) => exit_error_log(&error_message)
  }
}

pub fn do_new_project_subcommand(
  command: CLIProjectGenerationInfo,
  dep_config: &AllRawPredefinedDependencies,
  given_root_dir: &str,
  should_generate_cmakelists: &mut bool
) -> Option<GeneralNewProjectInfo> {
  let requires_project_operating_on: bool = match &command.project_type {
    CLIProjectTypeGenerating::RootProject => false,
    _ => true
  };

  match get_parent_project_for_new_project(&given_root_dir.clone(), dep_config, requires_project_operating_on) {
    Ok(maybe_project_info) => {
      let maybe_general_new_project_info = handle_create_project(
        command,
        &maybe_project_info.map(|it| it.operating_on).flatten(),
        given_root_dir,
        should_generate_cmakelists
      );

      return maybe_general_new_project_info;
    },
    Err(error_message) => exit_error_log(&error_message)
  }
}

pub fn do_dependency_config_update_subcommand(command: UpdateDependencyConfigsCommand) {
  match update_dependency_config_repo(&command.branch) {
    Ok(status) => match status {
      DepConfigUpdateResult::SubprocessError(git_subprocess_err_msg) => {
        exit_error_log(git_subprocess_err_msg);
      },
      DepConfigUpdateResult::NewlyDownloaded { branch, local_repo_location } => {
        println!(
          "Dependency config repo successfully downloaded to {}.",
          local_repo_location.to_str().unwrap()
        );
        println!("Checked out '{}' branch.", branch);
      },
      DepConfigUpdateResult::UpdatedBranch { branch: maybe_branch, .. } => {
        match maybe_branch {
          Some(checked_out_branch) => {
            println!(
              "Successfully checked out and updated dependency config repo '{}' branch.",
              checked_out_branch
            );
          },
          None => {
            println!("Successfully updated dependency config repo");
          }
        }
      }
    },
    Err(err) => exit_error_log(err.to_string())
  }
}

pub fn get_parent_project_for_new_project(
  current_root: &str,
  dep_config: &AllRawPredefinedDependencies,
  requires_all_yaml_files_present: bool
) -> Result<Option<UseableFinalProjectDataGroup>, String> {
  match parse_project_info(
    current_root,
    dep_config,
    None
  ) {
    Ok(project_data_group) => Ok(Some(project_data_group)),
    Err(failure_reason) => match failure_reason {
      ProjectLoadFailureReason::MissingYaml(error_message) => {
        if requires_all_yaml_files_present
          { Err(error_message) }
          else { Ok(None) }
      },
      ProjectLoadFailureReason::Other(error_message) => Err(error_message),
      ProjectLoadFailureReason::MissingRequiredTestFramework(error_message) => Err(error_message)
    }
  }
}