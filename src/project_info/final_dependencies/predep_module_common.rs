use std::collections::HashSet;

use super::final_target_map_common::FinalTargetConfigMap;

pub trait PredefinedDepFunctionality {
  fn get_target_config_map(&self) -> &FinalTargetConfigMap;
  fn target_name_set(&self) -> HashSet<String>;
}