use std::collections::{HashMap, HashSet};
use regex::Regex;
use serde::{Serialize, Deserialize};

use super::{raw_project_in::{RawCompiledItem, RawProject, BuildType}, PreBuildConfigIn, SingleLanguageConfig, LanguageConfigMap};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawSubproject {
  // TODO: Remove subproject name. A subproject's project name should be its directory name
  // prefixed with the root project's name.
  name: String,
  // If possible, should be the same as the project name
  include_prefix: String,
  description: String,
  version: String,
  prebuild_config: Option<PreBuildConfigIn>,
  output: HashMap<String, RawCompiledItem>,
  subprojects: Option<HashSet<String>>,
  // pub predefined_dependencies: Option<PredefinedDepMap>,
  // pub gcmake_dependencies: Option<GCMakeDepMap>
}

impl From<RawProject> for RawSubproject {
  fn from(project_data: RawProject) -> Self {
    Self {
      name: project_data.name,
      include_prefix: project_data.include_prefix,
      description: project_data.description,
      version: project_data.version,
      prebuild_config: project_data.prebuild_config,
      output: project_data.output,
      subprojects: project_data.subprojects
    }
  }
}

impl Into<RawProject> for RawSubproject {
  fn into(self) -> RawProject {
    RawProject {
      name: self.name.clone(),
      // If possible, should be the same as the project name
      include_prefix: self.include_prefix,
      description: self.description,
      version: self.version,
      vendor: self.name,
      // NOTE: This language config is only a placeholder. Subprojects will inherit
      // language info from their parent project.
      languages: LanguageConfigMap {
        c: SingleLanguageConfig {
          standard: 11
        },
        cpp: SingleLanguageConfig {
          standard: 17
        }
      },
      // Placeholder, no meaning
      test_framework: None,
      // Placeholder, no meaning
      supported_compilers: HashSet::new(),
      // Placeholder, no meaning
      default_build_type: BuildType::Debug,
      prebuild_config: self.prebuild_config,
      // Build configs are also inherited from the parent project.
      build_configs: HashMap::new(),
      // Placeholder, no meaning
      global_defines: None,
      output: self.output,
      subprojects: self.subprojects,
      predefined_dependencies: None,
      gcmake_dependencies: None
    }
  }
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawTestProject {
  description: String,
  version: String,
  prebuild_config: Option<PreBuildConfigIn>,
  output: HashMap<String, RawCompiledItem>
}

impl RawTestProject {
  pub fn into_raw_subproject(self, name: impl AsRef<str>) -> RawSubproject {
    let name_string: String = name.as_ref().to_string();

    return RawSubproject {
      name: name_string.clone(),
      // TODO: Do this, plus additional validation for all include prefixes, not just test project ones.
      include_prefix: Regex::new("[\\- ]").unwrap().replace_all(&name_string, "_").to_uppercase(),
      description: self.description,
      version: self.version,
      prebuild_config: self.prebuild_config,
      output: self.output,
      subprojects: None
    }
  }
}

impl From<RawSubproject> for RawTestProject {
  fn from(raw_subproject: RawSubproject) -> Self {
    return Self {
      description: raw_subproject.description,
      version: raw_subproject.version,
      prebuild_config: raw_subproject.prebuild_config,
      output: raw_subproject.output
    }
  }
}