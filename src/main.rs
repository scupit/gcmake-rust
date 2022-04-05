mod project_info;
mod logger;
mod file_writers;
mod cli_config;
mod project_generator;
mod program_actions;

use logger::exit_error_log;

use clap::Clap;
use cli_config::{Opts, SubCommand};
use program_actions::*;

use crate::{project_info::{raw_data_in::dependencies::{supported_dependency_configs, internal_dep_config::AllPredefinedDependencies}}, file_writers::write_configurations};


// TODO: Handle library creation for Static and Shared libraries.
// Also allow both at once, so the user can select which type is built in the CMake GUI.
fn main() {
  let opts: Opts = Opts::parse();

  let dep_config: AllPredefinedDependencies = match supported_dependency_configs() {
    Ok(config) => config,
    Err(error_message) => exit_error_log(&error_message)
  };

  // Project root is only set by the user when using the default command. When using subcommands or unspecified
  // in the main command, uses the current working directory.
  let mut project_root_dir: String = opts.project_root;
  let mut should_generate_cmakelists: bool = true;

  if let Some(subcommand) = opts.subcommand {
    match subcommand {
      SubCommand::New(command) => handle_create_project(
        command,
        &mut project_root_dir,
        &mut should_generate_cmakelists
      ),
      SubCommand::GenFile(command) => {
        get_project_info_then(&project_root_dir, &dep_config, |project_data| {
          match handle_create_files(project_data, &command) {
            Ok(_) => println!("Files written successfully!"),
            Err(error_message) => exit_error_log(&error_message)
          }
        })
      }
    }
  }

  if should_generate_cmakelists {
    get_project_info_then(&project_root_dir, &dep_config, |project_data| {
      write_configurations(
        &project_data,
        |config_name| println!("Beginning {} configuration step...", config_name),
        |(config_name, config_result)| match config_result {
          Ok(_) => println!("{} configuration written successfully!", config_name),
          Err(err) => {
            println!("Writing {} configuration failed with error:", config_name);
            println!("{:?}", err)
          }
        }
      ); 
    });
  }

  println!("");
}