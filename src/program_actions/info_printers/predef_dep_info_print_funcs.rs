use crate::{project_info::raw_data_in::dependencies::internal_dep_config::raw_dep_common::RawPredepCommon};

pub fn print_predef_dep_header(dep_name: &str) {
  println!("\n========== {} ==========", dep_name);
}

pub fn print_predep_targets(dep_name: &str, common_info: &dyn RawPredepCommon) {
  println!("{}::{{", dep_name);

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