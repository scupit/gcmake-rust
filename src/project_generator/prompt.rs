use std::{io::{self, stdin, Write}, convert::TryInto};

use super::configuration::{MainFileLanguage, CreationProjectOutputType, OutputLibType};

#[derive(Debug)]
pub enum PromptResult {
  Yes,
  No,
  Custom(String),
  Empty
}

impl PromptResult {
  pub fn unwrap_or(self, empty_replacement: String) -> String {
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


pub fn prompt_once(prompt: &str) -> io::Result<PromptResult> {
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

pub fn prompt_until_boolean(prompt: &str) -> io::Result<PromptResult> {
  prompt_until(prompt, |result| result.is_yes_or_no())
}

pub fn prompt_until_value(prompt: &str) -> io::Result<PromptResult> {
  prompt_until(prompt, |result| result.is_custom())
}

type PromptChoice<'a, T> = (&'a str, Box<dyn Fn() -> T>);

fn prompt_with_choices<T>(
  prompt_title: &str,
  choices: &[PromptChoice<T>]
) -> io::Result<T> {
  let choice_list_string: String = choices
    .iter()
    .enumerate()
    .map(|(index, (choice_name, _))|
      format!("{}: {}\n", index + 1, choice_name)
    )
    .collect();
    

  let valid_result: PromptResult = prompt_until(
    &format!("{}{}: ", choice_list_string, prompt_title),
    |prompt_result| {
      if let PromptResult::Custom(value) = prompt_result {
        match value.parse::<usize>() {
          Ok(value_int) => value_int > 0 && value_int <= choices.len().try_into().unwrap(),
          _ => false
        }
      }
      else { false }
    }
  )?;

  let result_index: usize = valid_result.unwrap_custom().parse::<usize>().unwrap() - 1;
  let (_, value_resolver) = &choices[result_index];

  return Ok(value_resolver());
}

pub fn prompt_for_language() -> io::Result<MainFileLanguage> {
  return prompt_with_choices(
    "Choose Language",
    &[
      ("C", Box::new(|| MainFileLanguage::C)),
      ("C++", Box::new(|| MainFileLanguage::Cpp))
    ]
  );
}

pub fn prompt_for_project_output_type() -> io::Result<CreationProjectOutputType> {
  return prompt_with_choices(
    "Choose Project Type",
    &[
      ("Executable", Box::new(|| CreationProjectOutputType::Executable)),
      ("Library", Box::new(|| {
        let mut lib_output_type: Option<OutputLibType> = None;

        loop {
          if let Ok(lib_type) = prompt_for_lib_output_type() {
            lib_output_type = Some(lib_type);
          }

          if let Some(lib_type) = lib_output_type {
            break CreationProjectOutputType::Library(lib_type)
          }
        }
      })),
    ]
  );
}

fn prompt_for_lib_output_type() -> io::Result<OutputLibType> {
  return prompt_with_choices(
    "Choose library type",
    &[
      ("Compiled (either static or shared)", Box::new(|| OutputLibType::ToggleStaticOrShared)),
      ("Static", Box::new(|| OutputLibType::Static)),
      ("Shared", Box::new(|| OutputLibType::Shared)),
      ("Header-Only", Box::new(|| OutputLibType::HeaderOnly))
    ]
  );
}

pub fn prompt_for_description() -> io::Result<String> {
  Ok(prompt_until_value("Description: ")?.unwrap_custom())
}