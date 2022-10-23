use std::collections::{HashSet, BTreeSet};

use crate::project_info::raw_data_in::dependencies::internal_dep_config::raw_dep_common::{RawEmscriptenConfig, RawDebianPackagesConfig};

use super::final_target_map_common::FinalTargetConfigMap;

#[derive(Clone)]
pub struct FinalDebianPackagesConfig {
  pub runtime: BTreeSet<String>,
  pub dev: BTreeSet<String>
}

impl FinalDebianPackagesConfig {
  pub fn make_from(maybe_config: Option<&RawDebianPackagesConfig>) -> Self {
    return match maybe_config {
      None => FinalDebianPackagesConfig { runtime: BTreeSet::new(), dev: BTreeSet::new() },
      Some(pack_config) => FinalDebianPackagesConfig {
        runtime: pack_config.runtime.clone().unwrap_or_default(),
        dev: pack_config.dev.clone().unwrap_or_default(),
      }
    }
  }

  pub fn has_packages(&self) -> bool {
    return !(self.runtime.is_empty() || self.dev.is_empty());
  }
}

pub trait PredefinedDepFunctionality {
  fn can_cross_compile(&self) -> bool;
  fn get_target_config_map(&self) -> &FinalTargetConfigMap;
  fn target_name_set(&self) -> HashSet<String>;
  fn supports_emscripten(&self) -> bool;
  fn raw_emscripten_config(&self) -> Option<&RawEmscriptenConfig>;
  fn uses_emscripten_link_flag(&self) -> bool;
  fn is_internally_supported_by_emscripten(&self) -> bool;
  fn debian_packages_config(&self) -> &FinalDebianPackagesConfig;
}