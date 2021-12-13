pub mod dependency_configs;
mod supported_dependencies;

pub mod dep_in;

use supported_dependencies::DEPENDENCIES_YAML_STRING;
use self::dependency_configs::AllPredefinedDependencies;

pub fn supported_dependency_configs() -> Result<AllPredefinedDependencies, String> {
  return match serde_yaml::from_str::<AllPredefinedDependencies>(DEPENDENCIES_YAML_STRING) {
    Ok(data) => Ok(data),
    Err(serde_error) => Err(serde_error.to_string())
  }
}