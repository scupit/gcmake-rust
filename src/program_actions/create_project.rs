use std::{rc::Rc};
use crate::{cli_config::{CLIProjectGenerationInfo, CLIProjectTypeGenerating}, project_info::{path_manipulation::cleaned_path_str, final_project_data::{FinalProjectData}}, logger::exit_error_log, project_generator::{GeneralNewProjectInfo, create_project_at}};
use colored::*;

pub enum ProjectTypeCreating {
  RootProject {
    include_emscripten_support: bool
  },
  Subproject {
    parent_project: Rc<FinalProjectData>
  },
  Test {
    parent_project: Rc<FinalProjectData>
  }
}

impl ProjectTypeCreating {
  pub fn is_test(&self) -> bool {
    return match self {
      ProjectTypeCreating::Test { .. } => true,
      _ => false
    }
  }

  fn from_generation_config(
    generation_info: &CLIProjectGenerationInfo,
    project_operating_on: &Option<Rc<FinalProjectData>>
  ) -> ProjectTypeCreating {
    return match &generation_info.project_type {
      CLIProjectTypeGenerating::RootProject => {
        match project_operating_on {
          None => ProjectTypeCreating::RootProject {
            include_emscripten_support: generation_info.should_include_emscripten_support
          },
          Some(_) => {
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
        println!("Project {} created successfully", &general_new_project_info.project.name.cyan());
        return Some(general_new_project_info);
      },
      None => {
        println!(
          "Project not created. {}",
          "Skipping CMakeLists generation.".green()
        );
        *should_generate_cmakelists = false;
      }
    },
    Err(err) => println!("{}", err)
  }

  return None;
}