use std::{collections::HashMap};

use crate::project_info::{raw_data_in::dependencies::{internal_dep_config::{RawSubdirectoryDependency}, user_given_dep_config::{UserGivenPredefinedDependencyConfig}}};

pub enum GitRevisionSpecifier {
  Tag(String),
  CommitHash(String)
}

// Unused for now, but will be required when using config-mode find_package for
// CMake dependencies already installed on the system.
pub enum DependencyVersion {
  MinRequired(String),
  Exact(String)
}

pub struct FinalGitRepoDescriptor {
  pub repo_url: String,
  pub revision_specifier: GitRevisionSpecifier
}

pub struct PredefinedSubdirDep {
  git_repo: FinalGitRepoDescriptor,
  include_dir_name: Option<String>,
  // Map of target base name to the namespaced target name used for linking.
  namespaced_target_map: HashMap<String, String>,
  requires_custom_populate: bool
}

impl PredefinedSubdirDep {
  pub fn custom_relative_include_dir_name(&self) -> &Option<String> {
    &self.include_dir_name
  }

  pub fn requires_custom_fetchcontent_populate(&self) -> bool {
    self.requires_custom_populate
  }

  pub fn get_linkable_target_name(&self, target_name: &str) -> Option<&str> {
    self.namespaced_target_map.get(target_name)
      .map(|str_ref| &str_ref[..])
  }

  pub fn has_target_named(&self, target_name: &str) -> bool {
    self.namespaced_target_map.get(target_name).is_some()
  }

  pub fn repo_url(&self) -> &str {
    &self.git_repo.repo_url
  }

  pub fn revision(&self) -> &GitRevisionSpecifier {
    &self.git_repo.revision_specifier
  }

  pub fn from_subdir_dep(
    subdir_dep: &RawSubdirectoryDependency,
    user_given_config: &UserGivenPredefinedDependencyConfig,
    dep_name: &str
  ) -> Result<Self, String> {
    let revision_specifier: GitRevisionSpecifier = if let Some(tag_string) = &user_given_config.git_tag {
      GitRevisionSpecifier::Tag(tag_string.clone())
    }
    else if let Some(hash_string) = &user_given_config.commit_hash {
      GitRevisionSpecifier::CommitHash(hash_string.clone())
    }
    else {
      return Err(format!("Must specify either a commit_hash or git_tag for dependency '{}'", dep_name));
    };

    let mut target_map: HashMap<String, String> = HashMap::new();

    for target_name in &subdir_dep.target_names {
      target_map.insert(
        target_name.into(),
        subdir_dep.namespaced_target(target_name).unwrap()
      );
    }

    return Ok(
      Self {
        git_repo: FinalGitRepoDescriptor {
          repo_url: subdir_dep.git_repo.repo_url.clone(),
          revision_specifier
        },
        include_dir_name: subdir_dep.include_dir_name.clone(),
        namespaced_target_map: target_map ,
        requires_custom_populate: subdir_dep.requires_custom_fetchcontent_populate
      }
    )
  }
}
