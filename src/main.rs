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
  let project_root_dir = &opts.project_root;

  if let Some(subcommand) = opts.subcommand {
    match subcommand {
      SubCommand::New(command) => match create_project_at(&command.new_project_root) {
        Ok(_) => println!("Project created successfully"),
        Err(err) => println!("{}", err)
      }
    }
  }
  else {
    match FinalProjectData::new(project_root_dir) {
      Ok(project_data) => {
        match write_cmakelists(&project_data) {
          Ok(_)=> println!("CMakeLists all written successfully!"),
          Err(err) => println!("{:?}", err)
        }
      },
      Err(message) => exit_error_log(&message)
    }
  }
}
