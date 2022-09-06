use crate::project_info::dependency_graph_mod::dependency_graph::{DependencyGraph};

pub fn print_project_header(project: &DependencyGraph) {
  println!("\n========== {} ==========", project.project_debug_name());
}

pub fn print_project_include_prefix(project_graph: &DependencyGraph) {
  match project_graph.project_wrapper().maybe_normal_project() {
    Some(normal_project) => {
      println!("Include prefix:\n\t{}", normal_project.get_full_include_prefix())
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
