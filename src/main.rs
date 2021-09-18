mod data_types;
mod item_resolver;
mod logger;
mod cmakelists_writer;
mod cli_config;
mod project_generator;

use std::io;

use item_resolver::FinalProjectData;
use logger::exit_error_log;
use cmakelists_writer::write_cmakelists;

use clap::Clap;
use cli_config::{NewProjectCommand, Opts, SubCommand};
use project_generator::{create_project_at, configuration::MainFileLanguage};

use crate::project_generator::configuration::ProjectOutputType;

// TODO: Handle library creation for Static and Shared libraries.
// Also allow both at once, so the user can select which type is built in the CMake GUI.
fn main() {
  let opts: Opts = Opts::parse();

  let mut project_root_dir: String = opts.project_root;
  let mut should_generate_cmakelists: bool = true;

  if let Some(subcommand) = opts.subcommand {
    match subcommand {
      SubCommand::New(command) => handle_create_project(command, &mut project_root_dir, &mut should_generate_cmakelists)
    }
  }

  if should_generate_cmakelists {
    println!("Beginning CMakeLists generation...");

    match FinalProjectData::new(&project_root_dir) {
      Ok(project_data) => {
        match write_cmakelists(&project_data) {
          Ok(_) => println!("CMakeLists all written successfully!"),
          Err(err) => println!("{:?}", err)
        }
      },
      Err(message) => exit_error_log(&message)
    }
  }
}

fn handle_create_project(
  command: NewProjectCommand,
  project_root_dir: &mut String,
  should_generate_cmakelists: &mut bool
) {
  if command.subproject {
    *project_root_dir = format!("subprojects/{}", project_root_dir);
  }

  let maybe_project_lang: Option<MainFileLanguage> = if command.c {
    Some(MainFileLanguage::C)
  } else if command.cpp {
    Some(MainFileLanguage::Cpp)
  } else {
    None
  };

  let maybe_project_output_type: Option<ProjectOutputType> = if command.executable {
    Some(ProjectOutputType::Executable)
  } else if command.library {
    Some(ProjectOutputType::Library)
  } else {
    None
  };

  match create_project_at(
    &command.new_project_root,
    maybe_project_lang,
    maybe_project_output_type,
    command.subproject
  ) {
    Ok(maybe_project) => match maybe_project {
      Some(default_project) => {
        let project_like = default_project.unwrap_projectlike();

        println!("{} created successfully", project_like.get_name());
        *project_root_dir = project_like.get_name().to_owned();
      },
      None => {
        println!("Project not created. Skipping CMakeLists generation.");
        *should_generate_cmakelists = false;
      }
    },
    Err(err) => println!("{}", err)
  }
}
