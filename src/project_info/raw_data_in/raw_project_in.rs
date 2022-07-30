use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use super::{dependencies::user_given_dep_config::{UserGivenPredefinedDependencyConfig}, project_common_types::{PredefinedDepMap, GCMakeDepMap}};

pub type BuildTypeOptionMap = HashMap<BuildConfigCompilerSpecifier, BuildConfig>;
pub type BuildConfigMap = HashMap<BuildType, BuildTypeOptionMap>;
pub type TargetBuildConfigMap = HashMap<TargetSpecificBuildType, BuildTypeOptionMap>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LanguageConfigMap {
  #[serde(rename = "C")]
  pub c: SingleLanguageConfig,
  #[serde(rename = "Cpp")]
  pub cpp: SingleLanguageConfig
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum RawTestFramework {
  Catch2(UserGivenPredefinedDependencyConfig),
  // GoogleTest(UserGivenPredefinedDependencyConfig),
  // #[serde(rename = "doctest")]
  // DocTest(UserGivenPredefinedDependencyConfig),
}

impl RawTestFramework {
  pub fn lib_config(&self) -> &UserGivenPredefinedDependencyConfig {
    match self {
      Self::Catch2(lib) => lib
    }
  }

  pub fn name(&self) -> &str {
    match self {
      Self::Catch2(_) => "Catch2",
      // Self::DocTest(_) => "doctest",
      // Self::GoogleTest(_) => "GoogleTest"
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawProject {
  pub name: String,
  // If possible, should be the same as the project name
  pub include_prefix: String,
  pub description: String,
  pub vendor: String,
  pub version: String,
  pub default_build_type: BuildType,
  pub languages: LanguageConfigMap,
  pub prebuild_config: Option<PreBuildConfigIn>,
  pub supported_compilers: HashSet<SpecificCompilerSpecifier>,
  pub output: HashMap<String, RawCompiledItem>,
  pub global_defines: Option<HashSet<String>>,
  pub subprojects: Option<HashSet<String>>,
  pub predefined_dependencies: Option<PredefinedDepMap>,
  pub gcmake_dependencies: Option<GCMakeDepMap>,
  pub build_configs: BuildConfigMap,
  pub test_framework: Option<RawTestFramework>
}

impl RawProject {
  pub fn get_subproject_dirnames(&self) -> &Option<HashSet<String>> {
    &self.subprojects
  }

  pub fn get_include_prefix(&self) -> &str {
    &self.include_prefix
  }

  pub fn get_name(&self) -> &str {
    &self.name
  }

  pub fn get_description(&self) -> &str {
    &self.description
  }

  pub fn get_version(&self) -> &str {
    &self.version
  }

  pub fn get_output(&self) -> &HashMap<String, RawCompiledItem> {
    &self.output
  }

  pub fn get_output_mut(&mut self) -> &mut HashMap<String, RawCompiledItem> {
    &mut self.output
  }

  pub fn get_build_configs(&self) -> &BuildConfigMap {
    &self.build_configs
  }

  pub fn get_default_build_config(&self) -> &BuildType {
    &self.default_build_type
  }

  pub fn get_langauge_info(&self) -> &LanguageConfigMap {
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PreBuildConfigIn {
  pub link: Option<Vec<String>>,
  pub build_config: Option<TargetBuildConfigMap>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SingleLanguageConfig {
  pub standard: i8
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, PartialOrd, Ord)]
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

impl BuildConfigCompilerSpecifier {
  pub fn to_specific(&self) -> Option<SpecificCompilerSpecifier> {
    return match *self {
      Self::All => None,
      Self::GCC => Some(SpecificCompilerSpecifier::GCC),
      Self::Clang => Some(SpecificCompilerSpecifier::Clang),
      Self::MSVC => Some(SpecificCompilerSpecifier::MSVC)
    }
  }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BuildConfig {
  pub compiler_flags: Option<HashSet<String>>,
  pub linker_flags: Option<HashSet<String>>,
  pub defines: Option<HashSet<String>>
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum OutputItemType {
  Executable,
  CompiledLib,
  StaticLib,
  SharedLib,
  HeaderOnlyLib
}

impl OutputItemType {
  pub fn name_string(&self) -> &str {
    match *self {
      OutputItemType::Executable => "Executable",
      OutputItemType::CompiledLib => "CompiledLib",
      OutputItemType::StaticLib => "StaticLib",
      OutputItemType::SharedLib => "SharedLib",
      OutputItemType::HeaderOnlyLib => "HeaderOnlyLib",
    }
  }
}

#[derive(Serialize, Deserialize, Hash, Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub enum SpecificCompilerSpecifier {
  GCC,
  MSVC,
  Clang
}

impl SpecificCompilerSpecifier {
  pub fn name_string(&self) -> &str {
    return match *self {
      Self::GCC => "GCC",
      Self::Clang => "Clang",
      Self::MSVC => "MSVC"
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum TargetSpecificBuildType {
  AllConfigs,
  Debug,
  Release,
  MinSizeRel,
  RelWithDebInfo
}

impl TargetSpecificBuildType {
  pub fn to_general_build_type(&self) -> Option<BuildType> {
    match self {
      Self::Debug => Some(BuildType::Debug),
      Self::Release => Some(BuildType::Release),
      Self::MinSizeRel => Some(BuildType::MinSizeRel),
      Self::RelWithDebInfo => Some(BuildType::RelWithDebInfo),
      Self::AllConfigs => None
    }
  }

  pub fn name_string(&self) -> &str {
    return match self {
      Self::AllConfigs => "All",
      other => other.to_general_build_type().unwrap().name_string()
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields, untagged)]
pub enum LinkSection {
  Uncategorized(Vec<String>),
  PublicPrivateCategorized {
    public: Option<Vec<String>>,
    private: Option<Vec<String>>
  }
}

impl LinkSection {
  pub fn add_exe_link(
    &mut self,
    container_name: &str,
    lib_name: &str
  ) {
    let formatted_link: String = format!("{}::{}", container_name, lib_name);

    match self {
      Self::Uncategorized(links) => links.push(formatted_link),
      Self::PublicPrivateCategorized { private, .. } => match private {
        Some(existing_links) => existing_links.push(formatted_link),
        None => {
          *private = Some(vec![formatted_link]);
        }
      }
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawCompiledItem {
  pub output_type: OutputItemType,
  pub entry_file: String,
  pub link: Option<LinkSection>,
  pub build_config: Option<TargetBuildConfigMap>,
  // Used for tests only
  pub requires_custom_main: Option<bool>
}

impl RawCompiledItem {
  pub fn get_entry_file(&self) -> &str {
    return &self.entry_file;
  }

  pub fn get_output_type(&self) -> &OutputItemType {
    return &self.output_type;
  }

  pub fn is_header_only_type(&self) -> bool {
    self.output_type == OutputItemType::HeaderOnlyLib
  }

  pub fn is_library_type(&self) -> bool {
    match self.output_type {
      OutputItemType::CompiledLib
      | OutputItemType::SharedLib
      | OutputItemType::StaticLib
      | OutputItemType::HeaderOnlyLib => true,
      OutputItemType::Executable => false
    }
  }

  pub fn is_executable_type(&self) -> bool {
    match self.output_type {
      OutputItemType::Executable => true,
      _ => false
    }
  }
}