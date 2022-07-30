use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::FromIterator;

use crate::project_info::raw_data_in::dependencies::internal_dep_config::RawPredefinedTargetMapIn;

#[derive(Clone)]
pub enum FinalRequirementSpecifier {
  Single(String),
  OneOf(Vec<String>)
}

impl FinalRequirementSpecifier {
  fn from(lib_names: HashSet<String>) -> Self {
    let mut name_vec: Vec<String> = lib_names.into_iter().collect();
    name_vec.sort();

    return match name_vec.len() {
      0 => panic!("Tried to create a FinalRequirementSpecifier from an empty name set."),
      1 => Self::Single(name_vec.get(0).unwrap().to_string()),
      _ => Self::OneOf(name_vec)
    }
  }
}

impl Hash for FinalRequirementSpecifier {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Self::Single(lib_name) => lib_name.hash(state),
      Self::OneOf(lib_set) => lib_set.hash(state)
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
      _ => false
    }
  }
}

impl Eq for FinalRequirementSpecifier { }

#[derive(Clone)]
pub struct FinalTargetConfig {
  pub requirements_set: HashSet<FinalRequirementSpecifier>
}

pub type FinalTargetConfigMap = HashMap<String, FinalTargetConfig>;

pub fn make_final_target_config_map(
  dep_name: &str,
  raw_target_config_map: &RawPredefinedTargetMapIn
) -> Result<FinalTargetConfigMap, String> {
  let mut final_map = FinalTargetConfigMap::new();

  for (target_name, raw_target_config) in raw_target_config_map {
    let mut requirements_set: HashSet<FinalRequirementSpecifier> = HashSet::new();

    if let Some(interdependent_requirement_specifier) = &raw_target_config.requires {
      for full_specifier in interdependent_requirement_specifier {
        let given_lib_names = parse_specifier(full_specifier);

        if given_lib_names.is_empty() {
          return Err(format!(
            "Target \"{}\" in predefined dependency \"{}\" lists an empty requirement. Each requirement specifier name must contain some text.",
            target_name,
            dep_name
          ));
        }

        for requirement_name in &given_lib_names {
          if !raw_target_config_map.contains_key(requirement_name) {
            return Err(format!(
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

        requirements_set.insert(FinalRequirementSpecifier::from(given_lib_names));
      }
    }

    final_map.insert(
      target_name.to_string(),
      FinalTargetConfig {
        requirements_set
      }
    );
  }

  return Ok(final_map);
}

fn parse_specifier(spec_str: &str) -> HashSet<String> {
  // This is fine for now, but should be made more robust if the specifiers become more complicated.
  return spec_str.split(" or ")
    .map(|lib_name| lib_name.trim().to_string())
    .filter(|lib_name| !lib_name.is_empty())
    .collect();
}