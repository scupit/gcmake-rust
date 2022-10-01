use serde::{Deserialize};
use super::{RawMutualExclusionSet, RawPredefinedTargetMapIn};

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawEmscriptenConfig {
  pub link_flag: Option<String>,
  pub is_internally_supported: Option<bool>,
  pub is_flag_link_time_only: Option<bool>
}

pub trait RawPredepCommon {
  fn can_trivially_cross_compile(&self) -> bool;
  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet>;
  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn;
  fn repo_url(&self) -> Option<&str>;
  fn github_url(&self) -> Option<&str>;
  fn get_emscripten_config(&self) -> Option<&RawEmscriptenConfig>;
  fn supports_emscripten(&self) -> bool;
  fn is_internally_supported_by_emscripten(&self) -> bool;
}