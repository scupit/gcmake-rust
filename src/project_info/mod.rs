mod helpers;
pub mod path_manipulation;
pub mod final_project_data;
pub mod final_dependencies;
pub mod raw_data_in;
pub mod final_project_configurables;
pub mod dependency_graph_mod;
mod link_spec_parser;

pub use final_project_configurables::*;
pub use helpers::ProjectOutputType;
pub use link_spec_parser::LinkSpecifier;

pub fn base_include_prefix_for_test(include_prefix: &str) -> String {
  return format!("TEST/{}", include_prefix);
}