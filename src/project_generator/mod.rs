mod c_file_generation;
mod cpp_file_generation;
mod default_project_config;
mod prompt;

pub use default_project_config::{*, configuration::*};
use serde::Serialize;

use std::{fs::{File, remove_dir_all, create_dir_all, create_dir, self}, io::{self, ErrorKind}, path::{Path, PathBuf}};

use crate::{project_generator::{c_file_generation::generate_c_main, cpp_file_generation::generate_cpp_main, prompt::{prompt_once, prompt_for_project_output_type, prompt_for_language, prompt_for_description, prompt_for_vendor}}, program_actions::{ProjectTypeCreating, gcmake_config_root_dir}};

use self::{prompt::{prompt_until_boolean, PromptResult}};

const SRC_DIR: &'static str = "src";
const INCLUDE_DIR: &'static str = "include";
const TEMPLATE_IMPL_DIR: &'static str = "template_impls";

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
    let prompt = format!("Directory {} already exists. Overwrite it? (y or n): ", new_project_root);

    match prompt_until_boolean(&prompt)? {
      PromptResult::No => should_create_project = false,
      PromptResult::Yes => {
        remove_dir_all(project_root)?;
        println!("Directory removed. Generating new project...");
      },
      _ => ()
    }
  }

  if should_create_project {
    create_dir_all(project_root)?;

    for nested_dir in [SRC_DIR, INCLUDE_DIR, TEMPLATE_IMPL_DIR, "subprojects"] {
      let mut extended_path = project_root.to_path_buf();
      extended_path.push(nested_dir);
      create_dir_all(extended_path)?;
    }

    let default_prefix = project_name
      .to_uppercase()
      .replace("-", "_");

    let include_prefix: String = prompt_once(
      &format!("include prefix ({}): ", &default_prefix)
    )?.unwrap_or(default_prefix);

    let folder_generation_include_prefix: String = match &project_type_creating {
      ProjectTypeCreating::RootProject => include_prefix.clone(),
      ProjectTypeCreating::Subproject { parent_project } => {
        parent_project.nested_include_prefix(&include_prefix)
      },
      ProjectTypeCreating::Test { parent_project } => {
        parent_project.nested_include_prefix(&include_prefix)
      }
    };

    for source_code_dir in [SRC_DIR, INCLUDE_DIR, TEMPLATE_IMPL_DIR] {
      let mut extended_path = project_root.to_path_buf();
      extended_path.push(source_code_dir);
      extended_path.push(&folder_generation_include_prefix);
      create_dir_all(extended_path)?;
    }

    let output_type_selection: CreationProjectOutputType = match project_output_type {
      Some(out_type) => out_type,
      None => prompt_for_project_output_type()?
    };

    let lang_selection: MainFileLanguage = match project_lang {
      Some(lang) => lang,
      None => prompt_for_language()?
    };

    let project_vendor: String = match &project_type_creating {
      ProjectTypeCreating::RootProject => prompt_for_vendor()?,
      ProjectTypeCreating::Subproject { .. }
        | ProjectTypeCreating::Test { .. } => String::from("THIS IS IGNORED")
    };
    
    let project_description: String = prompt_for_description()?;

    let project_info: DefaultProjectInfo = build_default_project_info(
      &project_type_creating,
      project_name,
      &include_prefix,
      &lang_selection,
      &output_type_selection,
      &project_description,
      &project_vendor
    );

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

    let mut main_file_path = project_root.to_owned();
    main_file_path.push(main_file_name(project_name, &lang_selection, &output_type_selection));

    match lang_selection {
      MainFileLanguage::C => generate_c_main(main_file_path, &output_type_selection)?,
      MainFileLanguage::Cpp => generate_cpp_main(main_file_path, &output_type_selection)?
    }

    println!("Generated {}", main_file_name(project_name, &lang_selection, &output_type_selection));

    if let ProjectTypeCreating::RootProject = &project_type_creating {
      println!("Checking for default .clang-format...");

      // Copy .clang-format from the gcmake config dir, if the file exists.
      let mut clang_format_file_path: PathBuf = gcmake_config_root_dir();
      clang_format_file_path.push(".clang-format");

      if clang_format_file_path.is_symlink() {
        println!("{} is a symlink. Resolving...", clang_format_file_path.to_str().unwrap());
        clang_format_file_path = fs::read_link(&clang_format_file_path)?;
        println!(
          "The .clang-format symlink points to '{}'. Using that path instead.",
          clang_format_file_path.to_str().unwrap()
        );
      }

      if clang_format_file_path.is_file() {
        let mut destination = PathBuf::from(&project_root);
        destination.push(".clang-format");

        fs::copy(&clang_format_file_path, destination)
          .map_err(|the_err| {
            println!("Failed to copy {}. Reason:", clang_format_file_path.to_str().unwrap());
            the_err
          })?;

        println!("Default .clang-format successfully copied into project.");
      }
      else {
        println!(
          "Skipped Clang format file copy because '{}' was not found.",
          clang_format_file_path.to_str().unwrap()
        );
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
  project_vendor: &str
) -> DefaultProjectInfo {
  match project_type_creating {
    ProjectTypeCreating::RootProject => {
      DefaultProjectInfo::RootProject(
        get_default_project_config(
          project_name,
          include_prefix,
          lang_selection,
          output_type_selection,
          project_description,
          project_vendor
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
          project_description
        )
      )
    },
    ProjectTypeCreating::Test { parent_project } => {
      DefaultProjectInfo::TestProject(
        get_default_test_project_config(
          project_name,
          include_prefix,
          lang_selection,
          output_type_selection,
          project_description
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
      println!("Successfully wrote cmake_data.yaml");
      Ok(())
    },
    Err(err) => return Err(io::Error::new(ErrorKind::Other, err))
  }
}
