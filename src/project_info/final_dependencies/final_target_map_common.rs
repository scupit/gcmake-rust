use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::FromIterator;

use crate::project_info::{LinkSpecifier, GivenConstraintSpecParseContext};
use crate::project_info::parsers::general_parser::ParseSuccess;
use crate::project_info::parsers::link_spec_parser::LinkAccessMode;
use crate::project_info::parsers::system_spec::platform_spec_parser::{SystemSpecifierWrapper, parse_leading_constraint_spec};
use crate::project_info::raw_data_in::dependencies::internal_dep_config::raw_dep_common::RawPredepCommon;
use crate::project_info::raw_data_in::dependencies::internal_dep_config::{RawPredefinedTargetMapIn, RawTargetConfig};

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
pub enum FinalExternalRequirementSpecifier {
  OneOf(Vec<LinkSpecifier>)
}

#[derive(Clone)]
pub struct FinalTargetConfig {
  pub requirements_set: HashSet<FinalRequirementSpecifier>,
  pub external_requirements_set: Vec<FinalExternalRequirementSpecifier>,
  pub system_spec_info: SystemSpecifierWrapper,
  pub cmakelists_name: String,
  pub cmake_yaml_name: String
}

pub type FinalTargetConfigMap = HashMap<String, FinalTargetConfig>;

type NameParsedTargetMapIn<'a> = HashMap<String, (Option<SystemSpecifierWrapper>, &'a RawTargetConfig)>;

fn name_parsed_target_map<'a>(
  raw_target_map: &'a RawPredefinedTargetMapIn,
  maybe_valid_feature_list: Option<&Vec<&str>>
) -> Result<NameParsedTargetMapIn<'a>, String> {
  let mut resulting_map = NameParsedTargetMapIn::new();
  
  for (target_name_with_system_spec, raw_target_config) in raw_target_map {

    let maybe_system_spec: Option<SystemSpecifierWrapper>;
    let target_name_only: &str;

    {
      let parsing_context = GivenConstraintSpecParseContext {
        is_before_output_name: false,
        maybe_valid_feature_list
      };

      match parse_leading_constraint_spec(target_name_with_system_spec, parsing_context)? {
        Some(ParseSuccess { value, rest }) => {
          maybe_system_spec = Some(value);
          target_name_only = rest;
        },
        None => {
          maybe_system_spec = None;
          target_name_only = target_name_with_system_spec;
        }
      }
    }

    resulting_map.insert(
      target_name_only.to_string(),
      (maybe_system_spec, raw_target_config)
    );
  }

  return Ok(resulting_map);
}

pub fn make_final_target_config_map(
  dep_name: &str,
  dep_info: &dyn RawPredepCommon,
  valid_feature_list: Option<&Vec<&str>>
) -> Result<FinalTargetConfigMap, String> {
  let mut final_map = FinalTargetConfigMap::new();
  let raw_target_config_map_with_parsed_names: NameParsedTargetMapIn = name_parsed_target_map(
    dep_info.raw_target_map_in(),
    valid_feature_list
  )?;

  for (target_name, (maybe_system_spec, raw_target_config)) in &raw_target_config_map_with_parsed_names {
    let mut requirements_set: HashSet<FinalRequirementSpecifier> = HashSet::new();

    if let Some(interdependent_requirement_specifier_set) = &raw_target_config.requires {
      for full_specifier in interdependent_requirement_specifier_set {
        let given_lib_names = separate_alternate_requirement_spec(full_specifier);

        let maybe_err_msg: Option<String> = verify_requirement_spec(
          &given_lib_names,
          &raw_target_config_map_with_parsed_names,
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

    let mut external_requirements_set: Vec<FinalExternalRequirementSpecifier> = Vec::new();

    if let Some(external_requirement_spec_set) = &raw_target_config.external_requires {
      for full_specifier in external_requirement_spec_set {
        let given_link_specs: Vec<LinkSpecifier> = separate_alternate_requirement_spec(full_specifier)
          .iter()
          .map(|link_spec_str| LinkSpecifier::parse_from(
            link_spec_str,
            LinkAccessMode::UserFacing,
            valid_feature_list
          ))
          .collect::<Result<_, String>>()?;

        if given_link_specs.is_empty() {
          return Err(format!(
            "External requirement specifier \"{}\" must contain at least one link specifier, but contains none.",
            full_specifier
          ));
        }

        for link_spec in &given_link_specs {
          if link_spec.get_target_list().len() > 1 {
            return Err(format!(
              "Each link specifier in a predefined dependency's 'external_requirements' must only contain one library, however the given specifier \"{}\" contains multiple. Specify that as a list of singles instead.",
              link_spec.original_spec_str()
            ));
          }

          if link_spec.get_namespace_queue().len() > 1 {
            return Err(format!(
              "Each link specifier in a predefined dependency's 'external_requirements' must not have nested namespaces, however the given specifier \"{}\" nests namespaces.",
              link_spec.original_spec_str()
            ));
          }
        }

        // I can't check that the referenced library or target exist yet because we only have this
        // dependency's info at this time. Those checks will be done by the dependency graph.
        external_requirements_set.push(FinalExternalRequirementSpecifier::OneOf(given_link_specs));
      }
    }

    final_map.insert(
      target_name.to_string(),
      FinalTargetConfig {
        requirements_set,
        external_requirements_set,
        cmake_yaml_name: target_name.to_string(),
        system_spec_info: maybe_system_spec.clone().unwrap_or_default(),
        cmakelists_name: match &raw_target_config.actual_target_name {
          Some(name) => name.clone(),
          None => target_name.to_string()
        }
      }
    );
  }

  if let Some(mutual_exclusion_specs) = dep_info.maybe_mutual_exclusion_groups() {
    for exclusion_group in mutual_exclusion_specs {
      let maybe_err_msg: Option<String> = verify_exclusion_spec(
        exclusion_group,
        &raw_target_config_map_with_parsed_names,
        &exclusion_set_string(exclusion_group),
        dep_name
      );

      if let Some(err_msg) = maybe_err_msg {
        return Err(err_msg);
      }

      for lib_name in exclusion_group {
        let target_config: &mut FinalTargetConfig = final_map.get_mut(lib_name).unwrap();

        for other_lib_name in exclusion_group {
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
  raw_target_config_map: &NameParsedTargetMapIn,
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
  raw_target_config_map: &NameParsedTargetMapIn,
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


fn separate_alternate_requirement_spec(spec_str: &str) -> HashSet<String> {
  // This is fine for now, but should be made more robust if the specifiers become more complicated.
  return spec_str.split(" or ")
    .map(|lib_name| lib_name.trim().to_string())
    .filter(|lib_name| !lib_name.is_empty())
    .collect();
}

fn exclusion_set_string(exclusion_set: &HashSet<String>) -> String {
  let name_list: String = exclusion_set
    .iter()
    .map(|the_str| &the_str[..])
    .collect::<Vec<&str>>()
    .join(", ");

  return format!("[{}]", name_list);
}