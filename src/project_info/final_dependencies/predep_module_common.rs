use std::collections::{HashSet, BTreeSet, HashMap, BTreeMap};

use colored::Colorize;

use crate::project_info::raw_data_in::dependencies::internal_dep_config::raw_dep_common::{RawEmscriptenConfig, RawDebianPackagesConfig, RawDepConfigOption, RawDepFeatureConfig, RawDepFeatureMode};
use crate::project_info::validators::is_valid_lowercase_identifier;
use crate::project_info::{FeatureValidationContext, GivenConstraintSpecParseContext, deny_feature_constraint_on_enabler, insert_union_merged, resolve_feature_default};
use crate::project_info::parsers::general_parser::ParseSuccess;
use crate::project_info::parsers::system_spec::platform_spec_parser::{SystemSpecifierWrapper, parse_leading_constraint_spec};

use super::final_target_map_common::FinalTargetConfigMap;

#[derive(Clone)]
pub struct FinalDebianPackagesConfig {
  pub runtime: BTreeSet<String>,
  pub dev: BTreeSet<String>
}

impl FinalDebianPackagesConfig {
  pub fn make_from(maybe_config: Option<&RawDebianPackagesConfig>) -> Self {
    return match maybe_config {
      None => FinalDebianPackagesConfig { runtime: BTreeSet::new(), dev: BTreeSet::new() },
      Some(pack_config) => FinalDebianPackagesConfig {
        runtime: pack_config.runtime.clone().unwrap_or_default(),
        dev: pack_config.dev.clone().unwrap_or_default(),
      }
    }
  }

  pub fn has_packages(&self) -> bool {
    return !(self.runtime.is_empty() || self.dev.is_empty());
  }
}

#[derive(Clone)]
pub struct FinalDepConfigOption {
  pub cache_description: Option<String>,
  pub cmake_var: String,
  pub value: String
}

pub fn resolve_final_config_options(
  maybe_reference_map: Option<&HashMap<String, RawDepConfigOption>>,
  // TODO: Change the item type once values other than Strings are supported.
  maybe_user_given_map: Option<HashMap<String, String>>
) -> Result<BTreeMap<String, FinalDepConfigOption>, String> {
  match (maybe_reference_map, maybe_user_given_map) {
    (_, None)=> return Ok(BTreeMap::new()),
    (None, Some(in_map)) => {
      if in_map.is_empty() {
        return Ok(BTreeMap::new())
      }
      else {
        return Err(format!(
          "Some config_option(s) were given, however the predefined dependency doesn't allow any configuration options."
        ));
      }
    },
    (Some(reference_map), Some(user_given_map)) => {
      let mut final_map: BTreeMap<String, FinalDepConfigOption> = BTreeMap::new();

      for (given_name, given_string_value) in user_given_map {
        match reference_map.get(&given_name) {
          Some(hidden_config) => {
            final_map.insert(given_name, FinalDepConfigOption {
              cache_description: hidden_config.cache_description.clone(),
              cmake_var: hidden_config.cmake_var.clone(),
              value: given_string_value
            });
          },
          None => {
            let valid_option_list: String = reference_map.keys()
              .map(|key| &key[..])
              .collect::<Vec<&str>>()
              .join(", ");

            return Err(format!(
              "User given config option '{}' isn't a valid option for the predefined dependency.\n\tValid options are [{}]",
              given_name.red(),
              valid_option_list.green()
            ));
          }
        }
      }

      return Ok(final_map);
    }
  }
}

#[derive(Clone)]
pub struct FinalDepFeature {
  // An unconstrained `default: true` is stored as a constraint matching all
  // systems.
  pub enabled_by_default_when: Option<SystemSpecifierWrapper>,
  // Unconstrained edges store a constraint matching all systems.
  pub enables: BTreeMap<String, SystemSpecifierWrapper>,
  // See the comment on the raw data struct for more info on what this means.
  pub list_value: String
}

