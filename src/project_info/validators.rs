use regex::Regex;

lazy_static! {
  static ref TARGET_NAME_REGEX: Regex = Regex::new("^[-_a-zA-Z0-9]+$").unwrap();
  static ref PROJECT_NAME_REGEX: Regex = Regex::new("^[-_a-zA-Z0-9]+$").unwrap();
  static ref INCLUDE_PREFIX_REGEX: Regex = Regex::new("^[a-zA-Z][-_a-zA-Z0-9]+$").unwrap();
  static ref RELATIVE_CODE_FILE_PATH_REGEX: Regex = Regex::new("^[-_a-zA-Z0-9]+$").unwrap();
}

pub fn is_valid_target_name(name: &str) -> bool {
  return TARGET_NAME_REGEX.is_match(name);
}

pub fn is_valid_project_name(name: &str) -> bool {
  return PROJECT_NAME_REGEX.is_match(name);
}

pub fn is_valid_base_include_prefix(include_prefix: &str) -> bool {
  return INCLUDE_PREFIX_REGEX.is_match(include_prefix);
}

// One use case is validating inputs when generating header/source file pairs.
pub fn is_valid_relative_code_file_path(path_str: &str) -> bool {
  return RELATIVE_CODE_FILE_PATH_REGEX.is_match(path_str);
}
