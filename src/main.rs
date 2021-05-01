mod data_types;

use std::fs;
use data_types::*;


fn main() {
    
  let project = fs::read_to_string("./gcmake-rust-test-project/cmake_data.yaml").unwrap();
  let serialized_project: RawProject = serde_yaml::from_str(&project).unwrap();
  println!("Project in YAML: {:?}", serialized_project);
}
