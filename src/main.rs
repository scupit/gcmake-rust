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
use project_info::final_project_data::UseableFinalProjectDataGroup;

use crate::{project_info::{raw_data_in::dependencies::{supported_dependency_configs, internal_dep_config::AllPredefinedDependencies}}, file_writers::write_configurations};

fn print_project_info(project_data_group: UseableFinalProjectDataGroup) {
  println!("PROJECT INFORMATION\n----------------------------------------");
  
  println!("\nroot project: {}", project_data_group.root_project.get_project_root());
  println!("root project absolute: {}\n", project_data_group.root_project.get_absolute_project_root());

  match project_data_group.operating_on {
    Some(proj) => {
      println!("operating on: {}", proj.get_project_root());
      println!("operating on absolute: {}", proj.get_absolute_project_root());
    },
    None => println!("Not operating on a project")
  }
}

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
  let mut given_root_dir: String = opts.project_root;
  let mut should_generate_cmakelists: bool = true;

  if let Some(subcommand) = opts.subcommand {
    match subcommand {
      SubCommand::New(command) => handle_create_project(
        command,
        &mut given_root_dir,
        &mut should_generate_cmakelists
      ),
      SubCommand::GenFile(command) => {
        get_project_info_then(&given_root_dir, &dep_config, |project_data_group| {
          // print_project_info(project_data_group);
          if let None = project_data_group.operating_on {
            exit_error_log("Tried to create files while not operating on a project. Make sure you are inside a project directory containing a cmake_data.{yaml|yml} file.")
          }

          match handle_create_files(&project_data_group.operating_on.unwrap(), &command) {
            Ok(_) => println!("Files written successfully!"),
            Err(error_message) => exit_error_log(&error_message)
          }
        })
      }
    }
  }

  if should_generate_cmakelists {
    get_project_info_then(&given_root_dir, &dep_config, |project_data_group| {
      // print_project_info(project_data_group);
      write_configurations(
        &project_data_group.root_project,
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