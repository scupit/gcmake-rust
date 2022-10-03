use std::collections::{HashSet, HashMap};

use serde::Deserialize;

/*
  Allowed formats:
    - lib-name
    - lib-name or alternative-lib-name
*/
type RequirementSpecifier = String;

/*
  Allowed formats:
    - predep-name::lib-name
    - predep-name::lib-name or predep-name::alternate-lib-name
*/
type ExternalRequirementSpecifier = String;

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawTargetConfig {
  pub requires: Option<HashSet<RequirementSpecifier>>,
  pub external_requires: Option<HashSet<ExternalRequirementSpecifier>>,
  pub actual_target_name: Option<String>
}

pub type RawPredefinedTargetMapIn = HashMap<String, RawTargetConfig>;

pub type RawMutualExclusionSet = Vec<HashSet<String>>;
