use std::collections::{HashMap, BTreeSet, BTreeMap};
use serde::{Serialize, Deserialize};

use super::{raw_project_in::{RawCompiledItem, RawProject, BuildType}, PreBuildConfigIn, SingleLanguageConfig, LanguageConfigMap};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawSubproject {
  // If possible, should be the same as the project name
  include_prefix: String,
  description: String,
  version: String,
  prebuild_config: Option<PreBuildConfigIn>,
  output: HashMap<String, RawCompiledItem>
}

impl From<RawProject> for RawSubproject {
  fn from(project_data: RawProject) -> Self {
    Self {
      include_prefix: project_data.include_prefix,
      description: project_data.description,
      version: project_data.version,
      prebuild_config: project_data.prebuild_config,
      output: project_data.output
    }
  }
}

impl Into<RawProject> for RawSubproject {
  fn into(self) -> RawProject {
    let placeholder_name: String = String::from("WHEN CONVERTING A RAW SUBPROJECT INTO A RAW PROJECT, THIS SHOULD BE IGNORED");

    RawProject {
      // If possible, should be the same as the project name
      name: placeholder_name.clone(),
      vendor: placeholder_name,
      include_prefix: self.include_prefix,
      description: self.description,
      version: self.version,
      features: None,
      installer_config: None,
      documentation: None,
      // NOTE: This language config is only a placeholder. Subprojects will inherit
      // language info from their parent project.
      languages: LanguageConfigMap {
        c: Some(SingleLanguageConfig {
          min_standard: String::from("11"),
          exact_standard: None
        }),
        cpp: Some(SingleLanguageConfig {
          min_standard: String::from("17"),
          exact_standard: None
        })
      },
      // Placeholder, no meaning
      test_framework: None,
      // Placeholder, no meaning
      supported_compilers: BTreeSet::new(),
      // Placeholder, no meaning
      default_build_type: BuildType::Debug,
      prebuild_config: self.prebuild_config,
      // Build configs are also inherited from the parent project.
      build_configs: BTreeMap::new(),
      // Placeholder, no meaning
      global_defines: None,
      global_properties: None,
      output: self.output,
      predefined_dependencies: None,
      gcmake_dependencies: None
    }
  }
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawTestProject {
  include_prefix: String,
  description: String,
  version: String,
  prebuild_config: Option<PreBuildConfigIn>,
  output: HashMap<String, RawCompiledItem>
}

impl RawTestProject {
  pub fn into_raw_subproject(self) -> RawSubproject {
    return RawSubproject {
      include_prefix: self.include_prefix,
      description: self.description,
      version: self.version,
      prebuild_config: self.prebuild_config,
      output: self.output
    }
  }
}

impl From<RawSubproject> for RawTestProject {
  fn from(raw_subproject: RawSubproject) -> Self {
    return Self {
      include_prefix: raw_subproject.include_prefix,
      description: raw_subproject.description,
      version: raw_subproject.version,
      prebuild_config: raw_subproject.prebuild_config,
      output: raw_subproject.output
    }
  }
}