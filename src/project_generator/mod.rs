mod c_file_generation;
mod cpp_file_generation;
mod default_project_config;

use std::{collections::{HashMap, HashSet}, error::Error, fs::{File, create_dir, remove_dir_all}, io::{self, ErrorKind, Write, stdin}, iter::FromIterator, path::Path};

use crate::{data_types::raw_types::RawProject, project_generator::default_project_config::{MainFileLanguage, ProjectType, get_default_project_config}};

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

pub fn create_project_at(new_project_root: &str) -> io::Result<Option<RawProject>> {
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

    let default_prefix = new_project_root
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

    // TODO: Refactor this, and probably the rest of the function honestly.
    let lang_selection: MainFileLanguage = prompt_until(
      "1: C\n2: C++\nChoose Language (1 or 2): ",
      |result| if let PromptResult::Custom(value) = result {
        match value.as_str() {
          "1" | "C" => true,
          "2" | "C++" => true,
          _ => false
        }
      } else { false }
    )?
      .custom_into(|value| match value.as_str() {
        "1" | "C" => MainFileLanguage::C,
        "2" | "C++" => MainFileLanguage::Cpp,
        _ => MainFileLanguage::Cpp
      });

    let project_info = get_default_project_config(
      &project_root,
      &include_prefix,
      lang_selection,
      // TODO: Prompt for project type
      ProjectType::Executable
    );

    let cmake_data_file = File::create(format!("{}/cmake_data.yaml", project_root.to_str().unwrap()))?;

    match serde_yaml::to_writer(&cmake_data_file, &project_info) {
      Ok(_) => println!("Successfully wrote cmake_data.yaml"),
      Err(err) => return Err(io::Error::new(ErrorKind::Other, err))
    }

    // TODO: Write main file
    return Ok(Some(project_info));
  }

  Ok(None)
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
