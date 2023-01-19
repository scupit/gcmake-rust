use std::collections::{HashMap, HashSet, BTreeSet, BTreeMap};
use enum_iterator::Sequence;
use serde::{Serialize, Deserialize};

use super::{dependencies::user_given_dep_config::{UserGivenPredefinedDependencyConfig}, project_common_types::{PredefinedDepMap, GCMakeDepMap}};

pub type BuildTypeOptionMap = BTreeMap<BuildConfigCompilerSpecifier, RawBuildConfig>;
pub type BuildConfigMap = BTreeMap<BuildType, BuildTypeOptionMap>;
pub type TargetBuildConfigMap = BTreeMap<TargetSpecificBuildType, BuildTypeOptionMap>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LanguageConfigMap {
  pub c: SingleLanguageConfig,
  pub cpp: SingleLanguageConfig
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum RawTestFramework {
  #[serde(rename = "catch2")]
  Catch2(UserGivenPredefinedDependencyConfig),
  #[serde(rename = "googletest")]
  GoogleTest(UserGivenPredefinedDependencyConfig),
  #[serde(rename = "doctest")]
  DocTest(UserGivenPredefinedDependencyConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum RawDocGeneratorName {
  Doxygen,
  Sphinx
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawDocumentationGeneratorConfig {
  pub generator: RawDocGeneratorName,
  pub headers_only: Option<bool>
}

impl RawTestFramework {
  pub fn lib_config(&self) -> &UserGivenPredefinedDependencyConfig {
    match self {
      Self::Catch2(lib) => lib,
      Self::GoogleTest(lib) => lib,
      Self::DocTest(lib) => lib
    }
  }

  pub fn name(&self) -> &str {
    match self {
      Self::Catch2(_) => "catch2",
      Self::DocTest(_) => "doctest",
      Self::GoogleTest(_) => "googletest"
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawShortcutConfig {
  pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawInstallerConfig {
  pub title: Option<String>,
  pub description: Option<String>,
  pub name_prefix: Option<String>,
  pub shortcuts: Option<HashMap<String, RawShortcutConfig>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum DefaultCompiledLibType {
  Static,
  Shared
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawGlobalPropertyConfig {
  pub ipo_enabled_by_default: Option<bool>,
  pub default_compiled_lib_type: Option<DefaultCompiledLibType>,
  pub are_language_extensions_enabled: Option<bool>
  // TODO: Add option for setting default Emscripten mode.
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawFeatureConfig {
  pub default: bool,
  pub enables: Option<HashSet<String>>
}

fn make_none<T>() -> Option<T> { None }

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawProject {
  pub name: String,
  // If possible, should be the same as the project name
  pub include_prefix: String,
  pub description: String,
  pub vendor: String,
  pub version: String,
  pub installer_config: Option<RawInstallerConfig>,
  pub default_build_type: BuildType,
  pub languages: LanguageConfigMap,
  pub documentation: Option<RawDocumentationGeneratorConfig>,
  pub features: Option<HashMap<String, RawFeatureConfig>>,
  pub prebuild_config: Option<PreBuildConfigIn>,
  pub supported_compilers: BTreeSet<SpecificCompilerSpecifier>,

  // See https://github.com/dtolnay/serde-yaml/pull/300
  #[serde(
    with = "serde_yaml::with::singleton_map",
    default = "make_none"
  )]
  pub test_framework: Option<RawTestFramework>,

  pub output: HashMap<String, RawCompiledItem>,
  pub global_defines: Option<Vec<String>>,
  pub global_properties: Option<RawGlobalPropertyConfig>,
  pub predefined_dependencies: Option<PredefinedDepMap>,
  pub gcmake_dependencies: Option<GCMakeDepMap>,
  pub build_configs: BuildConfigMap
}

impl RawProject {
  pub fn get_include_prefix(&self) -> &str {
    &self.include_prefix
  }

  pub fn get_name(&self) -> &str {
    &self.name
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
  pub build_config: Option<TargetBuildConfigMap>,
  pub generated_code: Option<BTreeSet<String>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SingleLanguageConfig {
  // TODO: Use a string instead of just an integer.
  pub standard: i8
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, PartialOrd, Ord, Sequence)]
#[serde(deny_unknown_fields)]
pub enum BuildType {
  Debug,
  Release,
  MinSizeRel,
  RelWithDebInfo
}

impl BuildType {
  pub fn name_str(&self) -> &'static str {
    match self {
      Self::Debug => "Debug",
      Self::Release => "Release",
      Self::MinSizeRel => "MinSizeRel",
      Self::RelWithDebInfo => "RelWithDebInfo"
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub enum BuildConfigCompilerSpecifier {
  AllCompilers,
  GCC,
  Clang,
  MSVC,
  Emscripten
}

impl BuildConfigCompilerSpecifier {
  pub fn to_specific(&self) -> Option<SpecificCompilerSpecifier> {
    return match *self {
      Self::AllCompilers => None,
      Self::GCC => Some(SpecificCompilerSpecifier::GCC),
      Self::Clang => Some(SpecificCompilerSpecifier::Clang),
      Self::MSVC => Some(SpecificCompilerSpecifier::MSVC),
      Self::Emscripten => Some(SpecificCompilerSpecifier::Emscripten)
    }
  }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawBuildConfig {
  pub compiler_flags: Option<Vec<String>>,
  pub link_time_flags: Option<Vec<String>>,
  pub linker_flags: Option<Vec<String>>,
  pub defines: Option<Vec<String>>
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
  Clang,
  MSVC,
  Emscripten
}

impl SpecificCompilerSpecifier {
  pub fn name_string(&self) -> &str {
    return match *self {
      Self::GCC => "GCC",
      Self::Clang => "Clang",
      Self::MSVC => "MSVC",
      Self::Emscripten => "Emscripten"
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
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
      other => other.to_general_build_type().unwrap().name_str()
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
  pub requires_custom_main: Option<bool>, // Used for tests executables only
  pub output_type: OutputItemType,
  pub entry_file: String,
  pub windows_icon: Option<String>, 
  pub emscripten_html_shell: Option<String>,
  pub defines: Option<Vec<String>>,
  pub build_config: Option<TargetBuildConfigMap>,
  pub link: Option<LinkSection>
}

impl RawCompiledItem {
  pub fn get_output_type(&self) -> &OutputItemType {
    return &self.output_type;
  }
}