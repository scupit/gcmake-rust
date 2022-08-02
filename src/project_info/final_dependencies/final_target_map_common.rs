use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::FromIterator;

use crate::project_info::raw_data_in::dependencies::internal_dep_config::raw_dep_common::RawPredepCommon;
use crate::project_info::raw_data_in::dependencies::internal_dep_config::{RawPredefinedTargetMapIn, RawMutualExclusionSet};

#[derive(Clone)]
pub enum FinalRequirementSpecifier {
  Single(String),
  OneOf(Vec<String>),
  MutuallyExclusive(String)
}

impl FinalRequirementSpecifier {
  fn new_inclusive(lib_names: HashSet<String>) -> Self {
    let mut name_vec: Vec<String> = lib_names.into_iter().collect();
    name_vec.sort();

    return match name_vec.len() {
      0 => panic!("Tried to create a FinalRequirementSpecifier from an empty name set."),
      1 => Self::Single(name_vec.get(0).unwrap().to_string()),
      _ => Self::OneOf(name_vec)
    }
  }

  fn new_exclusive(lib_name: String) -> Self {
    return Self::MutuallyExclusive(lib_name);
  }
}

impl Hash for FinalRequirementSpecifier {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Self::Single(lib_name) => lib_name.hash(state),
      Self::OneOf(lib_set) => lib_set.hash(state),
      // Hash the exclusion lib names in reverse to avoid collisions with Self::Single
      Self::MutuallyExclusive(lib_name) => lib_name.chars().rev().collect::<String>().hash(state)
    }
  }
}

impl PartialEq for FinalRequirementSpecifier {
  fn eq(&self, other: &Self) -> bool {
    return match (self, other) {
      (Self::Single(lib_name), Self::Single(other_lib_name)) => lib_name == other_lib_name,
      (Self::OneOf(lib_set), Self::OneOf(other_lib_set)) => {
        HashSet::<String>::from_iter(lib_set.clone())
          .difference(&HashSet::<String>::from_iter(other_lib_set.clone()))
          .collect::<Vec<_>>()
          .is_empty()
      },
      (Self::MutuallyExclusive(lib_name), Self::MutuallyExclusive(other_lib_name)) => lib_name == other_lib_name,
      _ => false
    }
  }
}

impl Eq for FinalRequirementSpecifier { }

#[derive(Clone)]
pub struct FinalTargetConfig {
  pub requirements_set: HashSet<FinalRequirementSpecifier>,
  pub cmakelists_name: String,
  pub cmake_yaml_name: String
}

pub type FinalTargetConfigMap = HashMap<String, FinalTargetConfig>;

pub fn make_final_target_config_map(
  dep_name: &str,
  dep_info: &dyn RawPredepCommon
  // mutual_exclusion_set: &Option<RawMutualExclusionSet>
) -> Result<FinalTargetConfigMap, String> {
  let mut final_map = FinalTargetConfigMap::new();
  let raw_target_config_map: &RawPredefinedTargetMapIn = dep_info.raw_target_map_in();

  for (target_name, raw_target_config) in raw_target_config_map {
    let mut requirements_set: HashSet<FinalRequirementSpecifier> = HashSet::new();

    if let Some(interdependent_requirement_specifier) = &raw_target_config.requires {
      for full_specifier in interdependent_requirement_specifier {
        let given_lib_names = parse_specifier(full_specifier);

        let maybe_err_msg: Option<String> = verify_requirement_spec(
          &given_lib_names,
          raw_target_config_map,
          full_specifier,
          target_name,
          dep_name
        );

        if let Some(err_message) = maybe_err_msg {
          return Err(err_message);
        }

        assert!(
          !given_lib_names.is_empty(),
          "The list of library names given to a valid requirement specifier should never be empty."
        );

        requirements_set.insert(FinalRequirementSpecifier::new_inclusive(given_lib_names));
      }
    }

    final_map.insert(
      target_name.to_string(),
      FinalTargetConfig {
        requirements_set,
        cmake_yaml_name: target_name.clone(),
        cmakelists_name: match &raw_target_config.actual_target_name {
          Some(name) => name.clone(),
          None => target_name.clone()
        }
      }
    );
  }

  if let Some(mutual_exclusion_specs) = dep_info.maybe_mutual_exclusion_groups() {
    for exclusion_spec in mutual_exclusion_specs {
      let exclusion_group: HashSet<String> = parse_mutual_exclusion(exclusion_spec);

      let maybe_err_msg: Option<String> = verify_exclusion_spec(
        &exclusion_group,
        raw_target_config_map,
        exclusion_spec,
        dep_name
      );

      if let Some(err_msg) = maybe_err_msg {
        return Err(err_msg);
      }

      for lib_name in &exclusion_group {
        let target_config: &mut FinalTargetConfig = final_map.get_mut(lib_name).unwrap();

        for other_lib_name in &exclusion_group {
          if lib_name != other_lib_name {
            target_config.requirements_set.insert(
              FinalRequirementSpecifier::new_exclusive(other_lib_name.to_string())
            );
          }
        }
      }
    }
  }

  return Ok(final_map);
}

