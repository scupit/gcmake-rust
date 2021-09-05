use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

pub type BuildTypeOptionMap = HashMap<BuildConfigCompilerSpecifier, BuildConfig>;
pub type BuildConfigMap = HashMap<BuildType, BuildTypeOptionMap>;

pub type LanguageMap = HashMap<ImplementationLanguage, LanguageConfig>;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawProject {
  name: String,
  // If possible, should be the same as the project name
  include_prefix: String,
  description: String,
  version: String,
  languages: LanguageMap,
  supported_compilers: HashSet<CompilerSpecifier>,
  default_build_type: BuildType,
  build_configs: BuildConfigMap,
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

  pub fn get_build_configs(&self) -> &BuildConfigMap {
    &self.build_configs
  }

  pub fn get_default_build_config(&self) -> &BuildType {
    &self.default_build_type
  }

  pub fn get_langauge_info(&self) -> &LanguageMap {
    &self.languages
  }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum ImplementationLanguage {
  C,
  Cpp
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LanguageConfig {
  pub default_standard: i8,
  allowed_standards: HashSet<i8>
}

impl LanguageConfig {
  pub fn get_sorted_standards(&self) -> Vec<String> {
    let mut temp: Vec<i8> = self.allowed_standards
      .iter()
      .map(|num| *num)
      .collect();

    temp.sort();

    return temp
      .iter()
      .map(|standard_num| standard_num.to_string())
      .collect()
  }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum BuildType {
  Debug,
  Release,
  MinSizeRel,
  RelWithDebInfo
}

impl BuildType {
  pub fn name_string(&self) -> &'static str {
    match *self {
      Self::Debug => "Debug",
      Self::Release => "Release",
      Self::MinSizeRel => "MinSizeRel",
      Self::RelWithDebInfo => "RelWithDebInfo"
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum BuildConfigCompilerSpecifier {
  All,
  GCC,
  MSVC,
  Clang
}


#[derive(Serialize, Deserialize, Debug)]
pub struct BuildConfig {
  pub flags: Option<HashSet<String>>,
  pub defines: Option<HashSet<String>>
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
