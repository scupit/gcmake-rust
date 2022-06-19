use std::rc::Rc;
use crate::{cli_config::{NewProjectCommand}, project_info::{path_manipulation::cleaned_path_str, final_project_data::{FinalProjectData}, raw_data_in::ProjectLike}, logger::exit_error_log, project_generator::{configuration::{MainFileLanguage, CreationProjectOutputType, OutputLibType}, create_project_at, GeneralNewProjectInfo}, program_actions::parse_project_info};

pub enum ProjectTypeCreating {
  RootProject,
  Subproject(Rc<FinalProjectData>)
}

fn make_project_creating_info(
  command: &NewProjectCommand,
  maybe_current_project: &Option<Rc<FinalProjectData>>
) -> ProjectTypeCreating {
  if let Some(current_project_rc) = maybe_current_project {
    if !command.subproject {
      exit_error_log(&format!(
        "Unable to find the current project operating on while attempting to generate a subproject. Make sure you are currently in a directory which contains a cmake_data.yaml file."
      ));
    } 
    return ProjectTypeCreating::Subproject(Rc::clone(current_project_rc));
  }
  else {
    if command.subproject {
      exit_error_log(&format!(
        "Unable to find the current project operating on while attempting to generate a subproject. Make sure you are currently in a directory which contains a cmake_data.yaml file."
      ));
    }
    return ProjectTypeCreating::RootProject;
  }
}

pub fn handle_create_project(
  command: NewProjectCommand,
  maybe_current_project: &Option<Rc<FinalProjectData>>,
  project_root_dir: &mut String,
  should_generate_cmakelists: &mut bool
) -> Option<GeneralNewProjectInfo> {
  let project_creation_info = make_project_creating_info(&command, maybe_current_project);

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

  let maybe_project_output_type: Option<CreationProjectOutputType> = if command.executable {
    Some(CreationProjectOutputType::Executable)
  } else if command.library {
    Some(CreationProjectOutputType::Library(OutputLibType::ToggleStaticOrShared))
  } else if command.static_lib {
    Some(CreationProjectOutputType::Library(OutputLibType::Static))
  } else if command.shared_lib {
    Some(CreationProjectOutputType::Library(OutputLibType::Shared))
  } else {
    None
  };

  match create_project_at(
    &project_root_generating,
    project_creation_info,
    maybe_project_lang,
    maybe_project_output_type
  ) {
    Ok(maybe_project) => match maybe_project {
      Some(general_new_project_info) => {
        let project_like = general_new_project_info.project.unwrap_projectlike();

        println!("{} created successfully", project_like.get_name());

        // TODO: After creating a subproject, add that subproject to the main build file automatically and rewrite it.
        // This isn't done currently because the default serializer looks messy.
        if command.subproject {
          println!(
            "\nMake sure you add your subproject \"{}\" to the main cmake_data.yaml. This is not yet done automatically.",
            command.new_project_name
          );
        }

        return Some(general_new_project_info);
      },
      None => {
        println!("Project not created. Skipping CMakeLists generation.");
        *should_generate_cmakelists = false;
      }
    },
    Err(err) => println!("{}", err)
  }

  return None;
}