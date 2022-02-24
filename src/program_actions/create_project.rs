use crate::{cli_config::{NewProjectCommand}, project_info::{path_manipulation::cleaned_path_str, raw_data_in::dependencies::internal_dep_config::AllPredefinedDependencies, final_project_data::FinalProjectData, final_dependencies::FinalGitRepoDescriptor}, logger::exit_error_log, project_generator::{configuration::{MainFileLanguage, ProjectOutputType, OutputLibType}, create_project_at}};

pub fn handle_create_project(
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