use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
  name: String,
  version: String,
  languages: Vec<String>,
  supported_compilers: Vec<String>
}

// #[derive(Serialize, Deserialize)]
// pub struct OutputData {
//   outputType: String
// }