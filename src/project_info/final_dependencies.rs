use std::collections::HashMap;

use super::raw_data_in::dependencies::{internal_dep_config::{AllPredefinedDependencies, RawDep, RawSubdirectoryDependency}, user_given_dep_config::{UserGivenPredefinedDependencyConfig, self}};

pub enum GitRevisionSpecifier {
  Tag(String),
  CommitHash(String)
}

pub enum DependencyVersion {
  MinRequired(String),
  Exact(String)
}

pub struct FinalGitRepoDescriptor {
  repo_url: String,
  download_specifier: GitRevisionSpecifier
}


// TODO: Construct these in FinalProjectData by merging the
// definition given by the user with the predefined definition by this library.
// While doing that, make sure the user spelled the project and library
// component names right.
pub struct FinalPredefinedDependency {
  git_repo: FinalGitRepoDescriptor,
  // Map of target base name to the namespaced target name used for linking.
  target_map: HashMap<String, String>
}

impl FinalPredefinedDependency {
  pub fn new(
    dep_config: &AllPredefinedDependencies,
    dep_name: &str,
    user_given_config: &UserGivenPredefinedDependencyConfig
  ) -> Result<Self, String> {
    return match dep_config.find_dependency(dep_name) {
      Some(dep) => match dep {
        RawDep::AsSubdirectory(subdir_dep) =>
          Self::from_subdir_dep(subdir_dep, user_given_config, dep_name)
      }
      None => Err(format!("Unable to find predefined dependency named '{}'.", dep_name))
    }
  }

  pub fn get_linkable_target_name(&self, target_name: &str) -> Option<&str> {
    self.target_map.get(target_name)
      .map(|str_ref| &str_ref[..])
  }

  pub fn has_target_named(&self, target_name: &str) -> bool {
    self.target_map.get(target_name).is_some()
  }

  pub fn repo_url(&self) -> &str {
    &self.git_repo.repo_url
  }

  pub fn revision(&self) -> &GitRevisionSpecifier {
    &self.git_repo.download_specifier
  }

  fn from_subdir_dep(
    subdir_dep: &RawSubdirectoryDependency,
    user_given_config: &UserGivenPredefinedDependencyConfig,
    dep_name: &str
  ) -> Result<Self, String> {
    let download_specifier: GitRevisionSpecifier = if let Some(tag_string) = &user_given_config.git_tag {
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
          download_specifier
        },
        target_map 
      }
    )
  }
}
