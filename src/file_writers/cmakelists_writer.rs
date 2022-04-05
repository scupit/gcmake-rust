use std::{collections::{HashMap, HashSet}, fs::File, io::{self, Write}, path::{Path, PathBuf}};

use crate::{file_writers::cmake_utils_writer::CMakeUtilWriter, project_info::{final_project_data::{FinalProjectData}, path_manipulation::cleaned_path_str, final_dependencies::GitRevisionSpecifier, raw_data_in::{BuildType, BuildConfig, BuildConfigCompilerSpecifier, SpecificCompilerSpecifier, CompiledItemType, LanguageConfigMap}, FinalProjectType, CompiledOutputItem, PreBuildScript}, logger::exit_error_log};

const RUNTIME_BUILD_DIR: &'static str = "${CMAKE_BINARY_DIR}/bin/${CMAKE_BUILD_TYPE}";
const LIB_BUILD_DIR: &'static str = "${CMAKE_BINARY_DIR}/lib/${CMAKE_BUILD_TYPE}";

pub fn configure_cmake(project_data: &FinalProjectData) -> io::Result<()> {
  for (_, subproject) in project_data.get_subprojects() {
    configure_cmake(subproject)?;
  }

  let cmake_util_path = Path::new(project_data.get_project_root()).join("cmake");
  let util_writer = CMakeUtilWriter::new(cmake_util_path);
  util_writer.write_cmake_utils()?;

  let cmake_configurer = CMakeListsWriter::new(project_data, util_writer)?;
  cmake_configurer.write_cmakelists()?;
  cmake_configurer.write_cmake_config_in()?;
  Ok(())
}

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

fn flattened_flags_string(maybe_flags: &Option<HashSet<String>>) -> String {
  return match maybe_flags {
    Some(flags) => {
      let flattened_string: String = flags.iter()
        .map(|flag| &flag[..])
        .collect::<Vec<&str>>()
        .join(" ");

      format!(" {} ", flattened_string)
    },
    None => String::from(" ")
  }
}

struct CMakeListsWriter<'a> {
  project_data: &'a FinalProjectData,
  util_writer: CMakeUtilWriter,
  cmakelists_file: File,
  cmake_config_in_file: File,
  installable_targets_varname: String
}

impl<'a> CMakeListsWriter<'a> {
  fn new(project_data: &'a FinalProjectData, util_writer: CMakeUtilWriter) -> io::Result<Self> {
    let cmakelists_file_name: String = format!("{}/CMakeLists.txt", project_data.get_project_root());
    let cmake_config_in_file_name: String = format!("{}/Config.cmake.in", project_data.get_project_root());

    Ok(Self {
      project_data,
      util_writer,
      cmakelists_file: File::create(cmakelists_file_name)?,
      cmake_config_in_file: File::create(cmake_config_in_file_name)?,
      installable_targets_varname: format!("{}_INSTALLABLE_TARGETS", project_data.get_project_name())
    })
  }

  fn write_cmake_config_in(&self) -> io::Result<()> {
    writeln!(&self.cmake_config_in_file, "@PACKAGE_INIT@\n")?;
    writeln!(&self.cmake_config_in_file,
      "include( \"${{CMAKE_CURRENT_LIST_DIR}}/{}Targets.cmake\" )",
      self.project_data.get_project_name()
    )?;
    
    Ok(())
  }

  fn write_cmakelists(&self) -> io::Result<()> {
    let project_type: &FinalProjectType = self.project_data.get_project_type();
    self.write_project_header()?;

    self.include_utils()?;
    self.write_newline()?;

    self.write_toplevel_tweaks()?;

    if self.project_data.has_predefined_dependencies() {
      self.write_predefined_dependencies()?;
    }

    if let FinalProjectType::Full = project_type {
      self.write_section_header("Language Configuration")?;
      self.write_language_config()?;
    }

    self.write_section_header("Build Configurations")?;
    self.write_build_config_section(project_type)?;

    // NOTE: All subprojects must be added after build configuration in order to
    // ensure they inherit all build configuration options.
    if self.project_data.has_subprojects() {
      self.write_section_header("Import Subprojects")?;
      self.write_subproject_includes()?;
    }

    self.write_section_header("Pre-build script configuration")?;
    self.write_prebuild_script_use()?;

    self.write_section_header("'resources' build-time directory copier")?;
    self.write_resource_dir_copier()?;

    self.write_section_header("Outputs")?;
    self.write_outputs()?;

    self.write_section_header("Installation and Export configuration")?;
    self.write_installation_and_exports()?;

    Ok(())
  }