fn verify_requirement_spec(
  given_lib_names: &HashSet<String>,
  raw_target_config_map: &RawPredefinedTargetMapIn,
  full_specifier: &str,
  target_name: &str,
  dep_name: &str
) -> Option<String> {
  if given_lib_names.is_empty() {
    return Some(format!(
      "Target \"{}\" in predefined dependency \"{}\" lists an empty requirement. Each requirement specifier name must contain some text.",
      target_name,
      dep_name
    ));
  }

  for requirement_name in given_lib_names {
    if !raw_target_config_map.contains_key(requirement_name) {
      return Some(format!(
        "Target \"{}\" in predefined dependency \"{}\" lists \"{}\" in a requirement specifier ({}), but \"{}\" does not have a target called \"{}\".",
        target_name,
        dep_name,
        requirement_name,
        full_specifier,
        dep_name,
        requirement_name
      ));
    }
  }

  return None;
}

fn verify_exclusion_spec(
  given_lib_names: &HashSet<String>,
  raw_target_config_map: &RawPredefinedTargetMapIn,
  full_exclusion_specifier: &str,
  dep_name: &str
) -> Option<String> {
  if given_lib_names.is_empty() {
    return Some(format!(
      "Predefined dependency \"{}\" lists an empty 'mutual exclusion' specifier group. Each mutual exclusion group must contain 2 or more library names.",
      dep_name
    ));
  }
  else if given_lib_names.len() == 1 {
    return Some(format!(
      "Predefined dependency \"{}\" mutual exclusion group (\"{}\") only lists one library. Each mutual exclusion group must contain 2 or more library names.",
      dep_name,
      full_exclusion_specifier
    ));
  }

  for lib_name in given_lib_names {
    if !raw_target_config_map.contains_key(lib_name) {
      return Some(format!(
        "Predefined dependency \"{}\" lists \"{}\" in a mutual exclusion specifier ({}), but \"{}\" does not have a target called \"{}\".",
        dep_name,
        lib_name,
        full_exclusion_specifier,
        dep_name,
        lib_name
      ));
    }
  }

  return None;
}


fn parse_specifier(spec_str: &str) -> HashSet<String> {
  // This is fine for now, but should be made more robust if the specifiers become more complicated.
  return spec_str.split(" or ")
    .map(|lib_name| lib_name.trim().to_string())
    .filter(|lib_name| !lib_name.is_empty())
    .collect();
}

fn parse_mutual_exclusion(exclusion_str: &str) -> HashSet<String> {
  return exclusion_str.split(',')
    .map(|lib_name| lib_name.trim().to_string())
    .filter(|lib_name| !lib_name.is_empty())
    .collect();
}