use std::{fmt::format, fs::File, io::{self, Write}, path::PathBuf};

use crate::{data_types::raw_types::CompiledItemType, item_resolver::FinalProjectData};

pub struct CMakeListsWriter {
  project_data: FinalProjectData,
  cmakelists_file: File
}

impl CMakeListsWriter {
  pub fn new(project_data: FinalProjectData) -> io::Result<Self> {
    let file_name: String = format!("{}/CMakeLists.txt", project_data.get_project_root());
    let cmakelists_file: File = File::create(file_name)?;

    Ok(CMakeListsWriter {
      project_data,
      cmakelists_file
    })
  }

  pub fn write_cmakelists(&self) -> io::Result<()> {
    self.write_version_header()?;
    self.write_project_header()?;
    self.write_outputs()?;
    Ok(())
  }

  fn write_version_header(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "cmake_minimum_required( VERSION 3.12 )")
  }

  fn write_project_header(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "project( {} )",
      self.project_data.get_project_name()
    )?;

    Ok(())
  }

  fn set_basic_var(&self, var_name: &str, var_value: &str) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "set( {} {} )", var_name, var_value)?;
    Ok(())
  }

  fn set_file_collection(
    &self,
    var_name: &str,
    file_location_root: &str,
    cmake_location_prefix: &str,
    file_path_collection: &Vec<PathBuf>
  ) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "set( {}", var_name)?;
    for file_path in file_path_collection {
      let fixed_file_path = file_path
        .to_string_lossy()
        .replace(file_location_root, "");

      writeln!(&self.cmakelists_file, "\t${{{}}}{}", &cmake_location_prefix, fixed_file_path)?;
    }
    writeln!(&self.cmakelists_file, ")")?;

    Ok(())
  }

  fn write_newline(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "")
  }

  fn write_outputs(&self) -> io::Result<()> {
    let project_name: &str = self.project_data.get_project_name();
    let include_prefix: &str = self.project_data.get_include_prefix();

    let src_root_varname: String = format!("{}_SRC_ROOT", project_name);
    let include_root_varname: String = format!("{}_HEADER_ROOT", project_name);
    let template_impls_root_varname: String = format!("{}_TEMPLATE_IMPLS_ROOT", project_name);
    
    let project_include_dir_varname: String = format!("{}_INCLUDE_DIR", project_name);

    let src_var_name: String = format!("{}_SOURCES", project_name);
    let includes_var_name: String = format!("{}_HEADERS", project_name);
    let template_impls_var_name: String = format!("{}_TEMPLATE_IMPLS", project_name);

    self.write_newline()?;

    self.set_basic_var(&src_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/src/{}", include_prefix))?;
    self.set_basic_var(&include_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/include/{}", include_prefix))?;
    self.set_basic_var(&template_impls_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/template_impls/{}", include_prefix))?;
    self.set_basic_var(&project_include_dir_varname, "${CMAKE_CURRENT_SOURCE_DIR}/include")?;

    self.write_newline()?;

    self.set_file_collection(
      &src_var_name,
      self.project_data.get_src_dir(),
      &src_root_varname,
      &self.project_data.src_files
    )?;

    self.set_file_collection(
      &includes_var_name,
      self.project_data.get_include_dir(),
      &include_root_varname,
      &self.project_data.include_files
    )?;

    self.set_file_collection(
      &template_impls_var_name,
      self.project_data.get_template_impl_dir(),
      &template_impls_root_varname,
      &self.project_data.template_impl_files
    )?;


    // Write the actual outputs
    for (output_name, output_data) in self.project_data.get_outputs() {
      // TODO: Write libraries
      match *output_data.get_output_type() {
        CompiledItemType::Executable => {
          writeln!(&self.cmakelists_file,
            "add_executable( {}\n\t# SOURCES\n\t\t{} ${{{}}}\n\t# HEADERS\n\t\t${{{}}} ${{{}}}\n)",
            output_name,
            format!("${{CMAKE_CURRENT_SOURCE_DIR}}/{}", output_data.get_entry_file().replace("./", "")),
            src_var_name,
            includes_var_name,
            template_impls_var_name
          )?;

          writeln!(&self.cmakelists_file,
            "target_include_directories( {}\n\tPRIVATE ${{{}}}\n)",
            output_name,
            &project_include_dir_varname
          )?;
        },
        _ => {
          println!("TODO: Write library.");
        }
      }
    }

    Ok(())
  }
}
