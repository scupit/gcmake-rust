use std::{rc::Rc};
use crate::{cli_config::{CLIProjectGenerationInfo, CLIProjectTypeGenerating}, project_info::{path_manipulation::cleaned_path_str, final_project_data::{FinalProjectData}}, logger::exit_error_log, project_generator::{configuration::{MainFileLanguage, CreationProjectOutputType, OutputLibType}, create_project_at, GeneralNewProjectInfo}, program_actions::{parse_project_info, manage_dependencies::gcmake_config_root_dir}};

pub enum ProjectTypeCreating {
  RootProject,
  Subproject {
    parent_project: Rc<FinalProjectData>
  },
  Test {
    parent_project: Rc<FinalProjectData>
  }
}

impl ProjectTypeCreating {
  fn from_generation_config(
    generation_info: &CLIProjectGenerationInfo,
    project_operating_on: &Option<Rc<FinalProjectData>>
  ) -> ProjectTypeCreating {
    return match &generation_info.project_type {
      CLIProjectTypeGenerating::RootProject => {
        match project_operating_on {
          None => ProjectTypeCreating::RootProject,
          Some(project_rc) => {
            exit_error_log(&format!(
              "Generating a root project inside another root project is forbidden. Try generating a subproject instead."
            ));
          },
        }
      },
      CLIProjectTypeGenerating::Subproject => {
        match project_operating_on {
          Some(project_rc) => {
            ProjectTypeCreating::Subproject {
              parent_project: Rc::clone(project_rc)
            }
          },
          None => {
            exit_error_log(&format!(
              "Unable to find the current project operating on while attempting to generate a subproject. Make sure your current working directory contains a cmake_data.yaml file."
            ));
          }
        }
      },
      CLIProjectTypeGenerating::Test => {
        match project_operating_on {
          Some(project_rc) => {
            ProjectTypeCreating::Test {
              parent_project: Rc::clone(project_rc)
            }
          },
          None => {
            exit_error_log(&format!(
              "Unable to find the current project operating on while attempting to generate a test. Make sure your current working directory contains a cmake_data.yaml file."
            ));
          }
        }
      }
    }
  }
}


pub fn handle_create_project(
  generation_info: CLIProjectGenerationInfo,
  maybe_current_project: &Option<Rc<FinalProjectData>>,
  project_root_dir: &str,
  should_generate_cmakelists: &mut bool
) -> Option<GeneralNewProjectInfo> {
  let project_creation_info: ProjectTypeCreating = ProjectTypeCreating::from_generation_config(
    &generation_info,
    maybe_current_project
  );

  if cleaned_path_str(&generation_info.project_name).contains("/") {
    exit_error_log(&format!(
      "When generating a project, the project root must be a name, not a path. However, \"{}\" is a path.",
      generation_info.project_name
    ));
  }

  let project_root_generating: String = match &generation_info.project_type {
    CLIProjectTypeGenerating::RootProject => {
      let true_project_root = format!("./{}", &generation_info.project_name);
      println!("\nCreating project in {}\n", true_project_root);
      true_project_root
    },
    CLIProjectTypeGenerating::Subproject => {
      let subproject_root = format!("./subprojects/{}", &generation_info.project_name);
      println!("\nCreating subproject in {}\n", &subproject_root);

      subproject_root
    },
    CLIProjectTypeGenerating::Test => {
      let test_root = format!("./tests/{}", &generation_info.project_name);
      println!("\nCreating test in {}\n", &test_root);

      test_root
    }
  };

  match create_project_at(
    &project_root_generating,
    project_creation_info,
    generation_info.language,
    generation_info.project_output_type
  ) {
    Ok(maybe_project) => match maybe_project {
      Some(general_new_project_info) => {
        println!("{} created successfully", &general_new_project_info.project.name);

        if let CLIProjectTypeGenerating::Subproject = &generation_info.project_type {
          // TODO: After creating a subproject, add that subproject to the main build file automatically and rewrite it.
          // This isn't done currently because the default serializer looks messy.
          // TODO: Actually, just remove the need to explicitly specify subprojects. They can be easily resolved
          // automatically.
          println!(
            "\nMake sure you add your subproject \"{}\" to the main cmake_data.yaml. This is not yet done automatically.",
            &generation_info.project_name
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