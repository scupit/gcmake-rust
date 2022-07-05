use std::{collections::{HashMap, HashSet}, fs::File, io::{self, Write}, path::{Path, PathBuf}, rc::Rc, cell::RefCell, ops::{Deref}};

use crate::{file_writers::cmake_utils_writer::CMakeUtilWriter, project_info::{final_project_data::{FinalProjectData, DependencySearchMode, UseableFinalProjectDataGroup}, path_manipulation::cleaned_path_str, final_dependencies::{GitRevisionSpecifier, PredefinedCMakeComponentsModuleDep, PredefinedSubdirDep, PredefinedCMakeModuleDep, FinalPredepInfo}, raw_data_in::{BuildType, BuildConfig, BuildConfigCompilerSpecifier, SpecificCompilerSpecifier, OutputItemType, LanguageConfigMap, TargetSpecificBuildType, dependencies::internal_dep_config::{CMakeModuleType, PredefinedCMakeDepHookFile}}, FinalProjectType, CompiledOutputItem, PreBuildScript, LinkMode}};

use super::cmake_utils_writer::CMakeUtilFile;

const RUNTIME_BUILD_DIR_VAR: &'static str = "${MY_RUNTIME_OUTPUT_DIR}";
const LIB_BUILD_DIR_VAR: &'static str = "${MY_LIBRARY_OUTPUT_DIR}";

pub fn configure_cmake(project_data: &UseableFinalProjectDataGroup) -> io::Result<()> {
  configure_cmake_helper(&project_data.root_project)
}

fn configure_cmake_helper(project_data: &FinalProjectData) -> io::Result<()> {
  for (_, subproject) in project_data.get_subprojects() {
    configure_cmake_helper(subproject)?;
  }

  let cmake_util_path = Path::new(project_data.get_project_root()).join("cmake");
  let util_writer = CMakeUtilWriter::new(cmake_util_path);

  if project_data.is_root_project() {
    util_writer.write_cmake_utils()?;
  }

  let mut cmake_configurer = CMakeListsWriter::new(project_data, util_writer)?;
  cmake_configurer.write_cmakelists()?;
  
  if project_data.is_root_project() {
    cmake_configurer.write_cmake_config_in()?;
  }
  Ok(())
}

fn compiler_matcher_string(compiler_specifier: &SpecificCompilerSpecifier) -> &str {
  match compiler_specifier {
    SpecificCompilerSpecifier::GCC => "USING_GCC",
    SpecificCompilerSpecifier::Clang => "USING_CLANG",
    SpecificCompilerSpecifier::MSVC => "USING_MSVC"
  }
}

fn defines_generator_string(build_type: &BuildType, build_config: &BuildConfig) -> Option<String> {
  return build_config.defines.as_ref()
    .map(|defines| {
      let defines_list = flattened_defines_list_string(defines, DefinesStringFormat::NoQuote);
      format!(
        "\"$<$<CONFIG:{}>:{}>\"",
        build_type.name_string(),
        defines_list
      )
    })
}

enum DefinesStringFormat {
  Quoted,
  NoQuote
}

fn flattened_defines_list_string(
  defines: &HashSet<String>,
  defines_format: DefinesStringFormat
) -> String {
  defines.iter()
    .map(|def| {
      let escaped: String = def.replace('"', "\\\"");

      if let DefinesStringFormat::Quoted = defines_format
        { format!("\"{}\"", escaped) }
        else { escaped }
    })
    .collect::<Vec<String>>()
    .join(";")
}

fn flattened_compiler_flags_string(compiler_flags: &HashSet<String>) -> String {
  let flattened_string: String = compiler_flags.iter()
    .map(|flag| &flag[..])
    .collect::<Vec<&str>>()
    .join(" ");

  return format!(" {} ", flattened_string)
}

fn flattened_maybe_compiler_flags_string(maybe_flags: &Option<HashSet<String>>) -> String {
  return match maybe_flags {
    Some(compiler_flags) => flattened_compiler_flags_string(compiler_flags),
    None => String::from(" ")
  }
}

fn flattened_linker_flags_string(linker_flags: &HashSet<String>) -> String {
  let comma_separated_flags: String = linker_flags.iter()
    .map(|flag| flag.trim())
    .collect::<Vec<&str>>()
    .join(",");

  format!("\"LINKER:{}\"", comma_separated_flags)
}

fn flattened_maybe_linker_flags_string(maybe_flags: &Option<HashSet<String>>) -> String {
  return match maybe_flags {
    Some(linker_flags) => flattened_linker_flags_string(linker_flags),
    None => String::from(" ")
  }
}

struct CMakeListsWriter<'a> {
  project_data: &'a FinalProjectData,
  util_writer: CMakeUtilWriter,
  cmakelists_file: File,
  cmake_config_in_file: File,
}

impl<'a> CMakeListsWriter<'a> {
  fn new(
    project_data: &'a FinalProjectData,
    util_writer: CMakeUtilWriter
  ) -> io::Result<Self> {
    let cmakelists_file_name: String = format!("{}/CMakeLists.txt", project_data.get_project_root());
    let cmake_config_in_file_name: String = format!("{}/Config.cmake.in", project_data.get_project_root());


    Ok(Self {
      project_data,
      util_writer,
      cmakelists_file: File::create(cmakelists_file_name)?,
      cmake_config_in_file: File::create(cmake_config_in_file_name)?
    })
  }

