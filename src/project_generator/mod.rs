mod c_file_generation;
mod cpp_file_generation;
mod default_project_config;
mod project_generator_prompts;
mod cpp_test_mains;

pub use default_project_config::{*, configuration::*};
use serde::Serialize;

use std::{fs::{File, remove_dir_all, create_dir_all, self}, io::{self, ErrorKind}, path::{Path, PathBuf}};

use crate::{program_actions::{ProjectTypeCreating, gcmake_config_root_dir}, common::prompt::{prompt_until_boolean, prompt_until_satisfies_or_default}, project_info::{base_include_prefix_for_test, gcmake_constants::{SRC_DIR, INCLUDE_DIR, ASSETS_DIR, SUBPROJECTS_DIR, TESTS_DIR}, validators::{is_valid_base_include_prefix}, FinalTestFramework}, project_generator::{project_generator_prompts::{prompt_for_project_output_type, prompt_for_language, prompt_for_vendor, prompt_for_description, prompt_for_needs_custom_main}, c_file_generation::generate_c_main, cpp_file_generation::{generate_cpp_main, TestMainInitInfo}}};
use colored::*;

pub struct GeneralNewProjectInfo {
  pub project: CreatedProject,
  pub project_lang: MainFileLanguage,
  pub project_output_type: CreationProjectOutputType,
  pub project_root: String
}

pub fn create_project_at(
  new_project_root: &str,
  project_type_creating: ProjectTypeCreating,
  project_lang: Option<MainFileLanguage>,
  project_output_type: Option<CreationProjectOutputType>
) -> io::Result<Option<GeneralNewProjectInfo>> {
  let project_name: &str;

  {
    let start_index: usize = if let Some(last_slash_index) = new_project_root.rfind("/")
      { last_slash_index + 1 }
      else { 0 };

    project_name = &new_project_root[start_index..];
  }

  let project_root = Path::new(new_project_root);
  let mut should_create_project: bool = true;

  if project_root.is_dir() {
    let prompt: String = format!("Directory {} already exists. Overwrite it?", new_project_root);

    if prompt_until_boolean(&prompt)? {
      remove_dir_all(project_root)?;
      println!("Directory removed. Generating new project...");
    }
    else {
      should_create_project = false;
    }
  }

  if should_create_project {
    if let ProjectTypeCreating::Test { parent_project } = &project_type_creating {
      if parent_project.get_test_framework().is_none() {
        return Err(io::Error::new(
          io::ErrorKind::Other,
          format!(
            "Tried to create a test project, however the root project doesn't specify a test framework. Please specify a test framework in the root project before generating a test."
          )
        ));
      }
    }

    // ----------------------------------------
    // Info collection section
    // ----------------------------------------

    let default_prefix = project_name
      .to_uppercase()
      .replace("-", "_");

    let include_prefix: String = prompt_until_satisfies_or_default(
      "Include prefix",
      is_valid_base_include_prefix,
      &default_prefix
    )?;

    let folder_generation_include_prefix: String = match &project_type_creating {
      ProjectTypeCreating::RootProject { .. } => include_prefix.clone(),
      ProjectTypeCreating::Subproject { parent_project } => {
        parent_project.nested_include_prefix(&include_prefix)
      },
      ProjectTypeCreating::Test { parent_project } => {
        parent_project.nested_include_prefix(&base_include_prefix_for_test(&include_prefix))
      }
    };

    let output_type_selection: CreationProjectOutputType = match project_output_type {
      Some(out_type) => out_type,
      None => prompt_for_project_output_type()?
    };

    let lang_selection: MainFileLanguage = match project_lang {
      Some(lang) => lang,
      None => prompt_for_language()?
    };

    let project_vendor: String = match &project_type_creating {
      ProjectTypeCreating::RootProject { .. } => prompt_for_vendor()?,
      _ => String::from("THIS IS IGNORED")
    };
    
    let project_description: String = prompt_for_description()?;

    let requires_custom_main: Option<bool> = match &project_type_creating {
      ProjectTypeCreating::Test { .. } => Some(prompt_for_needs_custom_main()?),
      _ => None
    };

    let project_info: DefaultProjectInfo = build_default_project_info(
      &project_type_creating,
      project_name,
      &include_prefix,
      &lang_selection,
      &output_type_selection,
      &project_description,
      &project_vendor,
      requires_custom_main
    );

    // ----------------------------------------
    // Folder and file generation section
    // ----------------------------------------
    // This section comes after all info is collected because the user should
    // be able to cancel during any project generation phase without a half-made
    // project being generated.

    create_dir_all(project_root)?;

    let cmake_data_file = File::create(
      format!("{}/cmake_data.yaml",
      project_root.to_str().unwrap())
    )?;

    println!("");

    match &project_info {
      DefaultProjectInfo::RootProject(project_info) => 
        write_cmake_yaml(&cmake_data_file, project_info)?,
      DefaultProjectInfo::Subproject(subproject_info) => 
        write_cmake_yaml(&cmake_data_file, subproject_info)?,
      DefaultProjectInfo::TestProject(test_project_info) =>
        write_cmake_yaml(&cmake_data_file, test_project_info)?
    }

    for nested_dir in [SRC_DIR, INCLUDE_DIR, ASSETS_DIR] {
      let mut extended_path = project_root.to_path_buf();
      extended_path.push(nested_dir);
      create_dir_all(extended_path)?;
    }

    if !project_type_creating.is_test() {
      for nested_dir in [SUBPROJECTS_DIR, TESTS_DIR] {
        let mut extended_path = project_root.to_path_buf();
        extended_path.push(nested_dir);
        create_dir_all(extended_path)?;
      }
    }

    for dir_requiring_include_suffix in [SRC_DIR, INCLUDE_DIR, ASSETS_DIR] {
      let mut extended_path = project_root.to_path_buf();
      extended_path.push(dir_requiring_include_suffix);
      extended_path.push(&folder_generation_include_prefix);
      create_dir_all(extended_path)?;
    }

    let mut main_file_path = project_root.to_owned();
    main_file_path.push(main_file_name(project_name, &lang_selection, &output_type_selection));

    match &project_type_creating {
      ProjectTypeCreating::Test { parent_project } => {
        let test_framework: &FinalTestFramework = parent_project.get_test_framework().as_ref().unwrap();

        generate_cpp_main(
          main_file_path,
          &output_type_selection,
          Some(TestMainInitInfo {
            test_framework,
            requires_custom_main: requires_custom_main.unwrap()
          }),
          &folder_generation_include_prefix,
          project_name,
          false
        )?;
      },
      _ => {
        match lang_selection {
          MainFileLanguage::C => {
            generate_c_main(
              main_file_path,
              &output_type_selection,
              &folder_generation_include_prefix,
              project_name
            )?
          },
          cpp_like_lang @ (MainFileLanguage::Cpp | MainFileLanguage::Cpp2) => {
            generate_cpp_main(
              main_file_path,
              &output_type_selection,
              None,
              &folder_generation_include_prefix,
              project_name,
              cpp_like_lang == MainFileLanguage::Cpp2
            )?
          }
        }
      }
    }

    println!(
      "{}",
      format!(
        "Generated {}",
        main_file_name(project_name, &lang_selection, &output_type_selection)
      ).cyan()
    );

    if let ProjectTypeCreating::RootProject { .. } = &project_type_creating {
      for default_file in [ ".clang-format", ".gitignore", ".clang-tidy" ] {
        println!("\nChecking for default {}...", default_file);

        // Copy file from the gcmake config root dir, if the file exists.
        let mut full_default_file_path: PathBuf = gcmake_config_root_dir();
        full_default_file_path.push(default_file);

        if full_default_file_path.is_symlink() {
          println!("{} is a symlink. Resolving...", full_default_file_path.to_str().unwrap());
          full_default_file_path = fs::read_link(&full_default_file_path)?;
          println!(
            "The {} symlink points to '{}'. Using that path instead.\n",
            default_file,
            full_default_file_path.to_str().unwrap()
          );
        }

        if full_default_file_path.is_file() {
          let mut destination = PathBuf::from(&project_root);
          destination.push(default_file);

          fs::copy(&full_default_file_path, destination)
            .map_err(|the_err| {
              println!("Failed to copy {}. Reason:", full_default_file_path.to_str().unwrap());
              the_err
            })?;

          println!("Default {} successfully copied into project.", default_file.cyan());
        }
        else {
          println!(
            "Skipped {} file copy because '{}' was not found.",
            default_file,
            full_default_file_path.to_str().unwrap()
          );
        }
      }
    }

    return Ok(Some( GeneralNewProjectInfo {
      project: CreatedProject {
        name: project_name.to_string(),
        info: project_info,
      },
      project_lang: lang_selection.clone(),
      project_output_type: output_type_selection,
      project_root: project_root.to_str().unwrap().to_string(),
    }));
  }

  Ok(None)
}

