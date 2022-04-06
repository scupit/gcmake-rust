use std::{collections::HashMap, rc::Rc};

use super::{raw_data_in::dependencies::{internal_dep_config::{AllPredefinedDependencies, RawDep, RawSubdirectoryDependency}, user_given_dep_config::{UserGivenPredefinedDependencyConfig, self, UserGivenGCMakeProjectDependency}}, final_project_data::FinalProjectData};

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
  repo_url: String,
  revision_specifier: GitRevisionSpecifier
}


// TODO: Construct these in FinalProjectData by merging the
// definition given by the user with the predefined definition by this library.
// While doing that, make sure the user spelled the project and library
// component names right.
pub struct FinalPredefinedDependency {
  git_repo: FinalGitRepoDescriptor,
  // Map of target base name to the namespaced target name used for linking.
  namespaced_target_map: HashMap<String, String>
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

  fn from_subdir_dep(
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
        namespaced_target_map: target_map 
      }
    )
  }
}

pub enum GCMakeDependencyStatus {
  // String is the placeholder project name. Used for namespacing targets until the dependency project exists
  // (is cloned) in dep/, when the real project name can be known. 
  NotDownloaded(String),
  Available(Rc<FinalProjectData>)
}

pub struct FinalGCMakeDependency {
  git_repo: FinalGitRepoDescriptor,
  dep_project_status: GCMakeDependencyStatus,
}

impl FinalGCMakeDependency {
  pub fn new(
    dep_name: &str,
    given_config: &UserGivenGCMakeProjectDependency,
    maybe_associated_project: Option<Rc<FinalProjectData>>
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

    return Ok(Self {
      git_repo: FinalGitRepoDescriptor {
        repo_url: given_config.repo_url.clone(),
        revision_specifier: download_specifier
      },
      dep_project_status: match maybe_associated_project {
        Some(project_info) => GCMakeDependencyStatus::Available(project_info),
        None => GCMakeDependencyStatus::NotDownloaded(dep_name.to_string())
      }
    })
  }

  pub fn repo_url(&self) -> &str {
    &self.git_repo.repo_url
  }

  pub fn revision(&self) -> &GitRevisionSpecifier {
    &self.git_repo.revision_specifier
  }

  pub fn project_status(&self) -> &GCMakeDependencyStatus {
    &self.dep_project_status
  }

  pub fn get_linkable_target_name(&self, base_name: &str) -> Result<Option<String>, String> {
    match self.project_status() {
      GCMakeDependencyStatus::NotDownloaded(placeholder_prefix) => {
        Ok(Some(format!(
          "{}::{}",
          placeholder_prefix,
          base_name
        )))
      },
      GCMakeDependencyStatus::Available(project_info) => {
        project_info.get_namespaced_public_linkable_target_name(base_name)
      }
    }
  }
}
