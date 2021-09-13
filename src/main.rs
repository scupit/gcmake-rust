mod data_types;
mod item_resolver;
mod logger;
mod cmakelists_writer;
mod cli_config;
mod project_generator;

use item_resolver::FinalProjectData;
use logger::exit_error_log;
use cmakelists_writer::write_cmakelists;

use clap::Clap;
use cli_config::{CommandNew, Opts, SubCommand};
use project_generator::create_project_at;

fn main() {
  let opts: Opts = Opts::parse();

  let mut project_root_dir: String = opts.project_root;
  let mut should_generate_cmakelists: bool = true;

  if let Some(subcommand) = opts.subcommand {
    match subcommand {
      SubCommand::New(command) => match create_project_at(&command.new_project_root) {
        Ok(maybe_project) => match maybe_project {
          Some(raw_project) => {
            println!("Project {} created successfully", raw_project.get_name());
            project_root_dir = raw_project.name;
          },
          None => {
            println!("Project not created. Skipping CMakeLists generation.");
            should_generate_cmakelists = false;
          }
        },
        Err(err) => println!("{}", err)
      }
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
