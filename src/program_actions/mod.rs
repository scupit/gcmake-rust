mod create_project;
mod code_file_creator;
mod manage_dependencies;
mod info_printers;
mod default_file_creator;

pub use create_project::*;
pub use code_file_creator::*;
pub use manage_dependencies::*;
use std::{io, path::PathBuf, fs, cell::RefCell, rc::Rc};

use crate::{cli_config::{clap_cli_config::{UseFilesCommand, CreateFilesCommand, UpdateDependencyConfigsCommand, TargetInfoCommand, ProjectInfoCommand, PredepInfoCommand, ToolInfoCommand, CreateDefaultFilesCommand, CreateDefaultFileOption}, CLIProjectGenerationInfo, CLIProjectTypeGenerating}, common::{prompt::prompt_until_boolean}, logger::exit_error_log, project_info::{raw_data_in::dependencies::internal_dep_config::AllRawPredefinedDependencies, final_project_data::{UseableFinalProjectDataGroup, ProjectLoadFailureReason, FinalProjectData, FinalProjectLoadContext}, path_manipulation::absolute_path, dep_graph_loader::load_graph, dependency_graph_mod::dependency_graph::{DependencyGraphInfoWrapper, DependencyGraph, TargetNode, BasicTargetSearchResult, DependencyGraphWarningMode, BasicProjectSearchResult}, LinkSpecifier, validators::{is_valid_target_name, is_valid_project_name}}, file_writers::write_configurations, project_generator::GeneralNewProjectInfo, program_actions::info_printers::{target_info_print_funcs::{print_target_header, print_export_header_include_path, print_target_type}, project_info_print_funcs::{print_project_header, print_project_include_prefix, print_immediate_subprojects, print_project_repo_url, print_project_can_cross_compile, print_project_supports_emscripten, print_project_output_list, print_project_dependencies}}};

use self::{info_printers::predef_dep_info_print_funcs::{print_predef_dep_header, print_predep_targets, print_predep_repo_url, print_predep_github_url, print_predep_can_cross_compile, print_predep_supports_emscripten, print_predep_supported_download_methods, print_predep_doc_link}, default_file_creator::{write_default_doxyfile, write_default_sphinx_files}};
use colored::*;

fn parse_project_info(
  project_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  project_load_context: FinalProjectLoadContext
) -> Result<UseableFinalProjectDataGroup, ProjectLoadFailureReason> {
  FinalProjectData::new(
    project_root_dir,
    dep_config,
    project_load_context
  )
    .map_err(|failure_reason| failure_reason.map_message(|err_message|{
      format!(
        "{} loading project using path '{}':\n\n{}",
        "Error".red(),
        absolute_path(project_root_dir).unwrap().to_str().unwrap(),
        err_message
      )
    }))
}

fn get_project_info_or_exit(
  project_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  project_load_context: FinalProjectLoadContext
) -> UseableFinalProjectDataGroup {
  match parse_project_info(project_root_dir, dep_config, project_load_context) {
    Ok(project_group) => project_group,
    Err(failure_reason) => exit_error_log(failure_reason.extract_message())
  }
}

struct RootAndOperatingGraphs<'a> {
  graph_info_wrapper: DependencyGraphInfoWrapper<'a>,
  project_root_graph: Rc<RefCell<DependencyGraph<'a>>>,
  operating_on: Option<Rc<RefCell<DependencyGraph<'a>>>>
}

fn get_project_graph_or_exit<'a>(
  project_group: &'a UseableFinalProjectDataGroup,
  warning_mode: DependencyGraphWarningMode
) -> RootAndOperatingGraphs<'a> {
  match load_graph(&project_group, warning_mode) {
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
        project_root_graph: Rc::clone(&graph_info.root_dep_graph),
        graph_info_wrapper: graph_info
      }
    },
    Err(err_msg) => exit_error_log(err_msg)
  }
}

