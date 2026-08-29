use std::{rc::Rc, collections::{BTreeMap, BTreeSet}};

use crate::project_info::{final_project_data::FinalProjectData, raw_data_in::dependencies::{user_given_dep_config::UserGivenGCMakeProjectDependency}, parsers::general_parser::ParseSuccess, platform_spec_parser::{parse_leading_constraint_spec, SystemSpecifierWrapper}, FeatureValidationContext, FinalFeatureConfig, GivenConstraintSpecParseContext, deny_feature_constraint_on_enabler, insert_union_merged};

use super::{FinalGitRepoDescriptor, GitRevisionSpecifier};
use colored::Colorize;

const GCMAKE_DEP_HASH_FILE_NAME: &'static str = "unique_hash.txt";

pub fn relative_hash_file_path() -> String {
  return format!(".gcmake/{}", GCMAKE_DEP_HASH_FILE_NAME);
}

pub enum GCMakeDependencyStatus {
  // String is the placeholder project name. Used for namespacing targets until the dependency project exists
  // (is cloned) in dep/, when the real project name can be known. 
  NotDownloaded(String),
  Available(Rc<FinalProjectData>)
}

pub struct GCMakeDepIDHash {
  pub hash_string: String,
  pub relative_hash_file: String
}

pub struct FinalGCMakeDependency {
  name: String,
  git_repo: FinalGitRepoDescriptor,
  dep_project_status: GCMakeDependencyStatus,
  use_default_features: bool,
  // Unconstrained entries store a constraint matching all systems.
  features: BTreeMap<String, SystemSpecifierWrapper>,
  hash_info: GCMakeDepIDHash
}

fn resolve_specified_features(
  dep_name: &str,
  feature_entries: Option<&BTreeSet<String>>,
  maybe_valid_feature_list: Option<&Vec<&str>>,
  maybe_declared_dep_features: Option<&BTreeMap<String, FinalFeatureConfig>>
) -> Result<BTreeMap<String, SystemSpecifierWrapper>, String> {
  let mut features: BTreeMap<String, SystemSpecifierWrapper> = BTreeMap::new();

  if let Some(feature_entries) = feature_entries {
    if !feature_entries.is_empty()
      && maybe_declared_dep_features.map_or(false, |declared_features| declared_features.is_empty())
    {
      return Err(format!(
        "The configuration for GCMake dependency '{}' enables features, but '{}' doesn't declare any features.",
        dep_name.yellow(),
        dep_name
      ));
    }

    for feature_entry in feature_entries {
      let parsing_context = GivenConstraintSpecParseContext {
        is_before_output_name: false,
        feature_context: FeatureValidationContext::ConsumingProject { valid_feature_list: maybe_valid_feature_list }
      };

      let (constraint, feature_name): (SystemSpecifierWrapper, &str) = match parse_leading_constraint_spec(feature_entry, parsing_context)? {
        Some(ParseSuccess { value, rest }) => (value, rest),
        None => (SystemSpecifierWrapper::default_include_all(), &feature_entry[..])
      };

      deny_feature_constraint_on_enabler(
        &constraint,
        feature_entry,
        &format!("a 'features' entry for gcmake dependency '{}'", dep_name)
      )?;

      if let Some(declared_dep_features) = maybe_declared_dep_features {
        if !declared_dep_features.contains_key(feature_name) {
          let valid_feature_list_str: String = declared_dep_features.keys()
            .map(|key| &key[..])
            .collect::<Vec<&str>>()
            .join(", ");

          return Err(format!(
            "The configuration for GCMake dependency '{}' enables the feature '{}', but '{}' doesn't declare a feature with that name.\n\tValid features are [{}]",
            dep_name.yellow(),
            feature_name.red(),
            dep_name,
            valid_feature_list_str.green()
          ));
        }
      }

      insert_union_merged(&mut features, feature_name.to_string(), constraint);
    }
  }

  return Ok(features);
}