  fn write_project_header(&self) -> io::Result<()> {
    // CMake Version header
    // As of writing, latest version is 3.23. This version is required because it added
    // FILE_SET for headers, which makes installing header files much easier.
    writeln!(&self.cmakelists_file, "cmake_minimum_required( VERSION 3.23 )")?;


    // Project metadata
    writeln!(&self.cmakelists_file,
      "project( {} VERSION {} )",
      self.project_data.get_project_name(),
      self.project_data.version.to_string()
    )?;

    self.write_newline()?;
    self.set_basic_var("", "CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS", "true")?;

    Ok(())
  }

  fn write_toplevel_tweaks(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "if( \"${{CMAKE_CURRENT_SOURCE_DIR}}\" STREQUAL \"${{CMAKE_SOURCE_DIR}}\" )"
    )?;

    self.set_basic_var("\t", "CMAKE_RUNTIME_OUTPUT_DIRECTORY", RUNTIME_BUILD_DIR)?;
    self.set_basic_var("\t", "CMAKE_LIBRARY_OUTPUT_DIRECTORY", LIB_BUILD_DIR)?;

    writeln!(&self.cmakelists_file, "endif()")?;
    Ok(())
  }

  fn include_utils(&self) -> io::Result<()> {
    self.write_newline()?;
    writeln!(&self.cmakelists_file, "include(FetchContent)")?;

    for (util_name, _) in self.util_writer.get_utils() {
      writeln!(&self.cmakelists_file,
        "include( cmake/{}.cmake )",
        util_name
      )?;
    }

    self.write_newline()?;
    self.set_basic_var("", "FETCHCONTENT_QUIET", "OFF")?;

    Ok(())
  }

  fn write_prebuild_script_use(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "initialize_prebuild_step()\n")?;
    
    if let Some(prebuild_script) = self.project_data.get_prebuild_script() {
      match prebuild_script {
        PreBuildScript::Exe(exe_info) => {
          let script_target_name: &str = "pre-build-script-${PROJECT_NAME}";

          writeln!(&self.cmakelists_file,
            "add_executable( {} ${{CMAKE_CURRENT_SOURCE_DIR}}/{} )",
            script_target_name,
            exe_info.get_entry_file().replace("./", "")
          )?;

          self.write_properties_for_output(
            script_target_name,
            &HashMap::from([
              (String::from("RUNTIME_OUTPUT_DIRECTORY"), String::from(RUNTIME_BUILD_DIR)),
              (String::from("C_EXTENSIONS"), String::from("OFF")),
              (String::from("CXX_EXTENSIONS"), String::from("OFF"))
            ])
          )?;

          self.write_newline()?;

          self.write_target_compile_options_for_output(script_target_name, exe_info)?;
          self.write_newline()?;

          self.write_links_for_output(script_target_name, exe_info)?;
          self.write_newline()?;

          writeln!(&self.cmakelists_file,
            "use_executable_prebuild_script( {} )",
            script_target_name
          )?;
        },
        PreBuildScript::Python(python_script_path) => {
          writeln!(&self.cmakelists_file,
            "use_python_prebuild_script( ${{CMAKE_CURRENT_SOURCE_DIR}}/{} )",
            python_script_path
          )?;
        }
      }
    }

    Ok(())
  }

  // TODO: Change how this works. Currently, all files and folders in all resource dirs are merged into
  // a single toplevel one. This is bound to cause issues due to files overwriting each other. 
  fn write_resource_dir_copier(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "copy_resource_dir_if_exists(\n\t${{CMAKE_CURRENT_SOURCE_DIR}}/resources\n\t${{CMAKE_BINARY_DIR}}/bin/${{CMAKE_BUILD_TYPE}}/resources\n\t${{PROJECT_NAME}}-pre-build-step\n)"
    )?;

    Ok(())
  }

  fn write_subproject_includes(&self) -> io::Result<()> {
    for subproject_name in self.project_data.get_subproject_names() {
      writeln!( &self.cmakelists_file,
        "configure_subproject(\n\t\"${{CMAKE_CURRENT_SOURCE_DIR}}/subprojects/{}\"\n\t{}\n)",
        subproject_name,
        &self.installable_targets_varname
      )?;
    }

    Ok(())
  }

  fn write_language_config(&self) -> io::Result<()> {
    let LanguageConfigMap {
      C,
      Cpp
    } = self.project_data.get_language_info();

    // Write C info
    self.write_newline()?;
    self.set_basic_var(
      "",
      "PROJECT_C_LANGUAGE_STANDARD",
      &C.standard.to_string()
    )?;

    // TODO: Only write these messages if the project is toplevel.
    // CMAKE_SOURCE_DIR STREQUAL CMAKE_CURRENT_SOURCE_DIR
    self.write_message("", "Using at least C${PROJECT_C_LANGUAGE_STANDARD} standard")?;

    // Write C++ Info
    self.set_basic_var(
      "",
      "PROJECT_CXX_LANGUAGE_STANDARD",
      &Cpp.standard.to_string()
    )?;

    self.write_message("", "Using at least C++${PROJECT_CXX_LANGUAGE_STANDARD} standard")?;
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

  fn write_predefined_dependencies(&self) -> io::Result<()> {
    let mut all_dep_names: Vec<&str> = Vec::new();

    for (dep_name, dep_info) in self.project_data.get_predefined_dependencies() {
      all_dep_names.push(dep_name);

      writeln!(&self.cmakelists_file,
        "\nFetchContent_Declare(\n\t{}\n\tSOURCE_DIR ${{CMAKE_CURRENT_SOURCE_DIR}}/dep/{}\n\tGIT_REPOSITORY {}\n\tGIT_PROGRESS TRUE",
        dep_name,
        dep_name,
        dep_info.repo_url()
      )?;
      
      // TODO: Refactor this
      match dep_info.revision() {
        GitRevisionSpecifier::Tag(tag_string) => {
          writeln!(&self.cmakelists_file,
            "\tGIT_TAG {}",
            tag_string
          )?;
        },
        GitRevisionSpecifier::CommitHash(hash_string) => {
          writeln!(&self.cmakelists_file,
            "\tGIT_TAG {}",
            hash_string
          )?;
        }
      }

      writeln!(&self.cmakelists_file, ")")?;
    }

    writeln!(&self.cmakelists_file, "\nFetchContent_MakeAvailable(")?;

    for dep_name in all_dep_names {
      writeln!(&self.cmakelists_file,
        "\t{}",
        dep_name
      )?;
    }
    writeln!(&self.cmakelists_file, ")")?;

    Ok(())
  }

  fn write_build_configs(&self, project_type: &FinalProjectType) -> io::Result<()> {
    /*
      Compiler
        - <Build/Release...>
          - flags
          - defines
    */

    let mut simplified_map: HashMap<SpecificCompilerSpecifier, HashMap<&BuildType, &BuildConfig>> = HashMap::new();

    for (build_type, build_config) in self.project_data.get_build_configs() {
      for (build_config_compiler, specific_config) in build_config {
        let converted_compiler_specifier: SpecificCompilerSpecifier = match *build_config_compiler {
          BuildConfigCompilerSpecifier::GCC => SpecificCompilerSpecifier::GCC,
          BuildConfigCompilerSpecifier::Clang => SpecificCompilerSpecifier::Clang,
          BuildConfigCompilerSpecifier::MSVC => SpecificCompilerSpecifier::MSVC,
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
        let compiler_check_string: &str = match compiler {
          SpecificCompilerSpecifier::GCC => "${CMAKE_C_COMPILER_ID} MATCHES \"GNU\" OR ${CMAKE_CXX_COMPILER_ID} MATCHES \"GNU\"",
          SpecificCompilerSpecifier::Clang => "${CMAKE_C_COMPILER_ID} MATCHES \"Clang\" OR ${CMAKE_CXX_COMPILER_ID} MATCHES \"Clang\"",
          SpecificCompilerSpecifier::MSVC => "${MSVC}"
        };

        writeln!(&self.cmakelists_file,
          "{}if( {} )",
          if_prefix,
          compiler_check_string 
        )?;

        for (config_name, build_config) in config_map {
          // Write flags per compiler for each config.
          let flags_string: String = flattened_flags_string(&build_config.flags);
          let uppercase_config_name: String = config_name.name_string().to_uppercase();

          self.set_basic_var("\t",
            &format!("{}_BASE_FLAGS", uppercase_config_name),
            &flags_string
          )?;
        }

          
        if let FinalProjectType::Full = project_type {
          let definitions_generator_string: HashSet<String> = config_map
            .iter()
            .map(|(build_type, build_config)| defines_generator_string(build_type, build_config) )
            .filter(|def| def.is_some())
            .map(|def| def.unwrap())
            .collect();

          self.write_def_list("\t", &definitions_generator_string)?;

        }

        has_written_a_config = true;
        if_prefix = "else";
      }
    }

    if has_written_a_config {
      writeln!(&self.cmakelists_file, "endif()")?;
    }
    Ok(())
  }

  fn write_build_config_section(&self, project_type: &FinalProjectType) -> io::Result<()> {
    self.write_newline()?;
    
    if let FinalProjectType::Full = project_type {
      if let Some(def_list) = self.project_data.get_global_defines() {
        self.write_def_list("", def_list)?;
      }

      let config_names: Vec<&'static str> = self.project_data.get_build_configs()
        .iter()
        .map(|(build_type, _)| build_type.name_string())
        .collect();

      writeln!(&self.cmakelists_file, "\nget_property(isMultiConfigGenerator GLOBAL PROPERTY GENERATOR_IS_MULTI_CONFIG)")?;

      writeln!(&self.cmakelists_file,
        "\nif( NOT isMultiConfigGenerator )"
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
    }

    self.write_global_config_specific_defines()?;
    self.write_newline()?;
    self.write_build_configs(project_type)?;
    
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
    let cleaned_file_root:String = cleaned_path_str(file_location_root);

    writeln!(&self.cmakelists_file, "set( {}", var_name)?;
    for file_path in file_path_collection {
      let fixed_file_path = file_path
        .to_string_lossy()
        .replace(&cleaned_file_root, "");

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
    let include_prefix: &str = self.project_data.get_full_include_prefix();

    let src_root_varname: String = format!("{}_SRC_ROOT", project_name);
    let include_root_varname: String = format!("{}_HEADER_ROOT", project_name);
    let template_impls_root_varname: String = format!("{}_TEMPLATE_IMPLS_ROOT", project_name);
    
    let project_include_dir_varname: String = format!("{}_INCLUDE_DIR", project_name);

    let src_var_name: String = format!("{}_SOURCES", project_name);
    let includes_var_name: String = format!("{}_HEADERS", project_name);
    let template_impls_var_name: String = format!("{}_TEMPLATE_IMPLS", project_name);

    self.write_newline()?;

    // Variables shared between all targets in the current project
    self.set_basic_var("", &src_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/src/{}", include_prefix))?;
    self.set_basic_var("", &include_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/include/{}", include_prefix))?;
    self.set_basic_var("", &template_impls_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/template_impls/{}", include_prefix))?;
    self.set_basic_var("", &project_include_dir_varname, "${CMAKE_CURRENT_SOURCE_DIR}/include")?;
    self.set_basic_var("", "PROJECT_INCLUDE_PREFIX", &format!("\"{}\"", self.project_data.get_full_include_prefix()))?;

    self.write_newline()?;

    self.set_file_collection(
      &src_var_name,
      self.project_data.get_src_dir(),
      &src_root_varname,
      &self.project_data.src_files
    )?;
    self.write_newline()?;

    self.set_file_collection(
      &includes_var_name,
      self.project_data.get_include_dir(),
      &include_root_varname,
      &self.project_data.include_files
    )?;
    self.write_newline()?;

    self.set_file_collection(
      &template_impls_var_name,
      self.project_data.get_template_impl_dir(),
      &template_impls_root_varname,
      &self.project_data.template_impl_files
    )?;

    // Write libraries first
    for (output_name, output_data) in self.project_data.get_outputs() {
      match output_data.get_output_type() {
        CompiledItemType::StaticLib | CompiledItemType::SharedLib => {
          self.write_defined_type_library(
            output_data,
            output_name,
            &src_var_name,
            &includes_var_name,
            &template_impls_var_name,
            &project_include_dir_varname
          )?;
        }
        CompiledItemType::Library => {
          self.write_toggle_type_library(
            output_data,
            output_name,
            &src_var_name,
            &includes_var_name,
            &template_impls_var_name,
            &project_include_dir_varname
          )?;
        },
        _ => ()
      }
    }

    // Write executables after libraries, because the library targets must exist before the executables
    // can link to them.
    for (output_name, output_data) in self.project_data.get_outputs() {
      if let CompiledItemType::Executable = output_data.get_output_type() {
        self.write_executable(
          output_data,
          output_name,
          &src_var_name,
          &includes_var_name,
          &template_impls_var_name,
          &project_include_dir_varname
        )?;
      }
    }

    Ok(())
  }

  fn for_each_linkable_library_name<F>(
    &self,
    project_name: &str,
    lib_names: &Vec<String>,
    callback: F
  ) -> io::Result<()>
    where F: Fn(&str) -> io::Result<()>
  {
    if let Some(_subproject) = self.project_data.get_subprojects().get(project_name) {
      for lib_name in lib_names {
        callback(lib_name)?;
      }
    }
    else if let Some(predefined_dep) = self.project_data.get_predefined_dependencies().get(project_name) {
      for non_namespaced_name in lib_names {
        callback(predefined_dep.get_linkable_target_name(non_namespaced_name).unwrap())?;
      }
    }
    else {
      exit_error_log(&format!("Unable to find project with name {} when linking... this shouldn't happen.", project_name));
    }

    Ok(())
  }

  fn write_target_compile_options_for_output(
    &self,
    output_name: &str,
    output_data: &CompiledOutputItem
  ) -> io::Result<()> {
    // Compiler flags
    writeln!(&self.cmakelists_file,
      "target_compile_options( {}\n\tPRIVATE ",
      output_name
    )?;

    for (config, _) in self.project_data.get_build_configs() {
      writeln!(&self.cmakelists_file,
        "\t\t\"$<$<CONFIG:{}>:${{{}_BASE_FLAGS}}>\"",
        config.name_string(),
        config.name_string().to_uppercase()
      )?;
    }

    writeln!(&self.cmakelists_file,
      ")"
    )?;
    self.write_newline()?;

    // Language standard and extensions config
    writeln!(&self.cmakelists_file,
      "target_compile_features( {}\n\tPRIVATE ",
      output_name
    )?;

    writeln!(&self.cmakelists_file, "\t\tc_std_${{PROJECT_C_LANGUAGE_STANDARD}}")?;
    writeln!(&self.cmakelists_file, "\t\tcxx_std_${{PROJECT_CXX_LANGUAGE_STANDARD}}")?;

    writeln!(&self.cmakelists_file,
      ")"
    )?;

    Ok(())
  }

  fn write_links_for_output(&self, output_name: &str, output_data: &CompiledOutputItem) -> io::Result<()> {
    if output_data.has_links() {
      writeln!(&self.cmakelists_file,
        "target_link_libraries( {}",
        output_name
      )?;

      // TODO: Use subproject name for namespacing once namespaced output targets are implemented.
      // Or at least use the subproject name to get the subproject, and derive its namespace name
      // from that.
      for (project_name, lib_names_linking) in output_data.get_links().as_ref().unwrap() {
        self.for_each_linkable_library_name(
          project_name,
          lib_names_linking,
          |lib_name| {
            // TODO: Namespace the libs with the subproject include_prefix.
            // However, I need to implement aliased targets, exports, and install rules first.
            writeln!(&self.cmakelists_file,
              "\t{}",
              lib_name
            )?;
            Ok(())
          }
        )?;
      }

      writeln!(&self.cmakelists_file,
        ")"
      )?;
    }

    Ok(())
  }

  fn write_properties_for_output(
    &self,
    output_name: &str,
    property_map: &HashMap<String, String>
  ) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "set_target_properties( {} PROPERTIES",
      output_name
    )?;

    for (prop_name, prop_value) in property_map {
      writeln!(&self.cmakelists_file,
        "\t{} {}",
        prop_name,
        prop_value
      )?;
    }

    writeln!(&self.cmakelists_file, ")")?;
    Ok(())
  }

  fn write_output_title(&self, output_name: &str) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "\n# ========== {} ==========",
      output_name
    )?;
    Ok(())
  }

  fn write_depends_on_pre_build(&self, output_name: &str) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      // TODO: Move pre-build macro target name into its own variable (in this source code)
      "add_dependencies( {} ${{PROJECT_NAME}}-pre-build-step )",
      output_name
    )?;

    Ok(())
  }

  fn write_defined_type_library(
    &self,
    output_data: &CompiledOutputItem,
    output_name: &str,
    src_var_name: &str,
    includes_var_name: &str,
    template_impls_var_name: &str,
    project_include_dir_varname: &str
  ) -> io::Result<()> {
    self.write_output_title(output_name)?;

    let lib_type_string: &'static str = match output_data.get_output_type() {
      CompiledItemType::StaticLib => "STATIC",
      CompiledItemType::SharedLib => "SHARED",
      _ => panic!("Defined type library does not have a static or shared output type.")
    };

    writeln!(&self.cmakelists_file,
      "add_library( {} {} )",
      output_name,
      lib_type_string
    )?;
    self.write_newline()?;

    self.write_general_library_data(
      output_data,
      output_name,
      project_include_dir_varname,
      includes_var_name,
      template_impls_var_name,
      src_var_name
    )?;

    Ok(()) 
  }

  fn write_toggle_type_library(
    &self,
    output_data: &CompiledOutputItem,
    output_name: &str,
    src_var_name: &str,
    includes_var_name: &str,
    template_impls_var_name: &str,
    project_include_dir_varname: &str
  ) -> io::Result<()> {
    self.write_output_title(output_name)?;

    writeln!(&self.cmakelists_file,
      // TODO: Find a way to get the make_toggle_lib function name at runtime from the CMakeUtilsWriter
      // struct. This could easily cause hard to track bugs if the function name is changed.
      "make_toggle_lib( {} {} )",
      output_name,
      "STATIC"
    )?;
    self.write_newline()?;

    self.write_general_library_data(
      output_data,
      output_name,
      project_include_dir_varname,
      includes_var_name,
      template_impls_var_name,
      src_var_name
    )?;

    Ok(()) 
  }

  fn write_general_library_data(
    &self,
    output_data: &CompiledOutputItem,
    output_name: &str,
    project_include_dir_varname: &str,
    includes_var_name: &str,
    template_impls_var_name: &str,
    src_var_name: &str
  ) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "set( {} \"${{{}}};{}\" )",
      &self.installable_targets_varname,
      &self.installable_targets_varname,
      output_name
    )?;
    self.write_newline()?;

    writeln!(&self.cmakelists_file,
      "apply_lib_files( {} \"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\" \"${{{}}}\" \"${{{}}}\" \"${{{}}}\" )",
      output_name,
      output_data.get_entry_file().replace("./", ""),
      src_var_name,
      includes_var_name,
      template_impls_var_name
    )?;

    writeln!(&self.cmakelists_file,
      "apply_include_dirs( {} COMPILED_LIB \"${{{}}}\" )",
      output_name,
      &project_include_dir_varname
    )?;
    self.write_newline()?;

    self.write_properties_for_output(
      output_name,
      &HashMap::from([
        (String::from("RUNTIME_OUTPUT_DIRECTORY"), String::from(RUNTIME_BUILD_DIR)),
        (String::from("LIBRARY_OUTPUT_DIRECTORY"), String::from(LIB_BUILD_DIR)),
        (String::from("ARCHIVE_OUTPUT_DIRECTORY"), String::from(LIB_BUILD_DIR)),
        (String::from("C_EXTENSIONS"), String::from("OFF")),
        (String::from("CXX_EXTENSIONS"), String::from("OFF"))
      ])
    )?;
    self.write_newline()?;

    self.write_depends_on_pre_build(output_name)?;
    self.write_target_compile_options_for_output(output_name, output_data)?;
    self.write_newline()?;

    self.write_links_for_output(output_name, output_data)?;
    Ok(())
  }

  fn write_executable(
    &self,
    output_data: &CompiledOutputItem,
    output_name: &str,
    src_var_name: &str,
    includes_var_name: &str,
    template_impls_var_name: &str,
    project_include_dir_varname: &str
  ) -> io::Result<()> {
    self.write_output_title(output_name)?;

    writeln!(&self.cmakelists_file,
      "add_executable( {} )",
      output_name
    )?;
    self.write_newline()?;

    writeln!(&self.cmakelists_file,
      "set( {} \"${{{}}};{}\" )",
      &self.installable_targets_varname,
      &self.installable_targets_varname,
      output_name
    )?;

    writeln!(&self.cmakelists_file,
      "apply_include_dirs( {} EXE \"${{{}}}\" )",
      output_name,
      &project_include_dir_varname
    )?;

    writeln!(&self.cmakelists_file,
      "apply_exe_files( {}\n\t\"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\"\n\t\"${{{}}}\"\n\t\"${{{}}}\"\n\t\"${{{}}}\"\n)",
      output_name,
      output_data.get_entry_file().replace("./", ""),
      src_var_name,
      includes_var_name,
      template_impls_var_name
    )?;
    self.write_newline()?;

    self.write_properties_for_output(
      output_name,
      &HashMap::from([
        (String::from("RUNTIME_OUTPUT_DIRECTORY"), String::from(RUNTIME_BUILD_DIR)),
        (String::from("C_EXTENSIONS"), String::from("OFF")),
        (String::from("CXX_EXTENSIONS"), String::from("OFF"))
      ])
    )?;
    self.write_newline()?;

    self.write_depends_on_pre_build(output_name)?;
    self.write_target_compile_options_for_output(output_name, output_data)?;
    self.write_newline()?;

    self.write_links_for_output(output_name, output_data)?;
    Ok(())
  }

  // See this page for help and a good example:
  // https://cmake.org/cmake/help/latest/guide/tutorial/Adding%20Export%20Configuration.html
  fn write_installation_and_exports(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "clean_list( \"${{{}}}\" {} )",
      &self.installable_targets_varname,
      &self.installable_targets_varname
    )?;

    match &self.project_data.get_project_type() {
      FinalProjectType::Full => {
        writeln!(&self.cmakelists_file,
          "configure_installation(\n\t\"{}\"\n\t\"${{{}}}\"\n)",
          self.project_data.get_project_name(),
          &self.installable_targets_varname
        )?;

      writeln!( &self.cmakelists_file,
        "message( \"Full target list: ${{{}}}\")",
        &self.installable_targets_varname
      )?;
      },
      FinalProjectType::Subproject(_) => {
        writeln!(&self.cmakelists_file,
          "raise_target_list( \"${{{}}}\" )",
          &self.installable_targets_varname
        )?;
      }
    }

    Ok(())
  }
}
