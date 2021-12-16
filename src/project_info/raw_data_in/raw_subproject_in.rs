use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use super::{dependencies::user_given_dep_config::UserGivenPredefinedDependencyConfig, raw_project_in::{RawCompiledItem, ProjectLike, RawProject, BuildType}, PreBuildConfigIn};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawSubproject {
  name: String,
  // If possible, should be the same as the project name
  include_prefix: String,
  description: String,
  version: String,
  prebuild_config: Option<PreBuildConfigIn>,
  output: HashMap<String, RawCompiledItem>,
  subprojects: Option<HashSet<String>>,
  pub predefined_dependencies: Option<HashMap<String, UserGivenPredefinedDependencyConfig>>
}

impl ProjectLike for RawSubproject {
  fn get_name(&self) -> &str {
    &self.name
  }
  
  fn get_description(&self) -> &str {
    &self.description
  }

  fn get_include_prefix(&self) -> &str {
    &self.include_prefix
  }

  fn get_version(&self) -> &str {
    &self.version
  }
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
      subprojects: project_data.subprojects,
      predefined_dependencies: project_data.predefined_dependencies
    }
  }
}

impl Into<RawProject> for RawSubproject {
  fn into(self) -> RawProject {
    RawProject {
      name: self.name,
      // If possible, should be the same as the project name
      include_prefix: self.include_prefix,
      description: self.description,
      version: self.version,
      languages: HashMap::new(),
      supported_compilers: HashSet::new(),
      default_build_type: BuildType::Debug,
      prebuild_config: self.prebuild_config,
      build_configs: HashMap::new(),
      global_defines: None,
      output: self.output,
      subprojects: self.subprojects,
      predefined_dependencies: self.predefined_dependencies
    }
  }
}