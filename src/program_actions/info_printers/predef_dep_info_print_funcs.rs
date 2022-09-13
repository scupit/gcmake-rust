use crate::{project_info::raw_data_in::dependencies::internal_dep_config::raw_dep_common::RawPredepCommon};
use colored::*;

pub fn print_predef_dep_header(dep_name: &str) {
  println!("\n========== {} ==========", dep_name.green());
}

pub fn print_predep_targets(dep_name: &str, common_info: &dyn RawPredepCommon) {
  println!("{}::{{", dep_name.green());

  // Full predefined dependency usage info is only instantiated when a predefined
  // dependency is used by a project. However since basic predefined dependency information
  // is globally available in the ~/.gcmake config directory, we shouldn't have to rely
  // on a project existing to print out basic predefined dependency information.
  // TODO: Try to load a project, and use loaded predefined dependency information from it
  // if possible. Otherwise stick with the raw information.
  for (target, _) in common_info.raw_target_map_in() {
    println!("    {},", target);
  }

  println!("}}");
}

pub fn print_predep_repo_url(common_info: &dyn RawPredepCommon) {
  match common_info.repo_url() {
    Some(repo_url) => println!("{}", repo_url),
    None => println!("No repo URL")
  }
}

pub fn print_predep_github_url(common_info: &dyn RawPredepCommon) {
  match common_info.github_url() {
    Some(github_url) => println!("{}", github_url),
    None => println!("No GitHub URL")
  }
}