pub fn resolve_final_dep_features(
  dep_name: &str,
  maybe_features_map: Option<&HashMap<String, RawDepFeatureConfig>>,
  maybe_feature_mode: Option<&RawDepFeatureMode>
) -> Result<BTreeMap<String, FinalDepFeature>, String> {
  let features_map: &HashMap<String, RawDepFeatureConfig> = match maybe_features_map {
    Some(the_map) if !the_map.is_empty() => the_map,
    _ => {
      // feature_mode's only job is communicating the enabled feature set, so it's meaningless
      // (and definitely a config mistake) without any features to communicate.
      if maybe_feature_mode.is_some() {
        return Err(format!(
          "Predefined dependency '{}' specifies a feature_mode, but doesn't declare any features. Either declare features or remove the feature_mode section.",
          dep_name
        ));
      }

      return Ok(BTreeMap::new());
    }
  };

  let mut final_features: BTreeMap<String, FinalDepFeature> = BTreeMap::new();

  // Constraints written in a registry config always refer to the dependency's OWN
  // declared features, never the consuming project's.
  let declared_features: BTreeSet<String> = features_map.keys().cloned().collect();

  for (feature_name, raw_feature) in features_map {
    if !is_valid_lowercase_identifier(feature_name) {
      return Err(format!(
        "Predefined dependency '{}' declares feature '{}', which is an invalid feature name. Feature names must only contain lowercase letters, numbers, hyphens, and underscores.",
        dep_name,
        feature_name.red()
      ));
    }

    let mut enables: BTreeMap<String, SystemSpecifierWrapper> = BTreeMap::new();

    if let Some(raw_enables) = raw_feature.enables.as_ref() {
      for enabler_entry in raw_enables {
        let parsing_context = GivenConstraintSpecParseContext {
          is_before_output_name: false,
          feature_context: FeatureValidationContext::PredefinedDep {
            dep_name,
            declared_features: &declared_features
          }
        };

        let (constraint, enabled_feature_name): (SystemSpecifierWrapper, &str) = match parse_leading_constraint_spec(enabler_entry, parsing_context)? {
          Some(ParseSuccess { value, rest }) => (value, rest),
          None => (SystemSpecifierWrapper::default_include_all(), &enabler_entry[..])
        };

        deny_feature_constraint_on_enabler(
          &constraint,
          enabler_entry,
          &format!("an 'enables' entry of feature '{}' of predefined dependency '{}'", feature_name, dep_name)
        )?;

        if !features_map.contains_key(enabled_feature_name) {
          return Err(format!(
            "Feature '{}' of predefined dependency '{}' enables '{}', but '{}' doesn't declare a feature named '{}'. NOTE that a predefined dependency's features may only enable other features of the same dependency.",
            feature_name.green(),
            dep_name,
            enabled_feature_name.red(),
            dep_name,
            enabled_feature_name.red()
          ));
        }

        insert_union_merged(&mut enables, enabled_feature_name.to_string(), constraint);
      }
    }

    final_features.insert(
      feature_name.clone(),
      FinalDepFeature {
        enabled_by_default_when: resolve_feature_default(
          &raw_feature.default,
          FeatureValidationContext::PredefinedDep {
            dep_name,
            declared_features: &declared_features
          },
          &format!("the 'default' value of feature '{}' of predefined dependency '{}'", feature_name, dep_name)
        )?,
        enables,
        list_value: raw_feature.list_value.clone()
          .unwrap_or_else(|| feature_name.clone())
      }
    );
  }

  return Ok(final_features);
}

pub trait PredefinedDepFunctionality {
  fn can_cross_compile(&self) -> bool;
  fn get_target_config_map(&self) -> &FinalTargetConfigMap;
  fn target_name_set(&self) -> HashSet<String>;
  fn supports_emscripten(&self) -> bool;
  fn raw_emscripten_config(&self) -> Option<&RawEmscriptenConfig>;
  fn uses_emscripten_link_flag(&self) -> bool;
  fn is_internally_supported_by_emscripten(&self) -> bool;
  fn debian_packages_config(&self) -> &FinalDebianPackagesConfig;
  fn config_options_map(&self) -> &BTreeMap<String, FinalDepConfigOption>;
  fn dep_features_map(&self) -> &BTreeMap<String, FinalDepFeature>;
  fn dep_feature_list_var(&self) -> Option<&str>;
}