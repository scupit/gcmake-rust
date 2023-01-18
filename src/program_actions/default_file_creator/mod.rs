use std::{io::{self, Write}, path::PathBuf, fs::File,};

use colored::Colorize;

use crate::{project_info::{final_project_data::UseableFinalProjectDataGroup, path_manipulation::cleaned_pathbuf}, common::prompt::prompt_until_boolean};

use self::doxygen::DEFAULT_DOXYFILE;

mod doxygen;

pub fn write_default_doxyfile(
  file_name: &str,
  project_group: &UseableFinalProjectDataGroup
) -> io::Result<()> {
  let mut doxyfile_in_path: PathBuf = PathBuf::from(project_group.root_project.get_docs_dir_relative_to_cwd());
  doxyfile_in_path.push(file_name);
  doxyfile_in_path = cleaned_pathbuf(doxyfile_in_path);

  let should_write_doxyfile: bool;

  if doxyfile_in_path.exists() {
    should_write_doxyfile = prompt_until_boolean(&format!(
      "{} already exists. Do you want to overwrite it?",
      doxyfile_in_path.to_str().unwrap().yellow()
    ))?;
  }
  else {
    should_write_doxyfile = true;
  }

  if should_write_doxyfile {
    let doxyfile_in: File = File::create(&doxyfile_in_path)?;
    writeln!(&doxyfile_in, "{}", DEFAULT_DOXYFILE)?;
    println!("{} generated successfully!", doxyfile_in_path.to_str().unwrap().bright_green());
  }
  else {
    println!("Skipping docs/Doxyfile.in creation.")
  }
  
  Ok(())
}