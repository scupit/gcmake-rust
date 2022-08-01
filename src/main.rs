#[macro_use]
extern crate lazy_static;

mod project_info;
mod logger;
mod file_writers;
mod cli_config;
mod project_generator;
mod program_actions;

use std::io;

use clap::Parser;

use cli_config::{CLIProjectGenerationInfo, clap_cli_config::UpdateDependencyConfigsCommand, CLIProjectTypeGenerating};
use logger::exit_error_log;

use program_actions::*;
use project_generator::{GeneralNewProjectInfo, DefaultProjectInfo};
use project_info::final_project_data::{UseableFinalProjectDataGroup};

use crate::{project_info::{raw_data_in::dependencies::{all_raw_supported_dependency_configs, internal_dep_config::AllRawPredefinedDependencies}, final_project_data::ProjectLoadFailureReason}, file_writers::write_configurations, project_generator::configuration::{MainFileLanguage, CreationProjectOutputType}, cli_config::clap_cli_config::{Opts, SubCommandStruct, DepConfigSubCommand, CreateFilesCommand, FileCreationLang}};

// fn print_project_info(project_data_group: UseableFinalProjectDataGroup) {
//   println!("PROJECT INFORMATION\n----------------------------------------");
  
//   println!("\nroot project: {}", project_data_group.root_project.get_project_root());
//   println!("root project absolute: {}\n", project_data_group.root_project.get_absolute_project_root());

//   match project_data_group.operating_on {
//     Some(proj) => {
//       println!("operating on: {}", proj.get_project_root());
//       println!("operating on absolute: {}", proj.get_absolute_project_root());
//     },
//     None => println!("Not operating on a project")
//   }
// }

fn main() {
  let opts: Opts = Opts::parse();

  if let Some(SubCommandStruct::DepConfig(dep_config_subcommand)) = opts.subcommand {
    match dep_config_subcommand {
      DepConfigSubCommand::Update(command_update_deps) => {
        do_dependency_config_update_subcommand(command_update_deps);
      }
    }

    return;
  }

  let dep_config: AllRawPredefinedDependencies = match all_raw_supported_dependency_configs() {
    Ok(config) => config,
    Err(error_message) => exit_error_log(&error_message)
  };

  // Project root is only set by the user when using the default command. When using subcommands or unspecified
  // in the main command, uses the current working directory.
  let mut given_root_dir: String = opts.project_root;
  let mut should_generate_cmakelists: bool = true;

  println!("");

  if let Some(subcommand) = opts.subcommand {
    match subcommand {
      SubCommandStruct::New(new_project_subcommand) => {
        let maybe_project_info = do_new_project_subcommand(
          CLIProjectGenerationInfo::from(new_project_subcommand),
          &dep_config,
          &given_root_dir,
          &mut should_generate_cmakelists
        );

        if let Some(new_project_info) = maybe_project_info {
          if let DefaultProjectInfo::RootProject(_) = &new_project_info.project.info {
            given_root_dir = new_project_info.project_root.clone();
          }

          if let CreationProjectOutputType::Library(lib_type) = new_project_info.project_output_type {
            if lib_type.is_compiled_lib() {
              // TODO: Refactor this somehow. Right now, this parses all cmake_data.yaml files 
              // for the whole project, then does it again when generating the CMakeLists.
              // While that isn't causing problems, it shouldn't have to be done twice
              // Potential fix: make a command chain builder. A string of commands should be built and
              // executed, reusing the same FinalProjectData tree and all that. That's a TODO for future
              // refactoring though, since this works fine.
              do_new_files_subcommand(
                CreateFilesCommand {
                  language: match new_project_info.project_lang {
                    MainFileLanguage::C => FileCreationLang::Cpp,
                    MainFileLanguage::Cpp => FileCreationLang::Cpp
                  },
                  file_name: String::from("Placeholder"),
                  file_types: String::from("hs"),
                  use_pragma_guards: false
                },
                &new_project_info.project_root,
                &dep_config,
                Some(new_project_info.project_root.clone())
              );
            }
          }
        }
      },
      SubCommandStruct::GenFile(command) => do_new_files_subcommand(
        command,
        &given_root_dir,
        &dep_config,
        None
      ),
      SubCommandStruct::DepConfig(_) => {
        unreachable!();
      }
    }
  }

  if should_generate_cmakelists {
    do_generate_project_configs(
      &given_root_dir,
      &dep_config,
      None
    );
  }

  println!("");
}

fn do_generate_project_configs(
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  just_created_project_at: Option<String>
) {
  match parse_project_info(&given_root_dir, &dep_config, just_created_project_at) {
    Ok(project_data_group) => {
      // print_project_info(project_data_group);
      let config_write_result: io::Result<()> = write_configurations(
        &project_data_group,
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
    },
    Err(failure_reason) => exit_error_log(&failure_reason.extract_message())
  }
}

fn do_new_files_subcommand(
  command: CreateFilesCommand,
  given_root_dir: &str,
  dep_config: &AllRawPredefinedDependencies,
  just_created_project_at: Option<String>
) {
  match parse_project_info(&given_root_dir, &dep_config, just_created_project_at) {
    Ok(project_data_group) => {
      // print_project_info(project_data_group);
      if let None = project_data_group.operating_on {
        exit_error_log("Tried to create files while not operating on a project. Make sure you are inside a project directory containing a cmake_data.yaml file.")
      }

      match handle_create_files(&project_data_group.operating_on.unwrap(), &command) {
        Ok(_) => println!("Files written successfully!"),
        Err(error_message) => exit_error_log(&error_message)
      }
    },
    Err(failure_reason) => exit_error_log(&failure_reason.extract_message())
  }
}

fn do_new_project_subcommand(
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

fn do_dependency_config_update_subcommand(command: UpdateDependencyConfigsCommand) {
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

fn get_parent_project_for_new_project(
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