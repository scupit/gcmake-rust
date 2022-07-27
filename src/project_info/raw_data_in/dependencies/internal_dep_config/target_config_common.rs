use std::collections::{HashSet, HashMap};

use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TargetConfig {
  pub requires: Option<HashSet<String>>
}

pub type RawPredefinedTargetMapIn = HashMap<String, TargetConfig>;