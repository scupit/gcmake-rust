mod raw_components_builtin_find_module;
mod raw_predefined_subdirectory_dep;
mod raw_builtin_find_module;

pub use raw_predefined_subdirectory_dep::*;
pub use raw_components_builtin_find_module::*;
pub use raw_builtin_find_module::*;

use std::collections::HashMap;
use serde::{Deserialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleRawPredefinedDependencyInfo {
  pub as_subdirectory: Option<RawSubdirectoryDependency>,
  pub cmake_builtin_find_components_module: Option<RawBuiltinComponentsFindModuleDep>,
  pub cmake_builtin_find_module: Option<RawBuiltinFindModuleDep>
}

// Container for all dependency types defined in supported_dependencies.yaml
pub type AllRawPredefinedDependencies = HashMap<String, SingleRawPredefinedDependencyInfo>;
