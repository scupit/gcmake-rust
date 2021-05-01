use std::collections::{HashMap, HashSet};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RawProject {
  name: String,
  version: String,
  languages: HashSet<String>,
  supported_compilers: HashSet<CompilerSpecifier>,
  output: HashMap<String, RawCompiledItem>
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
  output_type: CompiledItemType
}