  // This function is only run when using a root project.
  fn write_cmake_config_in(&mut self) -> io::Result<()> {
    writeln!(self.cmake_config_in_file, "@PACKAGE_INIT@\n")?;

    writeln!(
      self.cmake_config_in_file,
      "include( CMakeFindDependencyMacro )"
    )?;

    // Collect all PUBLIC/INTERFACE linked find_module and find_module_components
    // dependencies (+ all components) from the whole project. Import them here as if they
    // were being used in the CMakeLists file (including all cmake hooks).

    // All PUBLIC/INTERFACE linked subdirectory dependencies targets were already added as install targets
    // to the current project's export set, and were also installed by their respective projects.
    // Since they were built as part of the "current project", nothing needs to happen with those.

    // All PUBLIC/INTERFACE linked gcmake dependency targets should have already been added as install
    // targets to the current project's export set. This means their headers were installed too, since
    // FILE_SET is used by gcmake. 
    // The question then is, how to find_package that project's dependencies? It's possible to just include
    // the project's toplevel CMake.Config.in, but that would try to retrieve all the dependencies needed
    // by the that project (including dependencies for its executables, etc.). We only need the dependencies
    // of its libraries which were PUBLIC/INTERFACE linked to our project.
    // Looks like we might be able to recurse through all gcmake dependencies and collect a flattened list
    // to generate (just like everything above) using the same method as a regular project.

    // Actually you know what, it's fine to just include it. Successfully building and installing the
    // GCMake dependency as part of your project means that all the dependencies needed by the
    // GCMake dependency are already present. Since the dependencies of the GCMake dependency are
    // 'inherited' by this project, including the CMake.Config.in of the GCMake dependency is necesary
    // to ensure all needed system dependencies are available when importing the current project
    // using find_package.

    // if this project PUBLIC/INTERFACE links any libraries from a gcmake_dependency,
    // include the Config.CMake.in file of the gcmake_dependency project.

    // TODO: Definitely refactor this. This type is super ugly.
    let needed_find_components_module_targets: Rc<RefCell<HashMap<String, (HashSet<String>, Vec<String>, CMakeModuleType)>>> =
      Rc::new(RefCell::new(HashMap::new()));
    let needed_find_module_targets: Rc<RefCell<HashMap<String, CMakeModuleType>>> =
      Rc::new(RefCell::new(HashMap::new()));
    let needed_public_gcmake_projects: Rc<RefCell<HashSet<String>>> =
      Rc::new(RefCell::new(HashSet::new()));

    self.project_data.recurse_subprojects_and_current(&|project| {
      // This is annoying.
      let mut components_map_borrow = needed_find_components_module_targets.deref().borrow_mut();
      let mut singles_map_borrow = needed_find_module_targets.deref().borrow_mut();
      let mut gcmake_map_borrow = needed_public_gcmake_projects.deref().borrow_mut();

      for (_, output_item) in project.get_outputs() {
        for ref lib_container_name in output_item.get_links().all_projects_linked() {
          let link_view = output_item.get_links().get(lib_container_name).unwrap();
          let link_iterator = link_view
            .iter_by_link_mode([LinkMode::Public, LinkMode::Interface]);

          if let Some(_) = project.get_gcmake_dependencies().get(lib_container_name) {
            if link_iterator.count() > 0 {
              gcmake_map_borrow.insert(lib_container_name.to_string());
            }
          }
          else if let Some(predef_dep) = project.get_predefined_dependencies().get(lib_container_name) {
            match predef_dep.predefined_dep_info() {
              FinalPredepInfo::Subdirectory(_) => {
                // Nothing needs to happen here, since the subdirectory dependencies are installed
                // with this project by default.
              },
              FinalPredepInfo::CMakeComponentsModule(components_dep) => {
                // NOTE that this always creates the entry, even if it isn't needed.
                // Make sure to filter out entries with empty 'used component' lists later.
                let (used_targets, ordered_targets, _) = components_map_borrow
                  .entry(lib_container_name.clone())
                  .or_insert((
                    HashSet::new(),
                    Vec::new(),
                    components_dep.module_type().clone()
                  ));
                
                link_iterator
                  .for_each(|target_name| {
                    if !used_targets.contains(target_name) {
                      used_targets.insert(target_name.to_string());
                      ordered_targets.push(target_name.to_string());
                    }
                  })
              },
              FinalPredepInfo::CMakeModule(cmake_module) => {
                if link_iterator.count() > 0 {
                  singles_map_borrow.insert(
                    lib_container_name.to_string(),
                    cmake_module.module_type().clone()
                  );
                }
              }
            }
          }
        }
      }
    });

    for (dep_name, (_, ordered_components, module_type)) in needed_find_components_module_targets.deref().borrow().iter() {
      if ordered_components.is_empty() {
        continue;
      }

      let module_type_string: &str = match module_type {
        CMakeModuleType::ConfigFile => "CONFIG",
        CMakeModuleType::FindModule => "MODULE",
      };

      write!(&self.cmake_config_in_file,
        "find_dependency( {} {} COMPONENTS",
        dep_name,
        module_type_string
      )?;

      for component_name in ordered_components {
        write!(&self.cmake_config_in_file,
          " {}",
          component_name
        )?;
      }

      writeln!(&self.cmake_config_in_file, " )")?;
    }

    for (dep_name, module_type) in needed_find_module_targets.deref().borrow().iter() {
      let module_type_string: &str = match module_type {
        CMakeModuleType::ConfigFile => "CONFIG",
        CMakeModuleType::FindModule => "MODULE",
      };

      writeln!(&self.cmake_config_in_file,
        "find_dependency( {} {} )",
        dep_name,
        module_type_string
      )?;
    }

    // Is this how this should work?
    // I'm not sure if gcmake projects should be searched for using find_dependency in the install
    // directory, Or if their Config.CMake.in file should be included directly into this one (note
    // that the inclusion is recursive).
    // I'm leaning toward find_dependency, because that includes the gcmake project's package config
    // file anyways, which has the same effect. And since all gcmake projects are built as part of the
    // main project, their config files, headers, and libraries should always be installed with the
    // main project. This means including the package config file for gcmake dependencies should
    // always be reliable AS LONG AS THE GCMAKE DEPENDENCY EXISTS IN THE TREE.

    // I need to specify that CMake needs to search the current project's install tree.
    // Not sure if I need to use CMAKE_CURRENT_LIST_DIR or CMAKE_INSTALL_PREFIX. Probably
    // CMAKE_INSTALL_PREFIX.

    for gcmake_dep_name in needed_public_gcmake_projects.deref().borrow().iter() {
      writeln!(&self.cmake_config_in_file,
        "find_dependency( {} \n\tPATHS\n\t\t\"${{CMAKE_CURRENT_LIST_DIR}}/../{}\"\n)",
        gcmake_dep_name,
        gcmake_dep_name
      )?
    }

    writeln!(&self.cmake_config_in_file,
      "include( \"${{CMAKE_CURRENT_LIST_DIR}}/{}Targets.cmake\" )",
      self.project_data.get_project_name()
    )?;
    Ok(())
  }