pub fn print_target_info(
  command: &TargetInfoCommand,
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies
) {
  if command.selectors.is_empty() {
    // TODO: When no selector is provided, just print the info for all project targets.
    exit_error_log("Must provide at least one target selector. Example: self::main_exe");
  }

  let project_group: UseableFinalProjectDataGroup = get_project_info_or_exit(
    given_root_dir,
    dep_config,
    FinalProjectLoadContext::default()
  );
 
  let graph_info: RootAndOperatingGraphs = get_project_graph_or_exit(&project_group, DependencyGraphWarningMode::Off);

  assert!(
    graph_info.operating_on.is_some(),
    "When printing target data, there should always be an 'operating on' context project."
  );

  let operating_on = graph_info.operating_on.as_ref().unwrap();
  let result_list: Vec<Vec<BasicTargetSearchResult>> = command.selectors
    .iter()
    .map(|selector| {
      if is_valid_target_name(selector) {
        // When using only the target name as a selector, just search from the project root.
        // No need to restrict the search to subprojects here.
        let search_result = graph_info.project_root_graph
          .as_ref().borrow().find_targets_using_name_list(&vec![selector]);
        Ok(search_result)
      }
      else {
        let search_result = operating_on.as_ref().borrow().find_targets_using_link_spec(
          false,
          &LinkSpecifier::parse_with_full_permissions(selector, None)?
        )?;

        Ok(search_result)
      }
    })
    .collect::<Result<_, String>>()
    .unwrap_or_else(|err_msg| exit_error_log(err_msg));

  for list_from_selector in result_list {
    for search_result in list_from_selector {
      match search_result.target {
        None => {
          println!(
            "\nUnable to find '{}' in project [{}]",
            &search_result.searched_with,
            search_result.searched_project.as_ref().borrow().project_debug_name()
          );
        },
        Some(target_rc) => {
          // All data printing is done here.
          let target_node: &TargetNode = &target_rc.as_ref().borrow();
          print_target_header(target_node);

          if command.export_header {
            print_export_header_include_path(target_node);
          }

          if command.item_type {
            print_target_type(target_node);
          }
        }
      }
    }
  }
}

pub fn print_project_info(
  command: &ProjectInfoCommand,
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies
) {
  let selectors: Vec<String> = if command.selectors.is_empty()
    { vec![String::from("self")] }
    else { command.selectors.clone() };

  let project_group: UseableFinalProjectDataGroup = get_project_info_or_exit(
    given_root_dir,
    dep_config,
    FinalProjectLoadContext::default()
  );
 
  let graph_info: RootAndOperatingGraphs = get_project_graph_or_exit(&project_group, DependencyGraphWarningMode::Off);

  assert!(
    graph_info.operating_on.is_some(),
    "When printing project data, there should always be an 'operating on' context project."
  );

  let operating_on = graph_info.operating_on.as_ref().unwrap();

  let result_list: Vec<Vec<BasicProjectSearchResult>> = selectors
    .iter()
    .map(|selector| {
      if is_valid_project_name(selector) {
        // When using only the target name as a selector, just search from the project root.
        // No need to restrict the search to subprojects here.
        let search_result = operating_on
          .as_ref().borrow().find_projects_using_name_list(&vec![selector])?;
        Ok(search_result)
      }
      else {
        let search_result = operating_on.as_ref().borrow().find_projects_using_link_spec(
          false,
          &LinkSpecifier::parse_with_full_permissions(selector, None)?
        )?;

        Ok(search_result)
      }
    })
    .collect::<Result<_, String>>()
    .unwrap_or_else(|err_msg| exit_error_log(err_msg));

  for list_from_selector in result_list {
    for search_result in list_from_selector {
      match search_result.project {
        None => {
          println!(
            "\nProject '{}' not found in project [{}] as a normal project or dependency.",
            &search_result.project_name_searched,
            search_result.searched_project.as_ref().borrow().project_debug_name()
          );
        },
        Some(project_rc) => {
          // All data printing is done here.
          let project_graph: &DependencyGraph = &project_rc.as_ref().borrow();
          print_project_header(project_graph);

          if command.show_include_prefix {
            print_project_include_prefix(project_graph);
          }

          if command.list_targets {
            print_project_output_list(project_graph);
          }

          if command.list_dependencies {
            print_project_dependencies(project_graph);
          }

          if command.show_subprojects {
            print_immediate_subprojects(project_graph);
          }

          if command.show_repo_url {
            print_project_repo_url(project_graph);
          }

          if command.show_can_trivially_cross_compile {
            print_project_can_cross_compile(project_graph);
          }

          if command.show_supports_emscripten {
            print_project_supports_emscripten(project_graph);
          }
        }
      }
    }
  }
}

pub fn print_tool_info(command: ToolInfoCommand) {
  if command.show_config_dir {
    println!("{}", gcmake_config_root_dir().to_str().unwrap());
  }

  if command.show_dep_cache_dir {
    println!("{}", gcmake_dep_cache_dir().to_str().unwrap());
  }

  if command.show_dep_config_dir {
    println!("{}", gcmake_dep_config_dir().to_str().unwrap());
  }
}

