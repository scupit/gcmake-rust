use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RawProject {
  name: String,
  // If possible, should be the same as the project name
  include_prefix: String,
  description: String,
  version: String,
  languages: HashSet<String>,
  supported_compilers: HashSet<CompilerSpecifier>,
  output: HashMap<String, RawCompiledItem>
}

impl RawProject {
  pub fn get_include_prefix(&self) -> &str {
    return &self.include_prefix;
  }

  pub fn get_output(&self) -> &HashMap<String, RawCompiledItem> {
    return &self.output;
  }

  pub fn get_name(&self) -> &str {
    return &self.name;
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CompiledItemType {
  Executable,
  StaticLib,
  SharedLib
}

#[derive(Serialize, Deserialize, Hash, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum CompilerSpecifier {
  GCC,
  MSVC,
  Clang
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawCompiledItem {
  output_type: CompiledItemType,
  entry_file: String
}

impl RawCompiledItem {
  pub fn get_entry_file(&self) -> &str {
    return &self.entry_file;
  }

  pub fn get_output_type(&self) -> &CompiledItemType {
    return &self.output_type;
  }
}