impl FinalGCMakeDependency {
  pub fn new(
    dep_name: &str,
    given_config: &UserGivenGCMakeProjectDependency,
    unique_hash: String,
    maybe_associated_project: Option<Rc<FinalProjectData>>,
    maybe_valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Self, String> {
    let download_specifier: GitRevisionSpecifier = if let Some(tag_string) = &given_config.git_tag {
      GitRevisionSpecifier::Tag(tag_string.clone())
    }
    else if let Some(hash_string) = &given_config.commit_hash {
      GitRevisionSpecifier::CommitHash(hash_string.clone())
    }
    else {
      return Err(format!("Must specify either a commit_hash or git_tag for dependency '{}'", dep_name));
    };

    // Stops misspelled dependency features from being silently ignored once the
    // dependency's cmake_data.yaml is available. A newly added dependency cannot be
    // checked until CMake clones it, so its feature names are checked on the required
    // follow-up GCMake run instead.
    let features = resolve_specified_features(
      dep_name,
      given_config.features.as_ref(),
      maybe_valid_feature_list,
      maybe_associated_project.as_ref().map(|project| project.get_features())
    )?;

    return Ok(Self {
      name: dep_name.to_string(),
      git_repo: FinalGitRepoDescriptor {
        repo_url: given_config.repo_url.clone(),
        revision_specifier: download_specifier
      },
      dep_project_status: match maybe_associated_project {
        Some(project_info) => GCMakeDependencyStatus::Available(project_info),
        None => GCMakeDependencyStatus::NotDownloaded(dep_name.to_string())
      },
      use_default_features: given_config.use_default_features.unwrap_or(true),
      hash_info: GCMakeDepIDHash {
        hash_string: unique_hash,
        relative_hash_file: relative_hash_file_path()
      },
      features
    })
  }

  pub fn given_dependency_name(&self) -> &str {
    &self.name
  }

  pub fn get_hash_info(&self) -> &GCMakeDepIDHash {
    &self.hash_info
  }

  pub fn project_base_name(&self) -> &str {
    match self.project_status() {
      GCMakeDependencyStatus::NotDownloaded(_) => self.given_dependency_name(),
      GCMakeDependencyStatus::Available(project) => project.get_project_base_name()
    }
  }

  pub fn repo_url(&self) -> &str {
    &self.git_repo.repo_url
  }

  pub fn revision(&self) -> &GitRevisionSpecifier {
    &self.git_repo.revision_specifier
  }

  pub fn is_using_default_features(&self) -> bool {
    self.use_default_features
  }

  pub fn specified_features(&self) -> &BTreeMap<String, SystemSpecifierWrapper> {
    &self.features
  }

  pub fn project_status(&self) -> &GCMakeDependencyStatus {
    &self.dep_project_status
  }

  pub fn is_available(&self) -> bool {
    return match self.project_status() {
      GCMakeDependencyStatus::Available(_) => true,
      _ => false
    }
  }

  pub fn can_trivially_cross_compile(&self) -> bool {
    return match self.project_status() {
      // Use the least permissive mode until the actual state is known. This is kind of a hard
      // edge, and would be fixed if GCMake had some sort of package registry.
      GCMakeDependencyStatus::NotDownloaded(_) => false,
      GCMakeDependencyStatus::Available(available_gcmake_dep) => available_gcmake_dep.can_trivially_cross_compile()
    }
  }

  pub fn supports_emscripten(&self) -> bool {
    return match self.project_status() {
      // GCMake will fail with an error if Emscripten is listed in a project's supported compilers but the
      // project itself doesn't support Emscripten. Since the actual Emscripten support status is unknown
      // for a not-yet-downloaded dependency, return true so that the error is not thrown incorrectly.
      GCMakeDependencyStatus::NotDownloaded(_) => true,
      GCMakeDependencyStatus::Available(available_gcmake_dep) => available_gcmake_dep.supports_emscripten()
    }
  }

  pub fn get_linkable_target_name(&self, base_name: &str) -> String {
    match self.project_status() {
      GCMakeDependencyStatus::NotDownloaded(placeholder_prefix) => {
        format!(
          "{}::{}",
          placeholder_prefix,
          base_name
        )
      },
      GCMakeDependencyStatus::Available(project_info) => {
        project_info.prefix_with_project_namespace(base_name)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn declared_features(feature_names: &[&str]) -> BTreeMap<String, FinalFeatureConfig> {
    return feature_names.iter()
      .map(|feature_name| (
        feature_name.to_string(),
        FinalFeatureConfig {
          enabled_by_default_when: None,
          enables: BTreeMap::new()
        }
      ))
      .collect();
  }

  fn requested_features(feature_names: &[&str]) -> BTreeSet<String> {
    return feature_names.iter()
      .map(|feature_name| feature_name.to_string())
      .collect();
  }

  #[test]
  fn accepts_declared_gcmake_dependency_feature() {
    let requested = requested_features(&["first-one"]);
    let declared = declared_features(&["first-one"]);

    let resolved = resolve_specified_features(
      "features-lib",
      Some(&requested),
      None,
      Some(&declared)
    ).expect("A declared GCMake dependency feature should be accepted.");

    assert!(resolved.contains_key("first-one"));
  }

  #[test]
  fn rejects_unknown_gcmake_dependency_feature() {
    let requested = requested_features(&["typo"]);
    let declared = declared_features(&["first-one"]);

    let error = match resolve_specified_features(
      "features-lib",
      Some(&requested),
      None,
      Some(&declared)
    ) {
      Ok(_) => panic!("An unknown GCMake dependency feature should be rejected."),
      Err(error) => error
    };

    assert!(error.contains("features-lib"));
    assert!(error.contains("typo"));
    assert!(error.contains("first-one"));
  }

  #[test]
  fn rejects_features_for_featureless_gcmake_dependency() {
    let requested = requested_features(&["first-one"]);
    let declared = declared_features(&[]);

    let error = match resolve_specified_features(
      "features-lib",
      Some(&requested),
      None,
      Some(&declared)
    ) {
      Ok(_) => panic!("A featureless GCMake dependency should reject requested features."),
      Err(error) => error
    };

    assert!(error.contains("features-lib"));
    assert!(error.contains("doesn't declare any features"));
  }

  #[test]
  fn defers_feature_validation_for_unavailable_gcmake_dependency() {
    let requested = requested_features(&["typo"]);

    let resolved = resolve_specified_features(
      "features-lib",
      Some(&requested),
      None,
      None
    ).expect("Feature validation should wait until the GCMake dependency is available.");

    assert!(resolved.contains_key("typo"));
  }
}
