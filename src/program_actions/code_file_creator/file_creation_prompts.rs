use std::io;

use crate::common::prompt::{prompt_once, prompt_until, PromptResult};

pub fn prompt_for_initial_compiled_lib_file_pair_name(project_name: &str) -> io::Result<String> {
  let default_value: String = format!("{}Impl", project_name);

  return match prompt_once(&format!("Initial file pair name [{}]: ", default_value))? {
    PromptResult::Custom(value) => Ok(value),
    _ => Ok(default_value)
  }
}