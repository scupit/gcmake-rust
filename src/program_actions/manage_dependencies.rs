use std::{env, path::{PathBuf, Path}, process::{self, Output, Stdio}, io, fs};
use colored::*;

const GCMAKE_DEP_CONFIG_REPO_SSH_URL: &'static str = "git@github.com:scupit/gcmake-dependency-configs.git";

pub fn gcmake_config_root_dir() -> PathBuf {
  let user_home_var_name: &str = if cfg!(windows)
    { "USERPROFILE" }
    else { "HOME" };

  // Make sure this is keps in sync with the GCMAKE_CONFIG_DIR variable defined in
  // the gcmake-variables.cmake util file.
  let mut home_path = PathBuf::from(env::var(user_home_var_name).unwrap());
  home_path.push(".gcmake");
  return home_path;
}

pub fn gcmake_dep_cache_dir() -> PathBuf {
  let mut the_dir: PathBuf = gcmake_config_root_dir();
  // Make sure this is keps in sync with the GCMAKE_DEP_CACHE_DIR variable defined in
  // the gcmake-variables.cmake util file.
  the_dir.push("dep-cache");
  return the_dir;
}

fn get_repo_name() -> &'static str {
  let without_prefix: &str = GCMAKE_DEP_CONFIG_REPO_SSH_URL.trim_end_matches(".git");
  let first_name_index: usize = GCMAKE_DEP_CONFIG_REPO_SSH_URL.rfind('/')
    .map_or(0, |index| index + 1);

  return &without_prefix[first_name_index..];
}

pub fn gcmake_dep_config_dir() -> PathBuf {
  gcmake_config_root_dir().join(get_repo_name())
}

fn exited_successfully(output: &Output) -> bool {
  output.status.success()
}

fn command_error_string(
  command_run: impl AsRef<str>,
  output: &Output
) -> String {
  println!("stderr length: {}", output.stderr.len());
  println!("is empty?: {}", output.stderr.is_empty());
  return format!(
    "{}'{}': {}",
    "Error when running ".red(),
    command_run.as_ref(),
    String::from_utf8(output.stderr.clone()).unwrap()
  );
}

fn checkout_branch(
  local_repo_location: impl AsRef<Path>,
  branch_name: &str
) -> io::Result<Option<String>> {
  let checkout_output = process::Command::new("git")
    .current_dir(local_repo_location.as_ref())
    .args([
      "checkout",
      branch_name
    ])
    .output()?;

  if !exited_successfully(&checkout_output) {
    return Ok(Some(command_error_string(
      &format!("git checkout {}", branch_name),
      &checkout_output
    )));
  }

  Ok(None)
}

pub enum DepConfigUpdateResult {
  SubprocessError(String),
  NewlyDownloaded {
    local_repo_location: PathBuf,
    branch: String
  },
  UpdatedBranch {
    local_repo_location: PathBuf,
    branch: Option<String>
  }
}

pub fn update_dependency_config_repo(maybe_branch_name: &Option<String>) -> io::Result<DepConfigUpdateResult> {
  let local_repo_location: PathBuf = gcmake_dep_config_dir();

  if local_repo_location.is_dir() {
    if let Some(branch_name) = maybe_branch_name {
      if let Some(checkout_failure_message) = checkout_branch(&local_repo_location, branch_name)? {
        return Ok(DepConfigUpdateResult::SubprocessError(checkout_failure_message));
      }
    }

    let pull_output: Output = process::Command::new("git")
      .current_dir(&local_repo_location)
      .arg("pull")
      .stdout(Stdio::inherit())
      .output()?;

    if !exited_successfully(&pull_output) {
      return Ok(DepConfigUpdateResult::SubprocessError(command_error_string(
        "git pull",
        &pull_output
      )));
    }

    return Ok(DepConfigUpdateResult::UpdatedBranch {
      local_repo_location,
      branch: maybe_branch_name.clone()
    });
  }
  else {
    fs::create_dir_all(&local_repo_location)
      .map_err(|err| io::Error::from(err))?;

    let clone_output: Output = process::Command::new("git")
      .current_dir(&local_repo_location)
      .args([
        "clone",
        GCMAKE_DEP_CONFIG_REPO_SSH_URL,
        local_repo_location.to_str().unwrap()
      ])
      .output()?;

    if !exited_successfully(&clone_output) {
      return Ok(DepConfigUpdateResult::SubprocessError(command_error_string(
        "git clone",
        &clone_output
      )));
    }

    let default_branch = "develop";

    if let Some(checkout_error_message) = checkout_branch(&local_repo_location, default_branch)? {
      return Ok(DepConfigUpdateResult::SubprocessError(checkout_error_message));
    }

    return Ok(DepConfigUpdateResult::NewlyDownloaded {
      branch: default_branch.to_string(),
      local_repo_location
    });
  }
}
