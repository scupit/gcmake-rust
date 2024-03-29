use std::{path::{Path, PathBuf}, fs::{self, File}, io::{self, Write}, collections::{HashMap, HashSet}};

fn main() {
  // Order is important. Any listed file may depend on any of the files listed above it.
  let ordered_cmake_util_names: Vec<&str> = vec![
    "FindSphinx",
    "gcmake-variables",
    // CPM.cmake is a dependency manager. I did not write this file. It comes from here:
    // https://github.com/cpm-cmake/CPM.cmake
    // CPM configuration depends on variables set in gcmake-variables.cmake.
    "CPM",
    "gcmake-cross-compilation-utils",
    "gcmake-emscripten-utils",
    "gcmake-dir-config",
    "gcmake-toggle-lib-util",
    "gcmake-pre-build-configuration-utils",
    "gcmake-resource-copy-util",
    "gcmake-general-utils",
    "gcmake-windows-rc-file-utils",
    "gcmake-pgo-utils",
    "gcmake-installation-utils",
    "gcmake-cpack-utils",
    "gcmake-config-file-writer",
    "gcmake-features-util",
    "gcmake-cppfront-utils",
    "gcmake-documentation-utils"
  ];

  match combine_cmake_util_files(ordered_cmake_util_names) {
    Ok(fn_result) => if let Err(err_msg) = fn_result {
      panic!("Error when combining CMake util files: \"{}\"", err_msg);
    },
    Err(io_error) => panic!("IO error when combining CMake util files: \"{}\"", io_error.to_string())
  }

  if let Err(io_error) = write_test_mains() {
    panic!("IO Error when writing test_mains: \"{}\"", io_error.to_string());
  }

  let doxyfile_write_result = write_default_doxyfile(
    Path::new("./src/program_actions/default_file_creator/doxygen/Doxyfile.in"),
    Path::new("./src/program_actions/default_file_creator/doxygen/mod.rs")
  );

  if let Err(io_error) = doxyfile_write_result {
    panic!("IO error when writing default Doxyfile.in to string: {}", io_error.to_string());
  }

  let sphinx_write_result = write_default_sphinx_files(
    Path::new("./src/program_actions/default_file_creator/sphinx/conf.py.in"),
    Path::new("./src/program_actions/default_file_creator/sphinx/index.rst"),
    Path::new("./src/program_actions/default_file_creator/sphinx/mod.rs")
  );

  if let Err(io_error) = sphinx_write_result {
    panic!(
      "IO error when writing default Sphinx files 'conf.py.in' and 'index.rst' to string: {}",
      io_error.to_string()
    );
  }
}

// ==================================================
// Helper functions
// ==================================================

fn write_contents_to_var_in_file(
  var_name: impl AsRef<str>,
  in_file_path: &Path,
  out_file: &mut File
) -> io::Result<()> {
  writeln!(out_file,
    "pub const {}: &'static str =\n\"{}\";\n",
    var_name.as_ref(),
    fs::read_to_string(&in_file_path)?
      .replace('"', "\\\"")
  )?;

  Ok(())
}

// ==================================================
// Write main files as rust strings 
// ==================================================

fn write_test_mains() -> io::Result<()> {
  let mains_root_dir: PathBuf = PathBuf::from("./src/project_generator/cpp_test_mains");

  let mut output_file_path: PathBuf = mains_root_dir.clone();
  output_file_path.push("test_mains.rs");

  let mut output_file: File = File::create(&output_file_path)?;

  let main_group = [
    ("auto_main.cpp", "AUTO_MAIN"),
    ("custom_main.cpp", "CUSTOM_MAIN")
  ];

  for test_file_dir_name in ["catch2", "doctest", "googletest"] {
    let mut main_file_path: PathBuf = mains_root_dir.clone();
    main_file_path.push(test_file_dir_name);

    for (main_file_name, main_type_name) in &main_group {
      main_file_path.push(main_file_name);

      write_contents_to_var_in_file(
        format!("{}_{}", test_file_dir_name.to_uppercase(), main_type_name),
        &main_file_path,
        &mut output_file
      )?;

      main_file_path.pop();
    }
  }

  Ok(())
}


// ==================================================
// Write default files (for generation) into strings
// ==================================================

fn write_default_doxyfile(
  in_path: &Path,
  out_path: &Path
) -> io::Result<()> {
  let mut out_file: File = File::create(out_path)?;
  write_contents_to_var_in_file(
    "DEFAULT_DOXYFILE_IN_CONTENTS",
    in_path,
    &mut out_file
  )?;

  Ok(())
}

fn write_default_sphinx_files(
  conf_py_in_path: &Path,
  index_rst_path: &Path,
  out_path: &Path
) -> io::Result<()> {
  let mut out_file: File = File::create(out_path)?;

  write_contents_to_var_in_file(
    "DEFAULT_SPHINX_CONF_PY_IN_CONTENTS",
    conf_py_in_path,
    &mut out_file
  )?;

  write_contents_to_var_in_file(
    "DEFAULT_SPHINX_INDEX_RST_CONTENTS",
    index_rst_path,
    &mut out_file
  )?;

  Ok(())
}

// ==================================================
// Write CMake util files into strings
// ==================================================

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
    "const {}: &'static str = r##\"{}\"##;\n",
    util_var_name(name),
    contents
  )?;

  Ok(())
}