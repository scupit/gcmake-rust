pub mod internal_dep_config;
mod supported_dependencies;

pub mod user_given_dep_config;

use supported_dependencies::DEPENDENCIES_YAML_STRING;
use self::internal_dep_config::AllPredefinedDependencies;

pub fn supported_dependency_configs() -> Result<AllPredefinedDependencies, String> {
  return match serde_yaml::from_str::<AllPredefinedDependencies>(DEPENDENCIES_YAML_STRING) {
    Ok(data) => Ok(data),
    Err(serde_error) => Err(serde_error.to_string())
  }
}