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

pub fn print_predep_can_cross_compile(common_info: &dyn RawPredepCommon) {
  println!("Can trivially cross-compile: {}", common_info.can_trivially_cross_compile());
}

pub fn print_predep_supports_emscripten(common_info: &dyn RawPredepCommon) {
  println!("Supports Emscripten: {}", common_info.supports_emscripten());
}

pub fn print_predep_supported_download_methods(common_info: &dyn RawPredepCommon) {
  let mut supported_download_methods: Vec<&str> = Vec::new();

  if common_info.supports_git_download_method() {
    supported_download_methods.push("Git");
  }

  if common_info.supports_url_download_method() {
    supported_download_methods.push("Archive (from URL)");
  }

  if supported_download_methods.is_empty() {
    println!("Doesn't support any download methods.");
  }
  else {
    println!("Supported download methods:");
    for download_method in supported_download_methods {
      println!(
        "  - {}",
        download_method.cyan()
      );
    }
  }
}

pub fn print_predep_doc_link(common_info: &dyn RawPredepCommon) {
  match common_info.gcmake_readme_url() {
    None => println!("Doesn't have a README"),
    Some(gcmake_readme_url) => println!("{}", gcmake_readme_url)
  }
}