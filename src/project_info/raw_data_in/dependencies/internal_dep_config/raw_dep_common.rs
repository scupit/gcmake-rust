use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize};
use super::{RawMutualExclusionSet, RawPredefinedTargetMapIn};
use crate::project_info::raw_data_in::RawFeatureDefault;

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawDebianPackagesConfig {
  pub runtime: Option<BTreeSet<String>>,
  pub dev: Option<BTreeSet<String>>
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawDepConfigOption {
  pub cache_description: Option<String>,
  pub cmake_var: String
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawEmscriptenConfig {
  pub link_flag: Option<String>,
  pub is_internally_supported: Option<bool>,
  pub is_flag_link_time_only: Option<bool>
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawDepFeatureConfig {
  pub default: RawFeatureDefault,
  // Names of other features in the same dependency to transitively enable.
  // Cross-dependency enables are deliberately not supported here;
  // If you need to constrain on other dependencies, use `external_requires` with a constraint
  // expression instead.
  pub enables: Option<BTreeSet<String>>,
  // The actual string used in in the CMake variable defined by list_var.
  // Defaults to the feature's name.
  pub list_value: Option<String>
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawDepFeatureMode {
  // Name of a CMake list variable which receives every enabled feature's list_value
  // before the dependency's hooks and CMakeLists run (e.g. crow's CROW_FEATURES).
  pub list_var: String
}

pub trait RawPredepCommon {
  fn find_module_base_name(&self) -> Option<&str>;

  fn can_trivially_cross_compile(&self) -> bool;
  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet>;
  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn;
  fn repo_url(&self) -> Option<&str>;
  fn github_url(&self) -> Option<&str>;
  fn gcmake_readme_url(&self) -> Option<&str>;
  fn get_emscripten_config(&self) -> Option<&RawEmscriptenConfig>;
  fn supports_emscripten(&self) -> bool;
  fn is_internally_supported_by_emscripten(&self) -> bool;

  fn supports_url_download_method(&self) -> bool;
  fn supports_git_download_method(&self) -> bool;

  fn raw_debian_packages_config(&self) -> Option<&RawDebianPackagesConfig>;
  fn config_options_map(&self) -> Option<&HashMap<String, RawDepConfigOption>>;
  fn features_map(&self) -> Option<&HashMap<String, RawDepFeatureConfig>>;
  fn feature_mode(&self) -> Option<&RawDepFeatureMode>;
}