  fn write_cmakelists(&mut self) -> io::Result<()> {
    let project_type: &FinalProjectType = self.project_data.get_project_type();
    self.write_project_header()?;

    self.include_utils()?;
    self.write_newline()?;

    if self.project_data.is_root_project() {
      self.write_toplevel_tweaks()?;
    }

    if self.project_data.has_predefined_dependencies() {
      self.write_predefined_dependencies()?;
    }

    if self.project_data.has_gcmake_dependencies() {
      self.write_gcmake_dependencies()?;
    }

    self.write_apply_dependencies()?;

    if let FinalProjectType::Root = project_type {
      self.write_section_header("Language Configuration")?;
      self.write_language_config()?;

      self.write_section_header("Build Configurations")?;
      self.write_build_config_section()?;
    }

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

    if self.project_data.is_root_project() {
      self.write_newline()?;
      self.write_toplevel_cpack_config()?;
    }

    Ok(())
  }

  fn write_project_header(&self) -> io::Result<()> {
    // CMake Version header
    // As of writing, latest version is 3.23. This version is required because it added
    // FILE_SET for headers, which makes installing header files much easier.
    writeln!(&self.cmakelists_file, "cmake_minimum_required( VERSION 3.23 )")?;

    // Project metadata
    writeln!(&self.cmakelists_file,
      "project( {}\n\tVERSION {}\n\tDESCRIPTION \"{}\"\n)",
      self.project_data.get_project_name(),
      self.project_data.version.to_string(),
      self.project_data.get_description()
    )?;

    Ok(())
  }

  fn write_toplevel_tweaks(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "ensure_gcmake_config_dirs_exist()")?;
    writeln!(&self.cmakelists_file, "get_property( isMultiConfigGenerator GLOBAL PROPERTY GENERATOR_IS_MULTI_CONFIG)")?;
    self.set_basic_var(
      "",
      "LOCAL_TOPLEVEL_PROJECT_NAME", 
      &format!("\"{}\"", self.project_data.get_project_name())
    )?;
    self.set_basic_var(
      "",
      "TOPLEVEL_INCLUDE_PREFIX",
      self.project_data.get_base_include_prefix()
    )?;
    self.set_basic_var(
      "",
      "TOPLEVEL_PROJECT_DIR",
      "${CMAKE_CURRENT_SOURCE_DIR}"
    )?;
    self.write_newline()?;

    let config_names: Vec<&'static str> = self.project_data.get_build_configs()
      .iter()
      .map(|(build_type, _)| build_type.name_string())
      .collect();

    writeln!(&self.cmakelists_file,
      "if( NOT ${{isMultiConfigGenerator}} )"
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

    writeln!(&self.cmakelists_file,
      "if( ${{isMultiConfigGenerator}} )"
    )?;
      self.set_basic_var("\t", "MY_RUNTIME_OUTPUT_DIR", "\"$<1:${CMAKE_BINARY_DIR}/bin>\"")?;
      self.set_basic_var("\t", "MY_LIBRARY_OUTPUT_DIR", "\"$<1:${CMAKE_BINARY_DIR}/lib>\"")?;
    writeln!(&self.cmakelists_file, "else()")?;
      self.set_basic_var("\t", "MY_RUNTIME_OUTPUT_DIR", "\"${CMAKE_BINARY_DIR}/bin/${CMAKE_BUILD_TYPE}\"")?;
      self.set_basic_var("\t", "MY_LIBRARY_OUTPUT_DIR", "\"${CMAKE_BINARY_DIR}/lib/${CMAKE_BUILD_TYPE}\"")?;
    writeln!(&self.cmakelists_file, "endif()\n")?;

    writeln!(&self.cmakelists_file,
      "if( \"${{CMAKE_CURRENT_SOURCE_DIR}}\" STREQUAL \"${{CMAKE_SOURCE_DIR}}\" )"
    )?;

    self.set_basic_var("\t", "CMAKE_RUNTIME_OUTPUT_DIRECTORY", RUNTIME_BUILD_DIR_VAR)?;
    self.set_basic_var("\t", "CMAKE_LIBRARY_OUTPUT_DIRECTORY", LIB_BUILD_DIR_VAR)?;
    self.set_basic_var("\t", "CMAKE_ARCHIVE_OUTPUT_DIRECTORY", LIB_BUILD_DIR_VAR)?;

    writeln!(&self.cmakelists_file, "endif()")?;
    Ok(())
  }

