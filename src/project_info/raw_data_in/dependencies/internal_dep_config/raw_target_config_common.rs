use std::collections::{HashSet, HashMap};

use serde::Deserialize;

/*
  Allowed formats:
    - lib-name
    - lib-name or alternative-lib-name
*/
type RequirementSpecifier = String;

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawTargetConfig {
  pub requires: Option<HashSet<RequirementSpecifier>>,
  pub actual_target_name: Option<String>
}

pub type RawPredefinedTargetMapIn = HashMap<String, RawTargetConfig>;

pub type RawMutualExclusionSet = Vec<HashSet<String>>;
