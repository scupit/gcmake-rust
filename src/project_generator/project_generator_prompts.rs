use std::io;

use crate::common::prompt::{prompt_until_value, prompt_with_choices, prompt_until, PromptResult};

use super::{MainFileLanguage, CreationProjectOutputType, OutputLibType};

pub fn prompt_for_vendor() -> io::Result<String> {
  return prompt_until_value("Vendor name (your name or organization): ")
    .map(|the_result| the_result.unwrap_custom());
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

pub fn prompt_for_needs_custom_main() -> io::Result<bool> {
  let final_answer = prompt_until(
    "Does this test need to provide its own main? (y or n) [n]: ",
    |answer| match answer {
      PromptResult::Custom(_) => false,
      _ => true
    }
  )?;
  
  return match final_answer {
    PromptResult::Custom(_) => unreachable!("Input is constrained to be anything but a 'custom' value."),
    PromptResult::Yes => Ok(true),
    PromptResult::No => Ok(false),
    PromptResult::Empty => Ok(false)
  }
}
