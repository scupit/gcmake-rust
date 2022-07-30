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
pub struct TargetConfig {
  pub requires: Option<HashSet<RequirementSpecifier>>
}

pub type RawPredefinedTargetMapIn = HashMap<String, TargetConfig>;