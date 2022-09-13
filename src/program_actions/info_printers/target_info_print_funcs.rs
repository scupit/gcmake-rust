use crate::project_info::dependency_graph_mod::dependency_graph::{TargetNode, ContainedItem};
use colored::*;


pub fn print_target_header(target: &TargetNode) {
  println!("\n========== {} ==========", target.get_yaml_namespaced_target_name().green());
}

pub fn print_export_header_include_path(target: &TargetNode) {
  match target.get_contained_item() {
    ContainedItem::CompiledOutput(output) if output.is_compiled_library_type() => {
      println!(
        "\"{}/{}_export.h\"",
        target.container_project().as_ref().borrow().project_wrapper().clone().unwrap_normal_project().get_full_include_prefix(),
        target.get_name()
      )
    },
    _ => println!("No export header")
  }
}
