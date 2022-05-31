use std::collections::HashSet;

use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct InstallationLinks {
  pub prebuilt_downloads: Option<String>,
  pub building: Option<String>
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ComponentsFindModuleLinks {
  pub cmake_find_module: String,
  pub installation: Option<InstallationLinks>,
  pub components_doc: Option<String>
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub enum UsageMode {
  Variable,
  Target
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ComponentsFindModuleUsage {
  pub link_format: UsageMode,
  pub link_value: String,
  pub found_var: String
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawBuiltinComponentsFindModuleDep {
  pub links: ComponentsFindModuleLinks,
  pub cmakelists_usage: ComponentsFindModuleUsage,
  pub components: HashSet<String>
}