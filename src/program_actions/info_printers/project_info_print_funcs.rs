use std::borrow::Borrow;

use crate::project_info::{dependency_graph_mod::dependency_graph::{DependencyGraph, ProjectWrapper}, final_dependencies::FinalPredepInfo};
use colored::*;

pub fn print_project_header(project: &DependencyGraph) {
  println!("\n========== {} ==========", project.project_debug_name().green());
}

pub fn print_project_include_prefix(project_graph: &DependencyGraph) {
  match project_graph.project_wrapper().maybe_normal_project() {
    Some(normal_project) => {
      println!("Include prefix: {}", normal_project.get_full_include_prefix())
    },
    None => println!("Cannot determine include prefix.")
  }
}

pub fn print_immediate_subprojects(project_graph: &DependencyGraph) {
  match project_graph.project_wrapper().maybe_normal_project() {
    Some(_) => {
      print!("Subprojects: ");
      
      if project_graph.get_subprojects().is_empty() {
        println!("None");
      }
      else {
        println!();
        for (subproject_name, _) in project_graph.get_subprojects() {
          println!("\t- {}", subproject_name);
        }
      }
    },
    None => println!("Cannot determine subprojects")
  }
}

fn extract_repo_url(project_graph: &DependencyGraph) -> Result<String, String> {
  return match project_graph.project_wrapper() {
    ProjectWrapper::PredefinedDependency(predef_def) => match predef_def.predefined_dep_info() {
      FinalPredepInfo::Subdirectory(subdir_dep) if subdir_dep.download_method().is_git() => {
        Ok(subdir_dep.download_method().git_details().unwrap().repo_url.clone())
      },
      _ => {
        Err(format!(
          "Repository information is not available for \"{}\" because it is retrieved from the system, not cloned as part of the build.",
          project_graph.project_debug_name()
        ))
      }
    },
    ProjectWrapper::GCMakeDependencyRoot(gcmake_dep) => Ok(gcmake_dep.repo_url().to_string()),
    // Normal projects don't have a repository URL because they are part of the repository itself. However,
    // the root of a normal project may be a GCMake dependency.
    ProjectWrapper::NormalProject(_) => {
      if let ProjectWrapper::GCMakeDependencyRoot(root) = project_graph.root_project().as_ref().borrow().project_wrapper() {
        Ok(root.repo_url().to_string())
      }
      else {
        Err(format!("No repo URL"))
      }
    }
  }
  
}

pub fn print_project_repo_url(project_graph: &DependencyGraph) {
  match extract_repo_url(project_graph) {
    Ok(url) => println!("Repo URL:\n\t{}", url),
    Err(reason_missing) => println!("{}", reason_missing)
  }
}

pub fn print_project_can_cross_compile(project_graph: &DependencyGraph) {
  let mut project_is_available: bool = true;

  // TODO: Have a second bool (or maybe a list of projects) denoting which projects
  // are not available.
  let can_cross_compile: bool = match project_graph.project_wrapper() {
    ProjectWrapper::GCMakeDependencyRoot(gcmake_dep) => {
      project_is_available = gcmake_dep.is_available();
      gcmake_dep.can_trivially_cross_compile()
    },
    ProjectWrapper::NormalProject(project_info) => project_info.can_trivially_cross_compile(),
    ProjectWrapper::PredefinedDependency(predef_dep) => predef_dep.can_trivially_cross_compile()
  };

  print!("Can trivially cross-compile: {}", can_cross_compile);

  if project_is_available {
    println!();
  }
  else {
    println!(" (Can't accurately determine, since the project hasn't been downloaded yet)");
  }
}

pub fn print_project_supports_emscripten(project_graph: &DependencyGraph) {
  let mut project_is_available: bool = true;

  // TODO: Have a second bool (or maybe a list of projects) denoting which projects
  // are not available.
  let supports_emscripten: bool = match project_graph.project_wrapper() {
    ProjectWrapper::GCMakeDependencyRoot(gcmake_dep) => {
      project_is_available = gcmake_dep.is_available();
      gcmake_dep.supports_emscripten()
    },
    ProjectWrapper::NormalProject(project_info) => project_info.supports_emscripten(),
    ProjectWrapper::PredefinedDependency(predef_dep) => predef_dep.supports_emscripten()
  };

  print!("Supports Emscripten: {}", supports_emscripten);

  if project_is_available {
    println!();
  }
  else {
    println!(" (Can't accurately determine, since the project hasn't been downloaded yet)");
  }
}

struct TargetResolutionOptions {
  should_recurse: bool,
  include_tests: bool,
  include_pre_build: bool
}

pub fn print_project_output_list(project_graph: &DependencyGraph) {
  print_project_output_list_helper(
    project_graph,
    &TargetResolutionOptions {
      should_recurse: true,
      include_pre_build: true,
      include_tests: true
    }
  )
}

fn print_project_output_list_helper(
  project_graph: &DependencyGraph,
  options: &TargetResolutionOptions
) {
  print_single_project_outputs(project_graph, options);

  if options.include_tests {
    for (_, test_project) in project_graph.get_test_projects() {
      print_project_output_list_helper(&test_project.as_ref().borrow(), options);
    }
  }

  if options.should_recurse {
    for (_, subproject) in project_graph.get_subprojects() {
      print_project_output_list_helper(&subproject.as_ref().borrow(), options);
    }
  }
}

fn print_single_project_outputs(
  project_graph: &DependencyGraph,
  options: &TargetResolutionOptions
) {
  println!("\n{}::{{", project_graph.project_debug_name().magenta());

  if options.include_pre_build && project_graph.get_pre_build_node().is_some() {
    println!("   {}", "pre-build".bright_cyan());
  }

  for (_, output_item) in project_graph.get_this_target_map().borrow().iter() {
    println!(
      "   {}",
      output_item.as_ref().borrow().get_name()
    );
  }

  println!("}}");
}
