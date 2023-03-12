use std::io;

use colored::Colorize;

use crate::common::prompt::{prompt_with_choices, ChoiceValue, prompt_until_not_empty, prompt_until_boolean_or_default};

use super::{MainFileLanguage, CreationProjectOutputType, OutputLibType};

pub fn prompt_for_vendor() -> io::Result<String> {
  prompt_until_not_empty(format!("{} (your name or organization)", "Vendor name".bright_green()))
}

pub fn prompt_for_language() -> io::Result<MainFileLanguage> {
  return prompt_with_choices(
    "Choose Language".bright_green(),
    &[
      ("C", &ChoiceValue(MainFileLanguage::C)),
      ("C++", &ChoiceValue(MainFileLanguage::Cpp)),
      ("C++2 (CppFront's EXPERIMENTAL .cpp2)", &ChoiceValue(MainFileLanguage::Cpp2))
    ]
  );
}

pub fn prompt_for_project_output_type() -> io::Result<CreationProjectOutputType> {
  return prompt_with_choices(
    "Choose Project Type".bright_green(),
    &[
      ("Executable", &ChoiceValue(CreationProjectOutputType::Executable)),
      ("Library", &|| Ok(CreationProjectOutputType::Library(prompt_for_lib_output_type()?)) ),
    ]
  );
}

fn prompt_for_lib_output_type() -> io::Result<OutputLibType> {
  return prompt_with_choices(
    "Choose library type".bright_green(),
    &[
      ("Compiled (either static or shared)", &ChoiceValue(OutputLibType::ToggleStaticOrShared)),
      ("Static", &ChoiceValue(OutputLibType::Static)),
      ("Shared", &ChoiceValue(OutputLibType::Shared)),
      ("Header-Only", &ChoiceValue(OutputLibType::HeaderOnly))
    ]
  );
}

pub fn prompt_for_description() -> io::Result<String> {
  prompt_until_not_empty("Description".bright_green())
}

pub fn prompt_for_needs_custom_main() -> io::Result<bool> {
  prompt_until_boolean_or_default("Does this test need to provide its own main?".yellow(), false)
}
