use std::rc::Rc;

use crate::project_info::{final_project_data::FinalProjectData, raw_data_in::dependencies::{user_given_dep_config::UserGivenGCMakeProjectDependency, internal_dep_config::raw_dep_common::RawEmscriptenConfig}};

use super::{FinalGitRepoDescriptor, GitRevisionSpecifier};

pub enum GCMakeDependencyStatus {
  // String is the placeholder project name. Used for namespacing targets until the dependency project exists
  // (is cloned) in dep/, when the real project name can be known. 
  NotDownloaded(String),
  Available(Rc<FinalProjectData>)
}

pub struct FinalGCMakeDependency {
  name: String,
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
      name: dep_name.to_string(),
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

  pub fn get_name(&self) -> &str {
    &self.name
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
