use std::{path::Path, fs::{self, File}, io::{self, Write}, collections::{HashMap, HashSet}};

fn main() {
  let ordered_cmake_util_names: Vec<&str> = vec![
    "gcmake-variables",
    "gcmake-dir-config",
    "gcmake-toggle-lib-util",
    "gcmake-pre-build-configuration-utils",
    "gcmake-resource-copy-util",
    "gcmake-general-utils",
    "gcmake-installation-utils",
    "gcmake-cpack-utils"
  ];

  match combine_cmake_util_files(ordered_cmake_util_names) {
    Ok(fn_result) => if let Err(err_msg) = fn_result {
      panic!("Error when combining CMake util files: \"{}\"", err_msg);
    },
    Err(io_error) => panic!("IO error when combining CMake util files: \"{}\"", io_error.to_string())
  }
}

struct UtilInfo {
  name: String,
  contents: String
}

fn util_var_name(util_name: &str) -> String {
  format!("{}_contents", util_name)
    .replace('-', "_")
    .to_uppercase()
}

// Does rust have a Haskell-like traverse function?
fn condense_errs<T>(it: impl Iterator<Item=io::Result<T>>) -> io::Result<Vec<T>> {
  let mut item_vec: Vec<T> = Vec::new();

  for maybe_item in it {
    item_vec.push(maybe_item?);
  }
  return Ok(item_vec);
}

fn combine_cmake_util_files(ordered_utils: Vec<&str>) -> io::Result<Result<(), String>> {
  let combined_utils_path = Path::new("./src/file_writers/cmake_writer/ordered_utils.rs");
  let mut combined_utils_file = File::create(&combined_utils_path)?;
  let cmake_utils_dir = Path::new("./src/file_writers/cmake_writer/util_files");

  let iter_maybe_utils = cmake_utils_dir.read_dir()?
    .filter(|maybe_entry| {
      match maybe_entry {
        Ok(entry) => {
          let entry_path = entry.path();
          let has_cmake_extension: bool = entry_path.extension()
            .map(|extension| extension.to_str().unwrap() == "cmake")
            .unwrap_or(false);

          return entry_path.is_file() && has_cmake_extension;
        },
        Err(_) => false
      }
    })
    .map(|maybe_entry| {
      let entry = maybe_entry?;
      let contents: String = fs::read_to_string(entry.path())?; 

      return Ok(UtilInfo {
        name: entry.path().file_stem().unwrap().to_str().unwrap().to_string(),
        contents
      })
    });

  let util_map: HashMap<String, UtilInfo> = condense_errs(iter_maybe_utils)?
    .into_iter()
    .map(|util_info| (util_info.name.clone(), util_info))
    .collect();

  let wanted_set: HashSet<&str> = ordered_utils.clone().into_iter().collect();
  let found_set: HashSet<&str> = util_map.keys().map(|k| &k[..]).collect();
  // let diff: HashSet<&str> = wanted_set.difference(&found_set).map(|&k| k).collect();
  let diff: HashSet<&str> = found_set.difference(&wanted_set).map(|&k| k).collect();

  if !diff.is_empty() {
    return Ok(Err(format!(
      "These utils were found, but were not matched: [ {} ]",
      diff.into_iter().collect::<Vec<&str>>().join(", ")
    )));
  }
  else if util_map.len() != ordered_utils.len() {
    return Ok(Err(format!(
      "Found {} util files, but {} were expected. Make sure all file names match correctly",
      util_map.len(),
      ordered_utils.len()
    )));
  }

  writeln!(&mut combined_utils_file,
    "use super::cmake_utils_writer::CMakeUtilFile;\n"
  )?;

  for util_name in &ordered_utils {
    match util_map.get(*util_name) {
      None => {
        return Ok(Err(format!(
          "Unable to find {}.cmake in {}",
          util_name,
          combined_utils_path.to_str().unwrap()
        )))
      },
      Some(util_info) => {
        write_util(&mut combined_utils_file, util_info)?;
      }
    }
  }

  writeln!(&mut combined_utils_file,
    "pub fn ordered_utils_vec() -> Vec<CMakeUtilFile> {{\n\treturn vec!["
  )?;

  for util_name in ordered_utils {
    writeln!(&mut combined_utils_file,
      "\t\tCMakeUtilFile {{\n\t\t\tutil_name: \"{}\",\n\t\t\tutil_contents: {}\n\t\t}},",
      util_name,
      util_var_name(util_name)
    )?;
  };
  
  writeln!(&mut combined_utils_file,
    "\t]\n}}"
  )?;

  Ok(Ok(()))
}

fn write_util(
  combined_util_file: &mut File,
  UtilInfo { name, contents }: &UtilInfo
) -> io::Result<()> {
  writeln!(combined_util_file,
    "const {}: &'static str = r#\"{}\"#;\n",
    util_var_name(name),
    contents
  )?;

  Ok(())
}