  fn include_utils(&self) -> io::Result<()> {
    self.write_newline()?;

    if self.project_data.has_any_fetchcontent_ready_dependencies() {
      writeln!(&self.cmakelists_file,
        "if( NOT DEFINED FETCHCONTENT_QUIET )"
      )?;
      self.set_basic_var("", "FETCHCONTENT_QUIET", "OFF CACHE BOOL \"Enables QUIET option for all content population\"")?;
      writeln!(&self.cmakelists_file, "endif()\n")?;
      writeln!(&self.cmakelists_file, "include(FetchContent)")?;
    }

    if self.project_data.is_root_project() {
      for CMakeUtilFile { util_name, .. } in self.util_writer.get_utils() {
        writeln!(&self.cmakelists_file,
          "include( cmake/{}.cmake )",
          util_name
        )?;
      }
    }

    writeln!(&self.cmakelists_file, "initialize_target_list()")?;
    writeln!(&self.cmakelists_file, "initialize_needed_bin_files_list()")?;
    writeln!(&self.cmakelists_file, "initialize_install_list()")?;
    writeln!(&self.cmakelists_file, "initialize_uncached_dep_list()")?;

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
              (String::from("RUNTIME_OUTPUT_DIRECTORY"), format!("{}/prebuild-scripts", RUNTIME_BUILD_DIR_VAR)),
              (String::from("C_EXTENSIONS"), String::from("OFF")),
              (String::from("CXX_EXTENSIONS"), String::from("OFF"))
            ])
          )?;

          self.write_newline()?;

          self.write_flag_and_define_vars_for_output(script_target_name, exe_info)?;
          self.write_defines_for_output(script_target_name, exe_info)?;
          self.write_target_link_options_for_output(script_target_name, exe_info)?;
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
      "copy_resource_dir_if_exists(\n\t${{CMAKE_CURRENT_SOURCE_DIR}}/resources\n\t{}/resources\n\t${{PROJECT_NAME}}-pre-build-step\n)",
      RUNTIME_BUILD_DIR_VAR
    )?;

    Ok(())
  }

  fn write_subproject_includes(&self) -> io::Result<()> {
    for subproject_name in self.project_data.get_subproject_names() {
      writeln!( &self.cmakelists_file,
        "configure_subproject(\n\t\"${{CMAKE_CURRENT_SOURCE_DIR}}/subprojects/{}\"\n)",
        subproject_name
      )?;
    }

    Ok(())
  }

  fn write_language_config(&self) -> io::Result<()> {
    let LanguageConfigMap {
      C,
      Cpp
    } = self.project_data.get_language_info();

    self.write_newline()?;
    self.set_basic_var(
      "",
      "PROJECT_C_LANGUAGE_STANDARD",
      &C.standard.to_string()
    )?;

    self.set_basic_var(
      "",
      "PROJECT_CXX_LANGUAGE_STANDARD",
      &Cpp.standard.to_string()
    )?;

    writeln!(&self.cmakelists_file,
      "\nif( \"${{CMAKE_SOURCE_DIR}}\" STREQUAL \"${{CMAKE_CURRENT_SOURCE_DIR}}\" )"
    )?;

    self.write_message("\t", "${PROJECT_NAME} is using C${PROJECT_C_LANGUAGE_STANDARD}")?;
    self.write_message("\t", "${PROJECT_NAME} is using C++${PROJECT_CXX_LANGUAGE_STANDARD}")?;

    writeln!(&self.cmakelists_file, "endif()")?;

    Ok(())
  }

  fn write_def_list(
    &self,
    spacer: &'static str,
    items: &HashSet<String>
  ) -> io::Result<()> {
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
    for (dep_name, dep_info) in self.project_data.get_predefined_dependencies() {
      if let Some(pre_load) = dep_info.pre_load_script() {
        writeln!(&self.cmakelists_file, "{}", pre_load.contents_ref())?;
      }

      match dep_info.predefined_dep_info() {
        FinalPredepInfo::CMakeModule(find_module_dep) => {
          self.write_predefined_cmake_module_dep(dep_name, find_module_dep)?;
        },
        FinalPredepInfo::CMakeComponentsModule(components_dep) => {
          self.write_predefined_cmake_components_module_dep(dep_name, components_dep)?;
        },
        FinalPredepInfo::Subdirectory(subdir_dep) => {
          self.write_predefined_subdirectory_dependency(dep_name, subdir_dep)?;
        }
      }

      if let Some(post_load) = dep_info.post_load_script() {
        writeln!(&self.cmakelists_file, "{}", post_load.contents_ref())?;
      }
    }

    Ok(())
  }

  fn write_predefined_cmake_module_dep(
    &self,
    dep_name: &str,
    dep_info: &PredefinedCMakeModuleDep
  ) -> io::Result<()> {
    let search_type_spec: &str = match dep_info.module_type() {
      CMakeModuleType::FindModule => "MODULE",
      CMakeModuleType::ConfigFile => "CONFIG"
    };

    writeln!(&self.cmakelists_file,
      "find_package( {} {} REQUIRED )",
      dep_name,
      search_type_spec
    )?;

    writeln!(&self.cmakelists_file,
      "if( NOT {} )\n\tmessage( FATAL_ERROR \"{}\")\nendif()",
      dep_info.found_varname(),
      // TODO: Make a better error message. Include links to relevant pages if possible.
      format!("Dependency '{}' was not found on the system. Please make sure the library is installed on the system.", dep_name)
    )?;

    Ok(())
  }

  fn write_predefined_cmake_components_module_dep(
    &self,
    dep_name: &str,
    dep_info: &PredefinedCMakeComponentsModuleDep
  ) -> io::Result<()> {
    let search_type_spec: &str = match *dep_info.module_type() {
      CMakeModuleType::FindModule => "MODULE",
      CMakeModuleType::ConfigFile => "CONFIG"
    };

    write!(&self.cmakelists_file,
      "find_package( {} {} REQUIRED COMPONENTS ",
      dep_name,
      search_type_spec
    )?;

    for component_name in dep_info.get_ordered_used_components() {
      write!(&self.cmakelists_file,
        "{} ",
        component_name
      )?;
    }
    writeln!(&self.cmakelists_file, ")\n")?;

    writeln!(&self.cmakelists_file,
      "if( NOT {} )\n\tmessage( FATAL_ERROR \"{}\")\nendif()",
      dep_info.found_varname(),
      // TODO: Make a better error message. Include links to relevant pages if possible.
      format!("Dependency '{}' was not found on the system. Please make sure the library is installed on the system.", dep_name)
    )?;

    Ok(())
  }

  fn write_dep_clone_code(
    &self,
    dep_name: &str,
    git_revison: &GitRevisionSpecifier,
    repo_url: &str,
    should_recursive_clone: bool
  ) -> io::Result<()> {
    let git_revision_spec: String = match git_revison {
      GitRevisionSpecifier::Tag(tag_string) => {
        format!("\tGIT_TAG {}", tag_string)
      },
      GitRevisionSpecifier::CommitHash(hash_string) => {
        format!("\tGIT_TAG {}", hash_string)
      }
    };

    let should_rec_clone_string: String = should_recursive_clone.to_string().to_uppercase();

    writeln!(&self.cmakelists_file,
      "if( NOT IS_DIRECTORY \"${{GCMAKE_DEP_CACHE_DIR}}/{}\" )",
      dep_name
    )?;
    writeln!(&self.cmakelists_file,
      "\tFetchContent_Declare(\n\t\tgcmake_cached_{}\n\t\tSOURCE_DIR \"${{GCMAKE_DEP_CACHE_DIR}}/{}\"\n\t\tGIT_REPOSITORY {}\n\t\t{}\n\t\tGIT_PROGRESS TRUE\n\t\tGIT_SHALLOW FALSE\n\t\tGIT_SUBMODULES_RECURSE {}\n\t)",
      dep_name,
      dep_name,
      repo_url,
      git_revision_spec,
      &should_rec_clone_string
    )?;
    writeln!(&self.cmakelists_file,
      "\tappend_to_uncached_dep_list( gcmake_cached_{} )",
      dep_name
    )?;
    writeln!(&self.cmakelists_file, "endif()")?;
    self.write_newline()?;

    writeln!(&self.cmakelists_file,
      "FetchContent_Declare(\n\t{}\n\tSOURCE_DIR ${{CMAKE_CURRENT_SOURCE_DIR}}/dep/{}\n\tGIT_REPOSITORY \"${{GCMAKE_DEP_CACHE_DIR}}/{}\"\n\t{}\n\tGIT_PROGRESS TRUE\n\tGIT_SUBMODULES_RECURSE {}\n)",
      dep_name,
      dep_name,
      dep_name,
      git_revision_spec,
      should_rec_clone_string
    )?;
    
    Ok(())
  }

  fn write_predefined_subdirectory_dependency(
    &self,
    dep_name: &str,
    dep_info: &PredefinedSubdirDep
  ) -> io::Result<()> {
    // Subdir dependencies which have this 'custom import' script
    // might be installed in a weird way due to how CMake's FILE_SET
    // currently populates directories for its files.
    // Because of this, the custom_populate.cmake script for that dependency
    // should set this variable instead.
    if !dep_info.requires_custom_fetchcontent_populate() {
      self.set_basic_var(
        "\n",
        &format!("{}_RELATIVE_DEP_PATH", dep_name),
        dep_info.custom_relative_include_dir_name()
          .as_ref()
          .map_or(dep_name, |dep_name_string| dep_name_string.as_str())
      )?;
    }

    self.write_dep_clone_code(
      dep_name,
      dep_info.revision(),
      dep_info.repo_url(),
      dep_info.should_recursive_clone()
    )?;
    Ok(())
  }

  fn write_gcmake_dependencies(&self) -> io::Result<()> {
    for (dep_name, dep_info) in self.project_data.get_gcmake_dependencies() {
      self.set_basic_var(
        "\n",
        &format!("{}_RELATIVE_DEP_PATH", dep_name),
        &format!("dep/{}", dep_name)
      )?;

      self.write_dep_clone_code(
        dep_name,
        dep_info.revision(),
        dep_info.repo_url(),
        dep_info.should_recursive_clone()
      )?;
    }

    Ok(()) 
  }

  fn write_apply_dependencies(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "expose_uncached_deps()")?;

    if self.project_data.has_any_fetchcontent_ready_dependencies() {
      writeln!(&self.cmakelists_file, "\nFetchContent_MakeAvailable(")?;

      for dep_name in self.project_data.fetchcontent_dep_names() {
        writeln!(&self.cmakelists_file,
          "\t{}",
          dep_name
        )?;
      }
      writeln!(&self.cmakelists_file, ")")?;
    }

    for (dep_name, combined_dep_info) in self.project_data.get_predefined_dependencies() {
      if let FinalPredepInfo::Subdirectory(dep_info) = combined_dep_info.predefined_dep_info() {
        if dep_info.requires_custom_fetchcontent_populate() {
          writeln!(&self.cmakelists_file,
            "\nFetchContent_GetProperties( {} )",
            dep_name
          )?;

          writeln!(&self.cmakelists_file,
            "if( NOT {}_POPULATED )\n\tFetchContent_Populate( {} )",
            dep_name,
            dep_name
          )?;

          // The predefined dependency config loader guarantees that a custom_populate
          // script exists when a subdirectory dependency specifies that it must
          // be populated manually.
          for line in combined_dep_info.custom_populate_script().as_ref().unwrap().contents_ref().lines() {
            writeln!(&self.cmakelists_file,
              "\t{}",
              line
            )?;
          }

          writeln!(&self.cmakelists_file, "endif()")?;
        }
      }
    }

    Ok(())
  }

  fn write_build_configs(&self) -> io::Result<()> {
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
        let compiler_check_string: &str = compiler_matcher_string(compiler);

        writeln!(&self.cmakelists_file,
          "{}if( {} )",
          if_prefix,
          compiler_check_string 
        )?;

        for (config_name, build_config) in config_map {
          let uppercase_config_name: String = config_name.name_string().to_uppercase();

          // Write compiler flags per compiler for each config.
          self.set_basic_var("\t",
            &format!("{}_LOCAL_COMPILER_FLAGS", uppercase_config_name),
            &flattened_maybe_compiler_flags_string(&build_config.compiler_flags)
          )?;

          // Write linker flags per "compiler" for each config
          self.set_basic_var("\t",
            &format!("{}_LOCAL_LINKER_FLAGS", uppercase_config_name),
            &flattened_maybe_linker_flags_string(&build_config.linker_flags)
          )?;
        }

        if let FinalProjectType::Root = self.project_data.get_project_type() {
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

  fn write_build_config_section(&self) -> io::Result<()> {
    self.write_newline()?;
    
    if let Some(def_list) = self.project_data.get_global_defines() {
      self.write_def_list("", def_list)?;
    }

    self.write_global_config_specific_defines()?;
    self.write_newline()?;
    self.write_build_configs()?;
    
    Ok(())
  }

  fn set_basic_var(
    &self,
    spacer: &str,
    var_name: &str,
    var_value: &str
  ) -> io::Result<()> {
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
        OutputItemType::StaticLib
          | OutputItemType::SharedLib
          | OutputItemType::HeaderOnlyLib =>
        {
          self.write_defined_type_library(
            output_data,
            output_name,
            &src_var_name,
            &includes_var_name,
            &template_impls_var_name,
            &project_include_dir_varname
          )?;
        }
        OutputItemType::CompiledLib => {
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
      if let OutputItemType::Executable = output_data.get_output_type() {
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

  fn append_to_target_build_config_options(
    &self,
    spacer: &str,
    output_name: &str,
    build_type: &BuildType,
    build_config: &BuildConfig
  ) -> io::Result<()> {
    let build_type_name_upper: String = build_type.name_string().to_uppercase();

    // Append to defines, linker flags, and compiler flags.
    if let Some(compiler_flags) = &build_config.compiler_flags {
      writeln!(&self.cmakelists_file,
        "{}list( APPEND {}_{}_OWN_COMPILER_FLAGS {} )",
        spacer,
        output_name,
        &build_type_name_upper,
        flattened_compiler_flags_string(compiler_flags)
      )?;
    }

    if let Some(linker_flags) = &build_config.linker_flags {
      writeln!(&self.cmakelists_file,
        "{}list( APPEND {}_{}_OWN_LINKER_FLAGS {} )",
        spacer,
        output_name,
        &build_type_name_upper,
        flattened_linker_flags_string(linker_flags)
      )?;
    }

    if let Some(defines) = &build_config.defines {
      writeln!(&self.cmakelists_file,
        "{}list( APPEND {}_{}_OWN_DEFINES {} )",
        spacer,
        output_name,
        &build_type_name_upper,
        flattened_defines_list_string(defines, DefinesStringFormat::Quoted)
      )?;
    }

    Ok(())
  }

  fn write_flag_and_define_vars_for_output(
    &self,
    output_name: &str,
    output_data: &CompiledOutputItem
  ) -> io::Result<()> {
    // This works because the toplevel project build config is passed down
    // to all the subprojects.
    // Need to set these here for all allowed configs.
    for (build_type, _) in self.project_data.get_build_configs() {
      let build_type_name_upper: String = build_type.name_string().to_uppercase();

      self.set_basic_var(
        "",
        &format!("{}_{}_OWN_LINKER_FLAGS", output_name, &build_type_name_upper),
        &format!("\"${{{}_LOCAL_LINKER_FLAGS}}\"", &build_type_name_upper)
      )?;

      self.set_basic_var(
        "",
        &format!("{}_{}_OWN_COMPILER_FLAGS", output_name, &build_type_name_upper),
        &format!("\"${{{}_LOCAL_COMPILER_FLAGS}}\"", &build_type_name_upper)
      )?;
    }

    if let Some(build_config_map) = output_data.get_build_config_map() {
      if !build_config_map.is_empty() {

        // All configs and all compilers
        if let Some(config_by_compiler) = build_config_map.get(&TargetSpecificBuildType::AllConfigs) {
          if let Some(always_applicable_config) = config_by_compiler.get(&BuildConfigCompilerSpecifier::All) {
            for (build_type, _) in self.project_data.get_build_configs() {
              self.append_to_target_build_config_options(
                "",
                output_name,
                build_type,
                always_applicable_config
              )?;
            }
          }
        }

        let mut any_compiler_config: HashMap<BuildType, &BuildConfig> = HashMap::new();
        let mut by_compiler: HashMap<SpecificCompilerSpecifier, HashMap<TargetSpecificBuildType, &BuildConfig>> = HashMap::new();

        for (build_type, config_by_compiler) in build_config_map {
          for (compiler_or_all, build_config) in config_by_compiler {
            match compiler_or_all {
              BuildConfigCompilerSpecifier::All => {
                // Exclude settings configured for "all" compilers and "all" configs, since those were
                // already written above.
                if let Some(useable_build_type) = build_type.to_general_build_type() {
                  any_compiler_config.insert(useable_build_type, build_config);
                }
              },
              compiler_spec => {
                let specific_specifier: SpecificCompilerSpecifier = compiler_spec.to_specific().unwrap();

                if by_compiler.get(&specific_specifier).is_none() {
                  by_compiler.insert(specific_specifier.clone(), HashMap::new());
                }

                by_compiler.get_mut(&specific_specifier)
                  .unwrap()
                  .insert(build_type.clone(), build_config);
              }
            }
          }
        }

        // Settings for "all" compilers, by config
        for (build_type, config_for_any_compiler) in &any_compiler_config {
          self.append_to_target_build_config_options(
            "",
            output_name,
            build_type,
            config_for_any_compiler
          )?;
        }

        let mut is_first_run: bool = true;

        // Settings by compiler, by config. ('all build type' configs per compiler will also be here).
        for (specific_compiler, config_by_build_type) in by_compiler {
          let if_spec: &str = if is_first_run
            { "if" }
            else { "elseif" };

          is_first_run = false;

          writeln!(&self.cmakelists_file,
            "{}( {} )",
            if_spec,
            compiler_matcher_string(&specific_compiler)
          )?;

          for (given_build_type, build_config) in config_by_build_type {
            if let TargetSpecificBuildType::AllConfigs = &given_build_type {
              // Settings for all build types, for a specific compiler
              for (build_type, _) in self.project_data.get_build_configs() {
                self.append_to_target_build_config_options(
                  "\t",
                  output_name,
                  build_type,
                  build_config
                )?;
              }
            }
            else {
              // Settings for a single build type, for a specific compiler
              self.append_to_target_build_config_options(
                "\t",
                output_name,
                &given_build_type.to_general_build_type().unwrap(),
                build_config
              )?;
            }
          }
        }

        // If is_first_run is false, that means an if block has been written to the CMakeLists
        if !is_first_run {
          writeln!(&self.cmakelists_file,
            "endif()"
          )?;
        }

        for (build_type, build_config) in &any_compiler_config {
          self.append_to_target_build_config_options(
            "",
            output_name,
            build_type,
            build_config
          )?;
        }
      }
    }

    Ok(())
  }

  fn write_defines_for_output(
    &self,
    output_name: &str,
    output_data: &CompiledOutputItem
  ) -> io::Result<()> {
    let inheritance_method: &str = match output_data.get_output_type() {
      OutputItemType::HeaderOnlyLib => "INTERFACE",
      _ => "PRIVATE"
    };

    // Compiler defines
    writeln!(&self.cmakelists_file,
      "target_compile_definitions( {}\n\t{} ",
      output_name,
      inheritance_method
    )?;

    for (config, _) in self.project_data.get_build_configs() {
      writeln!(&self.cmakelists_file,
        "\t\t\"$<$<CONFIG:{}>:${{{}_{}_OWN_DEFINES}}>\"",
        config.name_string(),
        output_name,
        config.name_string().to_uppercase()
      )?;
    }

    writeln!(&self.cmakelists_file,
      ")"
    )?;
    self.write_newline()?;

    Ok(())
  }

  fn write_target_link_options_for_output(
    &self,
    output_name: &str,
    output_data: &CompiledOutputItem
  ) -> io::Result<()> {
    let inheritance_method: &str = match output_data.get_output_type() {
      OutputItemType::HeaderOnlyLib => "INTERFACE",
      _ => "PRIVATE"
    };

    // Linker flags
    writeln!(&self.cmakelists_file,
      "target_link_options( {}\n\t{} ",
      output_name,
      inheritance_method
    )?;

    for (config, _) in self.project_data.get_build_configs() {
      writeln!(&self.cmakelists_file,
        "\t\t\"$<$<CONFIG:{}>:${{{}_{}_OWN_LINKER_FLAGS}}>\"",
        config.name_string(),
        output_name,
        config.name_string().to_uppercase()
      )?;
    }

    writeln!(&self.cmakelists_file,
      ")"
    )?;
    self.write_newline()?;

    Ok(())
  }

  fn write_target_compile_options_for_output(
    &self,
    output_name: &str,
    output_data: &CompiledOutputItem
  ) -> io::Result<()> {
    let inheritance_method: &str = match output_data.get_output_type() {
      OutputItemType::HeaderOnlyLib => "INTERFACE",
      _ => "PRIVATE"
    };

    // Compiler flags
    writeln!(&self.cmakelists_file,
      "target_compile_options( {}\n\t{} ",
      output_name,
      inheritance_method
    )?;

    for (config, _) in self.project_data.get_build_configs() {
      writeln!(&self.cmakelists_file,
        "\t\t\"$<$<CONFIG:{}>:${{{}_{}_OWN_COMPILER_FLAGS}}>\"",
        config.name_string(),
        output_name,
        config.name_string().to_uppercase()
      )?;
    }

    writeln!(&self.cmakelists_file,
      ")"
    )?;
    self.write_newline()?;

    // Language standard and extensions config
    writeln!(&self.cmakelists_file,
      "target_compile_features( {}\n\t{} ",
      output_name,
      inheritance_method
    )?;

    writeln!(&self.cmakelists_file, "\t\tc_std_${{PROJECT_C_LANGUAGE_STANDARD}}")?;
    writeln!(&self.cmakelists_file, "\t\tcxx_std_${{PROJECT_CXX_LANGUAGE_STANDARD}}")?;

    writeln!(&self.cmakelists_file,
      ")"
    )?;

    Ok(())
  }

  fn write_links_for_output(
    &self,
    output_name: &str,
    output_data: &CompiledOutputItem
  ) -> io::Result<()> {
    if output_data.has_links() {
      let links = output_data.get_links();

      writeln!(&self.cmakelists_file,
        "target_link_libraries( {} ",
        output_name
      )?;

      for (output_link_map, link_mode) in links.iter_link_maps() {
        if !output_link_map.is_empty() {
          let inheritance_method: &str = match link_mode {
            LinkMode::Public => "PUBLIC",
            LinkMode::Private => "PRIVATE",
            LinkMode::Interface => "INTERFACE",
          };

          writeln!(&self.cmakelists_file,
            "\t{}",
            inheritance_method
          )?;

          for (project_name, lib_names_linking) in output_link_map {
            let maybe_matching_namespaced_lib_names: Option<Vec<String>> =
              self.project_data.get_namespaced_library_target_names(
                DependencySearchMode::AsParent,
                project_name,
                lib_names_linking
              )
              .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

            match maybe_matching_namespaced_lib_names {
              None => unreachable!("No link libraries were found even though they were defined. This code shouldn't be reached, since library existence is checked when project is configured."),
              Some(namespaced_lib_names) => {
                for namespaced_lib_name in namespaced_lib_names {
                  writeln!(&self.cmakelists_file,
                    "\t\t{}",
                    namespaced_lib_name
                  )?;
                }
              }
            }
          }

        }
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

    let lib_type_string: &'static str = match *output_data.get_output_type() {
      OutputItemType::StaticLib => "STATIC",
      OutputItemType::SharedLib => "SHARED",
      OutputItemType::HeaderOnlyLib => "INTERFACE",
      _ => panic!("Defined type library is not StaticLib or SharedLib, and is not a HeaderOnlyLib")
    };

    writeln!(&self.cmakelists_file,
      "add_library( {} {} )",
      output_name,
      lib_type_string
    )?;

    if let OutputItemType::SharedLib = output_data.get_output_type() {
      writeln!(&self.cmakelists_file,
        "set_target_properties( {}\n\tPROPERTIES\n\t\tWINDOWS_EXPORT_ALL_SYMBOLS TRUE\n)",
        output_name
      )?;
    }

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
    let lib_spec_string: &str = if output_data.is_header_only_type()
      { "HEADER_ONLY_LIB" }
      else { "COMPILED_LIB" };

    writeln!(&self.cmakelists_file,
      "add_library( {} ALIAS {} )",
      self.project_data.prefix_with_namespace(output_name),
      output_name
    )?;
    self.write_newline()?;

    writeln!(&self.cmakelists_file,
      "add_to_target_list( {} )",
      output_name
    )?;
    self.write_newline()?;

    writeln!(&self.cmakelists_file,
      "apply_lib_files( {} {} \"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\" \"${{{}}}\" \"${{{}}}\" \"${{{}}}\" )",
      output_name,
      lib_spec_string,
      output_data.get_entry_file().replace("./", ""),
      src_var_name,
      includes_var_name,
      template_impls_var_name
    )?;

    writeln!(&self.cmakelists_file,
      "apply_include_dirs( {} {} \"${{{}}}\" )",
      output_name,
      lib_spec_string,
      &project_include_dir_varname
    )?;
    self.write_newline()?;

    self.write_properties_for_output(
      output_name,
      &HashMap::from([
        (String::from("RUNTIME_OUTPUT_DIRECTORY"), String::from(RUNTIME_BUILD_DIR_VAR)),
        (String::from("LIBRARY_OUTPUT_DIRECTORY"), String::from(LIB_BUILD_DIR_VAR)),
        (String::from("ARCHIVE_OUTPUT_DIRECTORY"), String::from(LIB_BUILD_DIR_VAR)),
        (String::from("C_EXTENSIONS"), String::from("OFF")),
        (String::from("CXX_EXTENSIONS"), String::from("OFF"))
      ])
    )?;
    self.write_newline()?;

    self.write_depends_on_pre_build(output_name)?;

    self.write_flag_and_define_vars_for_output(output_name, output_data)?;
    self.write_defines_for_output(output_name, output_data)?;
    self.write_target_link_options_for_output(output_name, output_data)?;
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
      "add_to_target_list( {} )",
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
        (String::from("RUNTIME_OUTPUT_DIRECTORY"), String::from(RUNTIME_BUILD_DIR_VAR)),
        (String::from("C_EXTENSIONS"), String::from("OFF")),
        (String::from("CXX_EXTENSIONS"), String::from("OFF"))
      ])
    )?;
    self.write_newline()?;

    self.write_depends_on_pre_build(output_name)?;

    self.write_flag_and_define_vars_for_output(output_name, output_data)?;
    self.write_defines_for_output(output_name, output_data)?;
    self.write_target_link_options_for_output(output_name, output_data)?;
    self.write_target_compile_options_for_output(output_name, output_data)?;
    self.write_newline()?;

    self.write_links_for_output(output_name, output_data)?;
    Ok(())
  }


  fn pass_into_extra_install_targets(
    &self,
    extra_targets_to_install: &mut HashMap<String, String>,
    lib_container: &str,
    lib_names: &Vec<String>
  ) -> io::Result<()> {
    self.project_data.get_namespaced_library_target_names(
      DependencySearchMode::AsParent,
      lib_container,
      lib_names
    )
      .map_err(|err_message| io::Error::new(io::ErrorKind::Other, err_message))?
      .iter()
      .for_each(|namespaced_lib_names|
        for namespaced_target in namespaced_lib_names {
          extra_targets_to_install.insert(
            namespaced_target.to_string(),
            lib_container.to_string()
          );
        }
      );

    Ok(())
  }

  // See this page for help and a good example:
  // https://cmake.org/cmake/help/latest/guide/tutorial/Adding%20Export%20Configuration.html
  fn write_installation_and_exports(&self) -> io::Result<()> {
    let mut extra_targets_to_install: HashMap<String, String> = HashMap::new();

    for (_output_name, output_data) in self.project_data.get_outputs() {
      if output_data.has_links() {
        for ref lib_container in output_data.get_links().all_projects_linked() {
          let used_public_and_interface_links: Vec<String> = output_data
            .get_links()
            .get(lib_container)
            .unwrap()
            .iter_by_link_mode([LinkMode::Public, LinkMode::Interface])
            // .iter_all()
            .map(|the_str| the_str.to_string())
            .collect();

          let has_public_or_interface_links: bool = !used_public_and_interface_links.is_empty();

          if let Some(predep_config) = self.project_data.get_predefined_dependencies().get(lib_container) {
            let requires_local_install: bool = if let FinalPredepInfo::Subdirectory(subdir_dep) = predep_config.predefined_dep_info()
              { subdir_dep.requires_custom_fetchcontent_populate() }
              else { false };

            if requires_local_install {
              let all_used_link_targets: Vec<String> = output_data
                .get_links()
                .get(lib_container)
                .unwrap()
                .iter_all()
                .map(|the_str| the_str.to_string())
                .collect();

              self.pass_into_extra_install_targets(
                &mut extra_targets_to_install,
                lib_container,
                &all_used_link_targets
              )?;
            }
            else if has_public_or_interface_links && predep_config.should_install_if_public_linked() {
              self.pass_into_extra_install_targets(
                &mut extra_targets_to_install,
                lib_container,
                &used_public_and_interface_links
              )?;
            }
          }

          if let Some(_) = self.project_data.get_gcmake_dependencies().get(lib_container) {
            if has_public_or_interface_links {
              self.pass_into_extra_install_targets(
                &mut extra_targets_to_install,
                lib_container,
                &used_public_and_interface_links
              )?;
            }
          }
        }
      }
    }

    for (namespaced_target, container_lib) in extra_targets_to_install {
      writeln!(&self.cmakelists_file,
        "add_to_install_list( {} \"${{{}_RELATIVE_DEP_PATH}}\" )",
        namespaced_target,
        container_lib
      )?;
    }

    writeln!(&self.cmakelists_file, "clean_target_list()")?;
    writeln!(&self.cmakelists_file, "clean_needed_bin_files_list()")?;
    writeln!(&self.cmakelists_file, "clean_install_list()")?;

    match &self.project_data.get_project_type() {
      FinalProjectType::Root => {
        // writeln!(&self.cmakelists_file,
        //   "configure_installation(\n\t\"{}\"\n\t\"{}\"\n)",
        //   "${${PROJECT_NAME}_INSTALLABLE_TARGETS}",
        //   "${${PROJECT_NAME}_needed_bin_files}"
        // )?;
        writeln!(&self.cmakelists_file, "configure_installation()")?;
      },
      FinalProjectType::Subproject(_) => {
        writeln!(&self.cmakelists_file, "raise_target_list()")?;
        writeln!(&self.cmakelists_file, "raise_needed_bin_files_list()")?;
        writeln!(&self.cmakelists_file, "raise_install_list()")?;
      }
    }

    Ok(())
  }

  fn write_toplevel_cpack_config(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "if( \"${{CMAKE_SOURCE_DIR}}\" STREQUAL \"${{CMAKE_CURRENT_SOURCE_DIR}}\" )"
    )?;

    writeln!(&self.cmakelists_file,
      "\tgcmake_configure_cpack( \"{}\" )",
      self.project_data.get_vendor()
    )?;

    writeln!(&self.cmakelists_file, "endif()")?;
    Ok(())
  }
}
