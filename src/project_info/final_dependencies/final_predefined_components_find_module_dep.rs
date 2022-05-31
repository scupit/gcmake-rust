use std::collections::HashSet;

use crate::project_info::raw_data_in::dependencies::{internal_dep_config::{RawBuiltinComponentsFindModuleDep, ComponentsFindModuleLinks, ComponentsFindModuleUsage, UsageMode}, user_given_dep_config::{self, UserGivenPredefinedDependencyConfig}};

struct OrganizedComponents {
  available: HashSet<String>,
  used_component_set: HashSet<String>,
  used_ordered_components: Vec<String>
}

impl OrganizedComponents {
  fn new(available_components: &HashSet<String>) -> Self {
    Self {
      available: available_components.clone(),
      used_component_set: HashSet::new(),
      // CMake component inclusion order is important for some projects (ex: wxWidgets)
      // and compilers (ex: MinGW)
      used_ordered_components: Vec::new()
    }
  }
}

pub struct PredefinedComponentsFindModuleDep {
  raw_dep: RawBuiltinComponentsFindModuleDep,
  components: OrganizedComponents
}

impl PredefinedComponentsFindModuleDep {
  pub fn web_links(&self) -> &ComponentsFindModuleLinks {
    &self.raw_dep.links
  }

  pub fn name_of_find_module(&self) -> &str {
    &self.raw_dep.find_module_name
  }

  pub fn has_component_named(&self, name_searching: &str) -> bool {
    self.components.available.contains(name_searching)
  }

  pub fn get_ordered_used_components(&self) -> &Vec<String> {
    &self.components.used_ordered_components
  }

  pub fn found_varname(&self) -> &str {
    &self.raw_dep.cmakelists_usage.found_var
  }

  pub fn linkable_string(&self) -> String {
    match &self.raw_dep.cmakelists_usage.link_format {
      UsageMode::Target => self.raw_dep.cmakelists_usage.link_value.to_string(),
      UsageMode::Variable => format!(
        "${{{}}}",
        &self.raw_dep.cmakelists_usage.link_value
      )
    }
  }

  pub fn mark_multiple_components_used(
    &mut self,
    dep_name: &str,
    component_names: impl Iterator<Item=impl AsRef<str>>
  ) -> Result<(), String> {
    for name in component_names {
      self.mark_component_used(dep_name, name.as_ref())?;
    }
    Ok(())
  }

  pub fn mark_component_used(
    &mut self,
    dep_name: &str,
    component_name: &str,
  ) -> Result<(), String> {
    if !self.components.available.contains(component_name) {
      return Err(format!(
        "Component '{}' not found in dependency '{}'.",
        component_name,
        dep_name
      ));
    }

    if !self.components.used_component_set.contains(component_name) {
      self.components.used_component_set.insert(component_name.to_string());
      self.components.used_ordered_components.push(component_name.to_string());
    }

    Ok(())
  }

  pub fn from_components_find_module_dep(
    dep: &RawBuiltinComponentsFindModuleDep,
    user_given_dep_config: &UserGivenPredefinedDependencyConfig,
    dep_name: &str
  ) -> Self {
    Self {
      components: OrganizedComponents::new(&dep.components),
      raw_dep: dep.clone()
    }
  }
}