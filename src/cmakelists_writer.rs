use std::{borrow::BorrowMut, collections::{HashMap, HashSet}, fmt::format, fs::File, io::{self, Write}, path::PathBuf};

use crate::{data_types::raw_types::{BuildConfig, BuildConfigCompilerSpecifier, BuildType, CompiledItemType, CompilerSpecifier, ImplementationLanguage}, item_resolver::FinalProjectData};

fn defines_generator_string(build_type: &BuildType, build_config: &BuildConfig) -> Option<String> {
  if let Some(defines) = build_config.defines.as_ref() {
    let defines_list = defines.iter()
      .map(|def| &def[..])
      .collect::<Vec<&str>>()
      .join(";");

    Some(format!(
      "\"$<$<CONFIG:{}>:{}>\"",
      build_type.name_string(),
      defines_list
    ))
  } else {
    None
  }
}

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
    self.write_project_header()?;

    self.write_section_header("Language Configuration")?;
    self.write_language_config()?;

    self.write_section_header("Build Configurations")?;
    self.write_build_config_section()?;

    self.write_section_header("Outputs")?;
    self.write_outputs()?;
    Ok(())
  }

  fn write_project_header(&self) -> io::Result<()> {
    // CMake Version header
    writeln!(&self.cmakelists_file, "cmake_minimum_required( VERSION 3.12 )")?;

    // Project metadata
    writeln!(&self.cmakelists_file,
      "project( {} )",
      self.project_data.get_project_name()
    )?;

    // TODO: Set Output directory configuration by config
    // self.set_basic_var("", var_value)

    Ok(())
  }

  fn write_language_config(&self) -> io::Result<()> {
    for (lang, lang_config) in self.project_data.get_language_info() {
      self.write_newline()?;

      match *lang {
        ImplementationLanguage::C => {
          self.set_basic_var(
            "",
            "CMAKE_C_STANDARD",
            &format!("{} CACHE STRING \"C Compiler Standard\"", &lang_config.default_standard)
          )?;

          writeln!(&self.cmakelists_file,
            "set_property( CACHE CMAKE_C_STANDARD PROPERTY STRINGS {} )",
            lang_config.get_sorted_standards().join(" ")
          )?;
        }
        ImplementationLanguage::Cpp => {
          self.set_basic_var(
            "",
            "CMAKE_CXX_STANDARD",
            &format!("{} CACHE STRING \"CXX Compiler Standard\"", &lang_config.default_standard)
          )?;

          writeln!(&self.cmakelists_file,
            "set_property( CACHE CMAKE_CXX_STANDARD PROPERTY STRINGS {} )",
            lang_config.get_sorted_standards().join(" ")
          )?;
        }
      }
    }

    self.write_newline()?;
    self.set_basic_var("", "CMAKE_C_STANDARD_REQUIRED", "ON")?;
    self.set_basic_var("", "CMAKE_C_EXTENSIONS", "OFF")?;

    self.write_newline()?;
    self.set_basic_var("", "CMAKE_CXX_STANDARD_REQUIRED", "ON")?;
    self.set_basic_var("", "CMAKE_CXX_EXTENSIONS", "OFF")?;

    Ok(())
  }

  fn write_def_list(&self, spacer: &'static str, items: &HashSet<String>) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "{}add_compile_definitions(",
      spacer
    )?;
 
    for def in items {
      writeln!(&self.cmakelists_file,
        "{}\t{}",
        spacer,
        def
      )?;
    }

    writeln!(&self.cmakelists_file, "{})", spacer)?;

    Ok(())
  }

  fn write_message(&self, spacer: &str, message: &str) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "{}message( \"{}\" )",
      spacer,
      message
    )?;
    Ok(())
  }

  fn write_global_config_specific_defines(&self) -> io::Result<()> {
    let mut compiler_all_config_map: HashMap<&BuildType, &BuildConfig> = HashMap::new();

    for (build_type, build_config) in self.project_data.get_build_configs() {
      if let Some(all_compilers_config) = build_config.get(&BuildConfigCompilerSpecifier::All) {
        compiler_all_config_map.insert(build_type, all_compilers_config);
      }
    }

    let defines_list: HashSet<String> = compiler_all_config_map.iter()
      .map(|(build_type, build_config)| defines_generator_string(build_type, build_config))
      .filter(|def| def.is_some())
      .map(|def| def.unwrap())
      .collect();

    self.write_def_list("", &defines_list)?;

    Ok(())
  }

  fn write_build_configs(&self) -> io::Result<()> {
    /*
      Compiler
        - <Build/Release...>
          - flags
          - defines
    */

    let mut simplified_map: HashMap<CompilerSpecifier, HashMap<&BuildType, &BuildConfig>> = HashMap::new();

    for (build_type, build_config) in self.project_data.get_build_configs() {
      for (build_config_compiler, specific_config) in build_config {
        let converted_compiler_specifier: CompilerSpecifier = match *build_config_compiler {
          BuildConfigCompilerSpecifier::GCC => CompilerSpecifier::GCC,
          BuildConfigCompilerSpecifier::Clang => CompilerSpecifier::Clang,
          BuildConfigCompilerSpecifier::MSVC => CompilerSpecifier::MSVC,
          BuildConfigCompilerSpecifier::All => continue
        };

        if simplified_map.get(&converted_compiler_specifier).is_none() {
          simplified_map.insert(converted_compiler_specifier, HashMap::new());
        }

        simplified_map.get_mut(&converted_compiler_specifier)
          .unwrap()
          .insert(build_type, specific_config);
      }
    }

    let mut has_written_a_config: bool = false;
    let mut if_prefix: &str = "";

    for (compiler, config_map) in &simplified_map {
      if !config_map.is_empty() {
        // TODO: Make these strings global, otherwise a simple change to any name could mess all these up.
        let using_compiler_varname: &str = match compiler {
          CompilerSpecifier::GCC => "is_using_GCC",
          CompilerSpecifier::Clang => "is_using_Clang",
          CompilerSpecifier::MSVC => "is_using_MSVC"
        };

        writeln!(&self.cmakelists_file,
          "{}if( \"${{{}}}\" )",
          if_prefix,
          using_compiler_varname 
        )?;

        for (config_name, build_config) in config_map {
          // Write flags per compiler for each config.
          let mut flags_string: String = build_config.flags
            .as_ref()
            .unwrap_or(&HashSet::new())
            .iter()
            .map(|flag| &flag[..])
            .collect::<Vec<&str>>()
            .join(" ");
          
          flags_string = format!("\"{}\" ", flags_string);

          self.set_basic_var("\t",
            &format!("CMAKE_C_FLAGS_{}", config_name.name_string().to_uppercase()),
            &flags_string
          )?;

          self.set_basic_var("\t",
            &format!("CMAKE_CXX_FLAGS_{}", config_name.name_string().to_uppercase()),
            &flags_string
          )?;
        }

          
        let definitions_generator_string: HashSet<String> = config_map
          .iter()
          .map(|(build_type, build_config)| defines_generator_string(build_type, build_config) )
          .filter(|def| def.is_some())
          .map(|def| def.unwrap())
          .collect();

        self.write_newline()?;
        self.write_def_list("\t", &definitions_generator_string)?;

        has_written_a_config = true;
        if_prefix = "else";
      }
    }

    if has_written_a_config {
      writeln!(&self.cmakelists_file, "endif()")?;
    }
    Ok(())
  }

  fn write_build_config_section(&self) -> io::Result<()> {
    self.set_basic_var("", "is_using_GCC", "${CMAKE_C_COMPILER_ID} STREQUAL \"GNU\" OR ${CMAKE_CXX_COMPILER_ID} STREQUAL \"GNU\"")?;
    self.set_basic_var("", "is_using_Clang", " ${CMAKE_C_COMPILER_ID} MATCHES \"Clang\" OR ${CMAKE_CXX_COMPILER_ID} MATCHES \"Clang\"")?;
    self.set_basic_var("", "is_using_MSVC", "${MSVC}")?;

    // We will use configuration specific values to populate these later. However, they must be set
    // to empty because the configuration specific values only append to these variables.
    self.set_basic_var("", "CMAKE_C_FLAGS", "")?;
    self.set_basic_var("", "CMAKE_CXX_FLAGS", "")?;

    self.write_def_list("", self.project_data.get_global_defines())?;

    let config_names: Vec<&'static str> = self.project_data.get_build_configs()
      .iter()
      .map(|(build_type, _)| build_type.name_string())
      .collect();

    writeln!(&self.cmakelists_file,
      "\nif( NOT \"${{is_using_MSVC}}\" )"
    )?;

    writeln!(&self.cmakelists_file,
      "\tset_property( CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS {} )",
      config_names.join(" ")
    )?;

    writeln!(&self.cmakelists_file,
      "\n\tif( \"${{CMAKE_BUILD_TYPE}}\" STREQUAL \"\")\n\t\tset( CMAKE_BUILD_TYPE \"{}\" CACHE STRING \"Project Build configuration\" FORCE )\n\tendif()",
      self.project_data.get_default_build_config().name_string()
    )?;

    self.write_newline()?;
    self.write_message("\t", "Building configuration: ${CMAKE_BUILD_TYPE}")?;
    writeln!(&self.cmakelists_file, "endif()")?;

    self.write_newline()?;

    self.write_global_config_specific_defines()?;
    self.write_newline()?;
    self.write_build_configs()?;
    
    Ok(())
  }

  fn set_basic_var(&self, spacer: &str, var_name: &str, var_value: &str) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "{}set( {} {} )", spacer, var_name, var_value)?;
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

  fn write_section_header(&self, title: &str) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "\n# ////////////////////////////////////////////////////////////////////////////////")?;
    writeln!(&self.cmakelists_file, "# {}", title)?;
    writeln!(&self.cmakelists_file, "# ////////////////////////////////////////////////////////////////////////////////")?;
    Ok(())
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

    self.set_basic_var("", &src_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/src/{}", include_prefix))?;
    self.set_basic_var("", &include_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/include/{}", include_prefix))?;
    self.set_basic_var("", &template_impls_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/template_impls/{}", include_prefix))?;
    self.set_basic_var("", &project_include_dir_varname, "${CMAKE_CURRENT_SOURCE_DIR}/include")?;

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
      self.write_newline()?;

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
          self.write_newline()?;

          writeln!(&self.cmakelists_file,
            "target_include_directories( {}\n\tPRIVATE ${{{}}}\n)",
            output_name,
            &project_include_dir_varname
          )?;
          self.write_newline()?;

          writeln!(&self.cmakelists_file,
            "set_target_properties( {} PROPERTIES\n\tRUNTIME_OUTPUT_DIRECTORY ${{CMAKE_BINARY_DIR}}/bin/${{CMAKE_BUILD_TYPE}}\n)",
            output_name
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
