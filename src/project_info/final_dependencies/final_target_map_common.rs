use std::collections::{HashMap, HashSet};

use crate::project_info::raw_data_in::dependencies::internal_dep_config::RawPredefinedTargetMapIn;

#[derive(Clone)]
pub struct FinalTargetConfig {
  pub requires: HashSet<String>
}

pub type FinalTargetConfigMap = HashMap<String, FinalTargetConfig>;

pub fn make_final_target_config_map(
  dep_name: &str,
  raw_target_config_map: &RawPredefinedTargetMapIn
) -> Result<FinalTargetConfigMap, String> {
  let mut final_map = FinalTargetConfigMap::new();

  for (target_name, raw_target_config) in raw_target_config_map {
    if let Some(interdependent_requires) = &raw_target_config.requires {
      for required_target_name in interdependent_requires {
        if !raw_target_config_map.contains_key(required_target_name) {
          return Err(format!(
            "Target \"{}\" lists \"{}\" as a required dependency, however the predefined dependency does not have a target called \"{}\".",
            target_name,
            required_target_name,
            required_target_name
          ));
        }
      }
    }

    final_map.insert(
      target_name.to_string(),
      FinalTargetConfig {
        requires: raw_target_config.requires.clone().unwrap_or(HashSet::new())
      }
    );
  }

  return Ok(final_map);
}
