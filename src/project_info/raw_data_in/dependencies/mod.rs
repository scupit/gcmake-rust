pub mod internal_dep_config;
pub mod user_given_dep_config;

use self::internal_dep_config::AllPredefinedDependencies;

pub fn supported_dependency_configs() -> Result<AllPredefinedDependencies, String> {
  /*
    Whole bunch of TODOS related to the new dependency configuration system.
    ================================================================================

    configuration repository should be located in ~/.gcmake/gcmake-dependency-configs
    WHERE ~ is HOME env var on Unix and USERPROFILE on Windows.

    TODOS:
      1. Get all configurations from the external dependency repository if the repository is present.
      2. Otherwise, prompt the user asking to "download the dependency configuration repository to
          ~/.gcmake-/gcmake-dependency-configs" using the same steps as TODO #3.
      3. Add an 'update-dep-configs [-b <branch>]' command to clone the repo if it doesn't exist, checkout
          the given branch, and pull the latest changes on that branch.
      4. Generate a single file yaml index of all dependency configurations post-pull success.
          I'm not sure if this is needed or not, so this is more of a quality-of-life convenience step.
  */

  return Err(String::from("Haven't implemented getting dependency configurations using the new configs repository system yet."));

  // return match serde_yaml::from_str::<AllPredefinedDependencies>(DEPENDENCIES_YAML_STRING) {
  //   Ok(data) => Ok(data),
  //   Err(serde_error) => Err(serde_error.to_string())
  // }
}