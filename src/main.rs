#[macro_use]
extern crate lazy_static;

mod common;
mod project_info;
mod logger;
mod file_writers;
mod cli_config;
mod project_generator;
mod program_actions;

use clap::Parser;

use cli_config::{CLIProjectGenerationInfo};
use logger::exit_error_log;

use program_actions::*;
use project_generator::{DefaultProjectInfo};

use crate::{project_info::{raw_data_in::dependencies::{all_raw_supported_dependency_configs, internal_dep_config::AllRawPredefinedDependencies}}, project_generator::configuration::{MainFileLanguage, CreationProjectOutputType}, cli_config::clap_cli_config::{Opts, SubCommandStruct, DepConfigSubCommand, CreateFilesCommand, FileCreationLang}};

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
              println!();
              let new_file_name: String = match prompt_for_initial_compiled_lib_file_pair_name(&new_project_info.project.name) {
                Err(io_err) => exit_error_log(io_err.to_string()),
                Ok(relative_name) => relative_name
              };

              // TODO: Refactor this somehow. Right now, this parses all cmake_data.yaml files 
              // for the whole project, then does it again when generating the CMakeLists.
              // While that isn't causing problems, it shouldn't have to be done twice
              // Potential fix: make a command chain builder. A string of commands should be built and
              // executed, reusing the same FinalProjectData tree and all that. That's a TODO for future
              // refactoring though, since this works fine.
              do_new_files_subcommand(
                CreateFilesCommand {
                  language: match new_project_info.project_lang {
                    MainFileLanguage::C => FileCreationLang::C,
                    MainFileLanguage::Cpp => FileCreationLang::Cpp,
                    MainFileLanguage::Cpp2 => FileCreationLang::Cpp2
                  },
                  relative_file_names: vec![new_file_name],
                  which: String::from("hs"),
                  use_pragma_guards: false,
                  should_files_be_private: false
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
      SubCommandStruct::UseFile(command) => {
        should_generate_cmakelists = false;

        let file_copy_result = copy_default_file(
          &command,
          &given_root_dir,
          &dep_config
        );

        if let Err(err) = file_copy_result {
          exit_error_log(err.to_string());
        }
      },
      SubCommandStruct::GenDefault(command) => {
        let file_copy_result = generate_default_file(
          &command,
          &given_root_dir,
          &dep_config
        );

        if let Err(err) = file_copy_result {
          exit_error_log(err.to_string());
        }
      },
      SubCommandStruct::TargetInfo(command) => {
        should_generate_cmakelists = false;

        print_target_info(
          &command,
          &given_root_dir,
          &dep_config
        );
      },
      SubCommandStruct::ProjectInfo(command) => {
        should_generate_cmakelists = false;

        print_project_info(
          &command,
          &given_root_dir,
          &dep_config
        );
      },
      SubCommandStruct::PredepInfo(command) => {
        should_generate_cmakelists = false;

        print_predep_info(
          &command,
          &dep_config
        );
      },
      SubCommandStruct::ToolInfo(command) => {
        should_generate_cmakelists = false;

        print_tool_info(command);
      },
      SubCommandStruct::DepConfig(_) => {
        unreachable!();
      }
    }
  }

  if should_generate_cmakelists {
    do_generate_project_configs(
      &given_root_dir,
      &dep_config
    );
  }

  println!("");
}
