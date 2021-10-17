mod data_types;
mod item_resolver;
mod logger;
mod cmake_utils_writer;
mod cmakelists_writer;
mod cli_config;
mod project_generator;

use item_resolver::FinalProjectData;
use logger::exit_error_log;
use cmakelists_writer::write_cmakelists;

use clap::Clap;
use cli_config::{NewProjectCommand, Opts, SubCommand};
use project_generator::{create_project_at, configuration::MainFileLanguage};

use crate::{item_resolver::path_manipulation::cleaned_path_str, project_generator::configuration::{OutputLibType, ProjectOutputType}};

// TODO: Handle library creation for Static and Shared libraries.
// Also allow both at once, so the user can select which type is built in the CMake GUI.
fn main() {
  let opts: Opts = Opts::parse();

  // Project root is only set by the user when using the default command. When using subcommands or unspecified
  // in the main command, uses the current working directory.
  let mut project_root_dir: String = opts.project_root;
  let mut should_generate_cmakelists: bool = true;

  if let Some(subcommand) = opts.subcommand {
    match subcommand {
      SubCommand::New(command) => handle_create_project(
        command,
        &mut project_root_dir,
        &mut should_generate_cmakelists)
    }
  }

  if should_generate_cmakelists {
    println!("\nBeginning CMakeLists generation...");

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

  println!("");
}

fn handle_create_project(
  command: NewProjectCommand,
  project_root_dir: &mut String,
  should_generate_cmakelists: &mut bool
) {
  if cleaned_path_str(&command.new_project_name).contains("/") {
    exit_error_log(&format!(
      "When generating a project, the project root cannot be a path. However, \"{}\" is a path.",
      command.new_project_name
    ));
  }

  let project_root_generating: String = if command.subproject {
    let new_root = format!("./subprojects/{}", &command.new_project_name);
    println!("\nCreating subproject in {}\n", new_root);

    new_root
  }
  else {
    let true_project_root = format!("./{}", &command.new_project_name);
    println!("\nCreating project in {}\n", true_project_root);
    *project_root_dir = true_project_root.clone();
    true_project_root
  };

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
    Some(ProjectOutputType::Library(OutputLibType::ToggleStaticOrShared))
  } else if command.static_lib {
    Some(ProjectOutputType::Library(OutputLibType::Static))
  } else if command.shared_lib {
    Some(ProjectOutputType::Library(OutputLibType::Shared))
  } else {
    None
  };

  match create_project_at(
    &project_root_generating,
    maybe_project_lang,
    maybe_project_output_type,
    command.subproject
  ) {
    Ok(maybe_project) => match maybe_project {
      Some(default_project) => {
        let project_like = default_project.unwrap_projectlike();

        println!("{} created successfully", project_like.get_name());

        // TODO: After creating a subproject, add that subproject to the main build file automatically and rewrite it.
        // This isn't done currently because the default serializer looks messy.
        if command.subproject {
          println!(
            "\nMake sure you add your subproject \"{}\" to the main cmake_data.yaml. This is not yet done automatically.",
            command.new_project_name
          );
        }
      },
      None => {
        println!("Project not created. Skipping CMakeLists generation.");
        *should_generate_cmakelists = false;
      }
    },
    Err(err) => println!("{}", err)
  }
}
