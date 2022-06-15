mod c_file_generation;
mod cpp_file_generation;
mod default_project_config;
mod prompt;

pub use default_project_config::configuration;
use serde::Serialize;

use std::{fs::{File, remove_dir_all, create_dir_all, create_dir}, io::{self, ErrorKind}, path::Path};

use crate::{project_generator::{c_file_generation::generate_c_main, cpp_file_generation::generate_cpp_main, default_project_config::{DefaultProject, configuration::{MainFileLanguage, CreationProjectOutputType}, get_default_project_config, get_default_subproject_config, main_file_name}, prompt::{prompt_once, prompt_for_project_output_type, prompt_for_language, prompt_for_description}}, program_actions::ProjectTypeCreating};

use self::{prompt::{prompt_until_boolean, PromptResult}};

const SRC_DIR: &'static str = "src";
const INCLUDE_DIR: &'static str = "include";
const TEMPLATE_IMPL_DIR: &'static str = "template_impls";


pub fn create_project_at(
  new_project_root: &str,
  project_type_creating: ProjectTypeCreating,
  project_lang: Option<MainFileLanguage>,
  project_output_type: Option<CreationProjectOutputType>
) -> io::Result<Option<DefaultProject>> {
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
    create_dir(project_root)?;

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

    let folder_generation_include_prefix: String = if let ProjectTypeCreating::Subproject(current_project_context) = &project_type_creating
      { current_project_context.nested_include_prefix(&include_prefix) }
      else { include_prefix.clone() };

    for source_code_dir in [SRC_DIR, INCLUDE_DIR, TEMPLATE_IMPL_DIR] {
      let mut extended_path = project_root.to_path_buf();
      extended_path.push(source_code_dir);
      extended_path.push(&folder_generation_include_prefix);
      create_dir_all(extended_path)?;
    }

    let output_type_selection: CreationProjectOutputType = project_output_type
      .unwrap_or(prompt_for_project_output_type()?);

    let lang_selection: MainFileLanguage = project_lang.unwrap_or(prompt_for_language()?);
    let project_description: String = prompt_for_description()?;

    let project_info: DefaultProject = build_default_project_info(
      &project_type_creating,
      project_name,
      &include_prefix,
      &lang_selection,
      &output_type_selection,
      &project_description
    );

    let cmake_data_file = File::create(
      format!("{}/cmake_data.yaml",
      project_root.to_str().unwrap())
    )?;

    match &project_info {
      DefaultProject::MainProject(project_info) => 
        write_cmake_yaml(&cmake_data_file, project_info)?,
      DefaultProject::Subproject(project_info) => 
        write_cmake_yaml(&cmake_data_file, project_info)?
    }

    let mut main_file_path = project_root.to_owned();
    main_file_path.push(main_file_name(&lang_selection, &output_type_selection));

    match lang_selection {
      MainFileLanguage::C => generate_c_main(main_file_path, &output_type_selection)?,
      MainFileLanguage::Cpp => generate_cpp_main(main_file_path, &output_type_selection)?
    }

    println!("Generated {}", main_file_name(&lang_selection, &output_type_selection));

    return Ok(Some(project_info));
  }

  Ok(None)
}

fn build_default_project_info(
  project_type_creating: &ProjectTypeCreating,
  project_name: &str,
  include_prefix: &str,
  lang_selection: &MainFileLanguage,
  output_type_selection: &CreationProjectOutputType,
  project_description: &str
) -> DefaultProject {
  if let ProjectTypeCreating::Subproject(_) = project_type_creating {
    DefaultProject::Subproject(
      get_default_subproject_config(
        &project_name,
        &include_prefix,
        &lang_selection,
        &output_type_selection,
        &project_description
      )
    ) 
  }
  else {
    DefaultProject::MainProject(
      get_default_project_config(
        &project_name,
        &include_prefix,
        &lang_selection,
        &output_type_selection,
        &project_description
      )
    )
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
