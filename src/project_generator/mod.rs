mod c_file_generation;
mod cpp_file_generation;
mod default_project_config;

pub use default_project_config::configuration;
use serde::Serialize;

use std::{fs::{File, create_dir, remove_dir_all}, io::{self, ErrorKind, Write, stdin}, path::Path};

use crate::{data_types::raw_types::RawProject, project_generator::{c_file_generation::generate_c_main, cpp_file_generation::generate_cpp_main, default_project_config::{DefaultProject, configuration::{MainFileLanguage, ProjectOutputType}, get_default_project_config, get_default_subproject_config, main_file_name}}};

use self::configuration::OutputLibType;

const SRC_DIR: &'static str = "src";
const INCLUDE_DIR: &'static str = "include";
const TEMPLATE_IMPL_DIR: &'static str = "template_impls";

#[derive(Debug)]
enum PromptResult {
  Yes,
  No,
  Custom(String),
  Empty
}

impl PromptResult {
  fn unwrap_or(self, empty_replacement: String) -> String {
    return match self {
      Self::Yes => "y".to_owned(),
      Self::No => "n".to_owned(),
      Self::Custom(value) => value,
      Self::Empty => empty_replacement
    }
  }

  fn custom_into<T, F>(self, converter: F) -> T 
    where F: FnOnce(String) -> T
  {
    return converter(self.unwrap_custom())
  }

  fn custom_into_io_result<T, F>(self, converter: F) -> io::Result<T>
    where F: FnOnce(String) -> io::Result<T>
  {
    return converter(self.unwrap_custom())
  }

  fn unwrap_custom(self) -> String {
    if let Self::Custom(value) = self {
      return value;
    }

    panic!("Cannot unwrap a PrompResult which is not a Custom value.");
  }

  fn is_yes_or_no(&self) -> bool {
    return match *self {
      Self::Yes | Self::No => true,
      _ => false
    }
  }

  fn is_custom(&self) -> bool {
    return if let Self::Custom(_) = &self {
      true
    }
    else { false }
  }

  fn from_str(string: &str) -> PromptResult {
    match string.trim() {
      "" => PromptResult::Empty,
      "y" => PromptResult::Yes,
      "n" => PromptResult::No,
      custom_value => PromptResult::Custom(custom_value.to_string())
    }
  }
}

pub fn create_project_at(
  new_project_root: &str,
  project_lang: Option<MainFileLanguage>,
  project_output_type: Option<ProjectOutputType>,
  is_subproject: bool
) -> io::Result<Option<DefaultProject>> {
  let project_name: &str;

  {
    let start_index: usize = if let Some(last_slash_index) = new_project_root.rfind("/") {
      last_slash_index + 1
    } else {
      0
    };

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
      create_dir(extended_path)?;
    }

    let default_prefix = project_name
      .to_uppercase()
      .replace("-", "_");

    let include_prefix = prompt_once(
      &format!("include prefix ({}): ", &default_prefix)
    )?.unwrap_or(default_prefix);

    for source_code_dir in [SRC_DIR, INCLUDE_DIR, TEMPLATE_IMPL_DIR] {
      let mut extended_path = project_root.to_path_buf();
      extended_path.push(source_code_dir);
      extended_path.push(&include_prefix);
      create_dir(extended_path)?;
    }

    let output_type_selection: ProjectOutputType = if let Some(output_selection) = project_output_type {
      output_selection
    } else { prompt_for_project_output_type()? };

    let lang_selection: MainFileLanguage = if let Some(lang) = project_lang {
      lang
    } else { prompt_for_language()? };

    let project_description: String = prompt_for_description()?;

    let project_info: DefaultProject = if is_subproject {
      DefaultProject::Subproject(
        get_default_subproject_config(
          &project_name,
          &include_prefix,
          &lang_selection,
          &output_type_selection,
          &project_description
        )
      ) 
    } else {
      DefaultProject::MainProject(
        get_default_project_config(
          &project_name,
          &include_prefix,
          &lang_selection,
          &output_type_selection,
          &project_description
        )
      )
    };

    let cmake_data_file = File::create(format!("{}/cmake_data.yaml", project_root.to_str().unwrap()))?;

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

fn prompt_once(prompt: &str) -> io::Result<PromptResult> {
  let mut buffer = String::new();

  print!("{}", prompt);
  io::stdout().flush()?;

  stdin().read_line(&mut buffer)?;
  return Ok(PromptResult::from_str(buffer.trim()))
}

fn prompt_until<T>(prompt: &str, predicate: T) -> io::Result<PromptResult>
  where T: Fn(&PromptResult) -> bool
{
  let mut buffer = String::new();

  print!("{}", prompt);
  io::stdout().flush()?;

  stdin().read_line(&mut buffer)?;
  let mut result: PromptResult = PromptResult::from_str(buffer.trim());

  while !predicate(&result) {
    buffer.clear();

    print!("{}", prompt);
    io::stdout().flush()?;

    stdin().read_line(&mut buffer)?;
    result = PromptResult::from_str(buffer.trim());
  }

  return Ok(result)
}

fn prompt_until_boolean(prompt: &str) -> io::Result<PromptResult> {
  prompt_until(prompt, |result| result.is_yes_or_no())
}

fn prompt_until_value(prompt: &str) -> io::Result<PromptResult> {
  prompt_until(prompt, |result| result.is_custom())
}

// TODO: Refactor these multi-option prompts into an actual system. This is ugly and confusing.
fn prompt_for_language() -> io::Result<MainFileLanguage> {
  let prompt_result =  prompt_until(
    "1: C\n2: C++\nChoose Language (1 or 2): ",
    |result| if let PromptResult::Custom(value) = result {
      match value.as_str() {
        "1" | "C" => true,
        "2" | "C++" => true,
        _ => false
      }
    } else { false }
  )?;

  return Ok(
    prompt_result.custom_into(|value| match value.as_str() {
      "1" | "C" => MainFileLanguage::C,
      "2" | "C++" => MainFileLanguage::Cpp,
      _ => MainFileLanguage::Cpp
    })
  )
}

fn prompt_for_project_output_type() -> io::Result<ProjectOutputType> {
  let prompt_result =  prompt_until(
    "1: Executable\n2: Library\nChoose Project Type (1 or 2): ",
    |result| if let PromptResult::Custom(value) = result {
      match value.as_str() {
        "1" | "Executable" => true,
        "2" | "Library" => true,
        _ => false
      }
    } else { false }
  )?;

  return prompt_result.custom_into_io_result(|value| match value.as_str() {
    "1" | "Executable" => Ok(ProjectOutputType::Executable),
    "2" | "Library" => Ok(ProjectOutputType::Library(prompt_for_lib_output_type()?)),
    _ => Ok(ProjectOutputType::Executable)
  })
}

fn prompt_for_lib_output_type() -> io::Result<OutputLibType> {
  let prompt_result =  prompt_until(
    "1: Static\n2: Shared\nChoose Project Type (1 or 2): ",
    |result| if let PromptResult::Custom(value) = result {
      match value.as_str() {
        "1" | "Static" => true,
        "2" | "Shared" => true,
        _ => false
      }
    } else { false }
  )?;

  return Ok(
    prompt_result.custom_into(|value| match value.as_str() {
      "1" | "Static" => OutputLibType::Static,
      "2" | "Shared" => OutputLibType::Shared,
      _ => OutputLibType::Static
    })
  )
}

fn prompt_for_description() -> io::Result<String> {
  Ok(prompt_until_value("Description: ")?.unwrap_custom())
}