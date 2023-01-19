use std::{io::{self, Write}, path::{PathBuf, Path}, fs::{File, self},};

use colored::Colorize;

use crate::{project_info::{final_project_data::UseableFinalProjectDataGroup, path_manipulation::cleaned_pathbuf}, common::prompt::prompt_until_boolean};

use self::{doxygen::DEFAULT_DOXYFILE_IN_CONTENTS, sphinx::{DEFAULT_SPHINX_INDEX_RST_CONTENTS, DEFAULT_SPHINX_CONF_PY_IN_CONTENTS}};

mod doxygen;
mod sphinx;

struct FileGroup<'a> {
  file_name: &'a str,
  default_contents: &'static str
}

fn write_default_docs_config_files(
  project_group: &UseableFinalProjectDataGroup,
  file_groups: Vec<FileGroup>
) -> io::Result<()> {
  let docs_path: &Path = Path::new(project_group.root_project.get_docs_dir_relative_to_cwd());

  for FileGroup { file_name, default_contents } in file_groups {
    let full_file_path: PathBuf = cleaned_pathbuf(docs_path.join(file_name));
    let should_write_file: bool;

    if full_file_path.exists() {
      should_write_file = prompt_until_boolean(&format!(
        "{} already exists. Do you want to overwrite it?",
        full_file_path.to_str().unwrap().yellow()
      ))?;
    }
    else {
      should_write_file = true;
    }

    if should_write_file {
      // docs_path is the directory which contains the file. If that changes, we'll have to use
      // the docs_path.parent() directory.
      if !docs_path.exists() {
        fs::create_dir_all(docs_path)?;
      }

      let new_file: File = File::create(&full_file_path)?;
      writeln!(&new_file, "{}", default_contents)?;
      println!("{} generated successfully!", full_file_path.to_str().unwrap().cyan());
    }
    else {
      println!("Skipping docs/Doxyfile.in creation.")
    }
  }

  Ok(())
}

pub fn write_default_doxyfile(
  file_name: &str,
  project_group: &UseableFinalProjectDataGroup
) -> io::Result<()> {
  return write_default_docs_config_files(
    project_group,
    vec![FileGroup {
      file_name,
      default_contents: DEFAULT_DOXYFILE_IN_CONTENTS
    }]
  );
}

pub fn write_default_sphinx_files(
  index_rst_name: &str,
  conf_py_in_name: &str,
  project_group: &UseableFinalProjectDataGroup
) -> io::Result<()> {
  return write_default_docs_config_files(
    project_group,
    vec![
      FileGroup {
        file_name: index_rst_name,
        default_contents: DEFAULT_SPHINX_INDEX_RST_CONTENTS
      },
      FileGroup {
        file_name: conf_py_in_name,
        default_contents: DEFAULT_SPHINX_CONF_PY_IN_CONTENTS
      }
    ]
  );
}