fn build_default_project_info(
  project_type_creating: &ProjectTypeCreating,
  project_name: &str,
  include_prefix: &str,
  lang_selection: &MainFileLanguage,
  output_type_selection: &CreationProjectOutputType,
  project_description: &str,
  project_vendor: &str,
  requires_custom_main: Option<bool>
) -> DefaultProjectInfo {
  match project_type_creating {
    ProjectTypeCreating::RootProject { .. } => {
      DefaultProjectInfo::RootProject(
        get_default_project_config(
          project_name,
          include_prefix,
          lang_selection,
          output_type_selection,
          project_type_creating,
          project_description,
          project_vendor,
          requires_custom_main
        )
      )
    },
    ProjectTypeCreating::Subproject { .. } => {
      DefaultProjectInfo::Subproject(
        get_default_subproject_config(
          project_name,
          include_prefix,
          lang_selection,
          output_type_selection,
          project_type_creating,
          project_description,
          requires_custom_main
        )
      )
    },
    ProjectTypeCreating::Test { .. } => {
      DefaultProjectInfo::TestProject(
        get_default_test_project_config(
          project_name,
          include_prefix,
          lang_selection,
          output_type_selection,
          project_type_creating,
          project_description,
          requires_custom_main
        )
      )
    }
  }
}

fn write_cmake_yaml<T: Serialize>(
  cmake_data_file: &File,
  project_info: &T
) -> io::Result<()> {
  match serde_yaml::to_writer(cmake_data_file, project_info) {
    Ok(_) => {
      println!("{}", "Successfully wrote cmake_data.yaml".cyan());
      Ok(())
    },
    Err(err) => return Err(io::Error::new(ErrorKind::Other, err))
  }
}
