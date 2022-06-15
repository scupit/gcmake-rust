use std::rc::Rc;

use crate::project_info::{final_project_data::FinalProjectData, raw_data_in::dependencies::user_given_dep_config::UserGivenGCMakeProjectDependency};

use super::{FinalGitRepoDescriptor, GitRevisionSpecifier};

pub enum GCMakeDependencyStatus {
  // String is the placeholder project name. Used for namespacing targets until the dependency project exists
  // (is cloned) in dep/, when the real project name can be known. 
  NotDownloaded(String),
  Available(Rc<FinalProjectData>)
}

pub struct FinalGCMakeDependency {
  git_repo: FinalGitRepoDescriptor,
  dep_project_status: GCMakeDependencyStatus
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
        revision_specifier: download_specifier,
        recursive_clone: true
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

  pub fn should_recursive_clone(&self) -> bool {
    self.git_repo.recursive_clone
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
