use std::collections::HashMap;

use super::dependencies::user_given_dep_config::{UserGivenPredefinedDependencyConfig, UserGivenGCMakeProjectDependency};

pub type PredefinedDepMap = HashMap<String, UserGivenPredefinedDependencyConfig>;
pub type GCMakeDepMap = HashMap<String, UserGivenGCMakeProjectDependency>;