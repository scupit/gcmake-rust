use std::collections::{HashMap};

use serde::{Deserialize};

use super::{CMakeModuleType, raw_target_config_common::RawPredefinedTargetMapIn, RawMutualExclusionSet, raw_dep_common::{RawPredepCommon, RawEmscriptenConfig, RawDebianPackagesConfig, RawDepConfigOption}};

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ComponentsFindModuleLinks {
  pub gcmake_readme: String,
  pub cmake_find_module: Option<String>,
  pub components_doc: Option<String>
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub enum UsageMode {
  Variable,
  Target
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ComponentsFindModuleUsage {
  pub link_format: UsageMode,
  pub link_value: String,
  pub found_var: String
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawComponentsModuleDep {
  pub links: ComponentsFindModuleLinks,
  pub module_type: CMakeModuleType,
  pub module_name: String,
  pub cmakelists_usage: ComponentsFindModuleUsage,
  pub debian_packages: Option<RawDebianPackagesConfig>,
  pub mutually_exclusive: Option<RawMutualExclusionSet>,
  pub components: RawPredefinedTargetMapIn
}

impl RawPredepCommon for RawComponentsModuleDep {
  fn find_module_base_name(&self) -> Option<&str> {
    Some(&self.module_name)
  }

  fn raw_debian_packages_config(&self) -> Option<&RawDebianPackagesConfig> {
    self.debian_packages.as_ref()
  }

  fn can_trivially_cross_compile(&self) -> bool {
    false
  }

  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet> {
    &self.mutually_exclusive
  }

  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn {
    &self.components
  }

  fn repo_url(&self) -> Option<&str> {
    None
  }

  fn github_url(&self) -> Option<&str> {
    None
  }

  fn gcmake_readme_url(&self) -> Option<&str> {
    Some(&self.links.gcmake_readme)
  }
  
  fn supports_emscripten(&self) -> bool {
    false
  }

  fn get_emscripten_config(&self) -> Option<&RawEmscriptenConfig> {
    None
  }

  fn is_internally_supported_by_emscripten(&self) -> bool {
    false
  }

  fn supports_git_download_method(&self) -> bool {
    false
  }

  fn supports_url_download_method(&self) -> bool {
    false
  }

  fn config_options_map(&self) -> Option<&HashMap<String, RawDepConfigOption>> {
    // TODO: Implement config options for 'Components Find Modules'.
    None
  }
}