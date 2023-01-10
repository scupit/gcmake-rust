use crate::project_info::{dependency_graph_mod::dependency_graph::{TargetNode, ContainedItem}, CompiledOutputItem, PreBuildScriptType};
use colored::*;

pub fn print_target_header(target: &TargetNode) {
  println!("\n========== {} ==========", target.get_yaml_namespaced_target_name().green());
}

pub fn print_export_header_include_path(target: &TargetNode) {
  match target.get_contained_item() {
    ContainedItem::CompiledOutput(output) if output.is_compiled_library_type() => {
      let full_include_prefix: String = target
        .container_project().as_ref().borrow()
        .project_wrapper().clone()
        .unwrap_normal_project()
        .get_full_include_prefix().to_string();

      println!(
        "{}",
        CompiledOutputItem::export_macro_header_include_path(&full_include_prefix, target.get_name())
      );
    },
    _ => println!("No export header")
  }
}

pub fn print_target_type(target: &TargetNode) {
  let target_type: &str = match target.get_contained_item() {
    ContainedItem::CompiledOutput(output_item) => output_item.get_output_type().name_string(),
    ContainedItem::PreBuild(pre_build_script) => match pre_build_script.get_type() {
      PreBuildScriptType::Exe(_) => "Executable pre-build script",
      PreBuildScriptType::Python(_) => "Python pre-build script"
    },
    ContainedItem::PredefinedLibrary { .. } => "Predefined dependency library target"
  };

  println!("Type: {}", target_type);
}