pub fn print_predep_info(
  command: &PredepInfoCommand,
  dep_config: &AllRawPredefinedDependencies
) {
  if command.selectors.is_empty() {
    let mut dep_names: Vec<&String> = dep_config.keys().collect();
    dep_names.sort_by(|name1, name2| name1.to_lowercase().cmp(&name2.to_lowercase()));

    for dep_name in dep_names {
      println!("{}", dep_name);
    }
  }
  else {
    for dep_name in &command.selectors {
      print_predef_dep_header(dep_name);

      match dep_config.get(dep_name) {
        None => println!("Unable to find predefined dependency \"{}\"", dep_name),
        Some(raw_predep_config) => match raw_predep_config.dep_configs.get_common() {
          Err(err_msg) => {
            println!(
              "Error when trying to get target info for predefined dependency \"{}\": {}",
              dep_name,
              err_msg
            );
          },
          Ok(common_info) => {
            // Do info printing here.
            if command.show_targets {
              print_predep_targets(dep_name, common_info);
            }

            if command.show_repository_url {
              print_predep_repo_url(common_info);
            }

            if command.show_github_url {
              print_predep_github_url(common_info);
            }

            if command.show_can_trivially_cross_compile {
              print_predep_can_cross_compile(common_info);
            }

            if command.show_supported_download_methods {
              print_predep_supported_download_methods(common_info);
            }

            if command.show_doc_link {
              print_predep_doc_link(common_info);
            }

            if command.show_supports_emscripten {
              print_predep_supports_emscripten(common_info);
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
  dep_config: &AllRawPredefinedDependencies
) -> io::Result<()> {
  let file_name_str: &str = command.file.to_file_name();
  let project_info: UseableFinalProjectDataGroup = get_project_info_or_exit(
    given_root_dir,
    dep_config,
    FinalProjectLoadContext::default()
  );

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

pub fn generate_default_file(
  command: &CreateDefaultFilesCommand,
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies
) -> io::Result<()> {
  let about_to_generate_doxyfile: bool = match command.file {
    CreateDefaultFileOption::Doxyfile => true,
    CreateDefaultFileOption::SphinxConfig => true
  };

  let about_to_generate_sphinx_files: bool = match command.file {
    CreateDefaultFileOption::SphinxConfig => true,
    _ => false
  };

  let project_data_group: UseableFinalProjectDataGroup = get_project_info_or_exit(
    &given_root_dir,
    &dep_config,
    FinalProjectLoadContext {
      about_to_generate_doxyfile,
      about_to_generate_sphinx_files,
      just_created_library_project_at: None
    }
  );

  let doxyfile_name: &str = "Doxyfile.in";
  
  match &command.file {
    CreateDefaultFileOption::Doxyfile => {
      write_default_doxyfile(doxyfile_name, &project_data_group)?;
    },
    CreateDefaultFileOption::SphinxConfig => {
      write_default_doxyfile(doxyfile_name, &project_data_group)?;
      write_default_sphinx_files(
        "index.rst",
        "conf.py.in",
        &project_data_group
      )?;
    }
  }

  Ok(())
}

pub fn do_generate_project_configs(
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies
) {
  let project_data_group: UseableFinalProjectDataGroup = get_project_info_or_exit(
    &given_root_dir,
    &dep_config,
    FinalProjectLoadContext::default()
  );

  let RootAndOperatingGraphs { graph_info_wrapper, .. } = get_project_graph_or_exit(
    &project_data_group,
    DependencyGraphWarningMode::All
  );

  let config_write_result: io::Result<()> = write_configurations(
    &graph_info_wrapper,
    |config_name| println!("\nBeginning {} configuration step...", config_name.green()),
    |(config_name, config_result)| match config_result {
      Ok(_) => println!("{} configuration written successfully!", config_name.green()),
      Err(err) => {
        println!(
          "{}",
          format!(
            "Writing {} configuration failed with error:",
            config_name
          ).red()
        );
        println!("{:?}", err)
      }
    }
  ); 
  
  if let Err(err) = config_write_result {
    exit_error_log(err.to_string());
  }
  // print_project_info(project_data_group);
}

pub fn do_new_files_subcommand(
  command: CreateFilesCommand,
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  just_created_library_project_at: Option<String>
) {
  let project_data_group: UseableFinalProjectDataGroup = get_project_info_or_exit(
    &given_root_dir,
    &dep_config,
    FinalProjectLoadContext {
      about_to_generate_doxyfile: false,
      about_to_generate_sphinx_files: false,
      just_created_library_project_at
    }
  );

  // print_project_info(project_data_group);
  if let None = project_data_group.operating_on {
    exit_error_log("Tried to create files while not operating on a project. Make sure you are inside a project directory containing a cmake_data.yaml file.")
  }

  match handle_create_files(&project_data_group.operating_on.unwrap(), &command) {
    Ok(_) => {
      // Nothing needs to happen here, since a creation message is printed for every file that is created.
    },
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
        should_generate_cmakelists
      );

      return maybe_general_new_project_info;
    },
    Err(error_message) => exit_error_log(&error_message)
  }
}

pub fn do_dependency_config_update_subcommand(command: UpdateDependencyConfigsCommand) {
  println!("{}", "Beginning dependency config repo update...".green());

  match update_dependency_config_repo(&command.branch) {
    Err(err) => exit_error_log(format!(
      "{}\n\t{}",
      "Failed to update dependency config repo: ".red(),
      err.to_string()
    )),
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
            println!(
              "Successfully {}",
              "updated dependency config repo".green()
            );
          }
        }
      }
    }
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
    FinalProjectLoadContext::default()
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