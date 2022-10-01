use std::collections::HashSet;

use crate::project_info::raw_data_in::dependencies::internal_dep_config::raw_dep_common::RawEmscriptenConfig;

use super::final_target_map_common::FinalTargetConfigMap;

pub trait PredefinedDepFunctionality {
  fn can_cross_compile(&self) -> bool;
  fn get_target_config_map(&self) -> &FinalTargetConfigMap;
  fn target_name_set(&self) -> HashSet<String>;
  fn supports_emscripten(&self) -> bool;
  fn raw_emscripten_config(&self) -> Option<&RawEmscriptenConfig>;
  fn uses_emscripten_link_flag(&self) -> bool;
  fn is_internally_supported_by_emscripten(&self) -> bool;
}