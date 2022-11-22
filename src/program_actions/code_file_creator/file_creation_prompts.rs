use std::io;

use crate::{common::prompt::{prompt_once, prompt_until_satisfies_or_default}, project_info::validators::is_valid_relative_code_file_path};

pub fn prompt_for_initial_compiled_lib_file_pair_name(_project_name: &str) -> io::Result<String> {
  return prompt_until_satisfies_or_default(
    "Initial file pair name",
    is_valid_relative_code_file_path,
    String::from("Placeholder")
  );
}