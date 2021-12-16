use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use super::dependencies::user_given_dep_config::UserGivenPredefinedDependencyConfig;

pub type BuildTypeOptionMap = HashMap<BuildConfigCompilerSpecifier, BuildConfig>;
pub type BuildConfigMap = HashMap<BuildType, BuildTypeOptionMap>;
pub type LanguageMap = HashMap<ImplementationLanguage, LanguageConfig>;

pub trait ProjectLike {
  fn get_name(&self) -> &str;
  fn get_include_prefix(&self) -> &str;
  fn get_description(&self) -> &str;
  fn get_version(&self) -> &str;
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawProject {
  pub name: String,
  // If possible, should be the same as the project name
  pub include_prefix: String,
  pub description: String,
  pub version: String,
  pub default_build_type: BuildType,
  pub languages: LanguageMap,
  pub prebuild_config: Option<PreBuildConfigIn>,
  pub supported_compilers: HashSet<CompilerSpecifier>,
  pub output: HashMap<String, RawCompiledItem>,
  pub global_defines: Option<HashSet<String>>,
  pub subprojects: Option<HashSet<String>>,
  pub predefined_dependencies: Option<HashMap<String, UserGivenPredefinedDependencyConfig>>,
  pub build_configs: BuildConfigMap,
}

impl ProjectLike for RawProject {
  fn get_include_prefix(&self) -> &str {
    &self.include_prefix
  }

  fn get_name(&self) -> &str {
    &self.name
  }

  fn get_description(&self) -> &str {
    &self.description
  }

  fn get_version(&self) -> &str {
    &self.version
  }
}

impl RawProject {
  pub fn get_subproject_dirnames(&self) -> &Option<HashSet<String>> {
    &self.subprojects
  }

  pub fn get_output(&self) -> &HashMap<String, RawCompiledItem> {
    return &self.output;
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

  pub fn get_global_defines(&self) -> &Option<HashSet<String>> {
    &self.global_defines
  }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
#[serde(deny_unknown_fields)]
pub enum ImplementationLanguage {
  C,
  Cpp
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct PreBuildConfigIn {
  pub link: Option<Vec<String>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LanguageConfig {
  pub default_standard: i8,
  pub allowed_standards: HashSet<i8>
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

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
#[serde(deny_unknown_fields)]
pub enum BuildType {
  Debug,
  Release,
  MinSizeRel,
  RelWithDebInfo
}

impl BuildType {
  pub fn name_string(&self) -> &'static str {
    match self {
      Self::Debug => "Debug",
      Self::Release => "Release",
      Self::MinSizeRel => "MinSizeRel",
      Self::RelWithDebInfo => "RelWithDebInfo"
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
#[serde(deny_unknown_fields)]
pub enum BuildConfigCompilerSpecifier {
  All,
  GCC,
  MSVC,
  Clang
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BuildConfig {
  pub flags: Option<HashSet<String>>,
  pub defines: Option<HashSet<String>>
}


#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum CompiledItemType {
  Executable,
  Library,
  StaticLib,
  SharedLib
}

#[derive(Serialize, Deserialize, Hash, Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub enum CompilerSpecifier {
  GCC,
  MSVC,
  Clang
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawCompiledItem {
  pub output_type: CompiledItemType,
  pub entry_file: String,
  // Link order can be important. Eventually figure out how to make/use an ordered Set.
  // Single format: subproject_name::lib_name
  // Multiple format: subproject_name::{lib_name, another_lib_name}

  // The link format is namespaced like rust imports. subproject_name is the name of 
  // the library project which contains the library linking to. Eventually you will be able
  // to link to items inside dependencies as well, once dependency support is added.
  pub link: Option<Vec<String>>
}

impl RawCompiledItem {
  pub fn get_entry_file(&self) -> &str {
    return &self.entry_file;
  }

  pub fn get_output_type(&self) -> &CompiledItemType {
    return &self.output_type;
  }

  pub fn is_library_type(&self) -> bool {
    match self.output_type {
      CompiledItemType::Library
      | CompiledItemType::SharedLib
      | CompiledItemType::StaticLib => true,
      CompiledItemType::Executable => false
    }
  }

  pub fn is_executable_type(&self) -> bool {
    match self.output_type {
      CompiledItemType::Executable => true,
      _ => false
    }
  }
}