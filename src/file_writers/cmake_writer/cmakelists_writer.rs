use std::{collections::{HashMap, HashSet}, fs::File, io::{self, Write, ErrorKind}, path::{PathBuf, Path}, rc::Rc, cell::{RefCell, Ref}, borrow::Borrow};

use crate::{project_info::{final_project_data::{FinalProjectData}, path_manipulation::{cleaned_path_str, relative_to_project_root}, final_dependencies::{GitRevisionSpecifier, PredefinedCMakeComponentsModuleDep, PredefinedSubdirDep, PredefinedCMakeModuleDep, FinalPredepInfo, GCMakeDependencyStatus, FinalPredefinedDependencyConfig, encoded_repo_url, PredefinedDepFunctionality}, raw_data_in::{BuildType, RawBuildConfig, BuildConfigCompilerSpecifier, SpecificCompilerSpecifier, OutputItemType, LanguageConfigMap, TargetSpecificBuildType, dependencies::internal_dep_config::{CMakeModuleType}, DefaultCompiledLibType}, FinalProjectType, CompiledOutputItem, PreBuildScript, LinkMode, FinalTestFramework, dependency_graph_mod::dependency_graph::{DependencyGraph, OrderedTargetInfo, ProjectWrapper, TargetNode, SimpleNodeOutputType, Link, EmscriptenLinkFlagInfo}, SystemSpecifierWrapper, SingleSystemSpec, CompilerDefine, FinalBuildConfig, CompilerFlag, LinkerFlag, gcmake_constants::{SRC_DIR, INCLUDE_DIR, TEMPLATE_IMPL_DIR}, platform_spec_parser::parse_leading_system_spec}, file_writers::cmake_writer::cmake_writer_helpers::system_constraint_expression};

use super::cmake_utils_writer::{CMakeUtilFile, CMakeUtilWriter};

const RUNTIME_BUILD_DIR_VAR: &'static str = "${MY_RUNTIME_OUTPUT_DIR}";
const LIB_BUILD_DIR_VAR: &'static str = "${MY_LIBRARY_OUTPUT_DIR}";

pub fn configure_cmake_helper<'a>(
  dep_graph: &Rc<RefCell<DependencyGraph<'a>>>,
  sorted_target_info: &'a OrderedTargetInfo<'a>
) -> io::Result<()> {
  let borrowed_graph = dep_graph.as_ref().borrow();

  for(_, gcmake_dep) in borrowed_graph.get_gcmake_dependencies() {
    configure_cmake_helper(gcmake_dep, sorted_target_info)?;
  }

  for (_, test_project_graph) in borrowed_graph.get_test_projects() {
    configure_cmake_helper(test_project_graph, sorted_target_info)?;
  }

  for (_, subproject_graph) in borrowed_graph.get_subprojects() {
    configure_cmake_helper(subproject_graph, sorted_target_info)?;
  }

  if let Some(project_data) = borrowed_graph.project_wrapper().maybe_normal_project() {
    let cmake_util_path = Path::new(project_data.get_project_root()).join("cmake");
    let maybe_util_writer: Option<CMakeUtilWriter> = if project_data.is_root_project()
      { Some(CMakeUtilWriter::new(cmake_util_path)) }
      else { None };

    if let Some(util_writer) = &maybe_util_writer {
      util_writer.write_cmake_utils()?;
    }

    let mut cmake_configurer = CMakeListsWriter::new(
      Rc::clone(dep_graph),
      sorted_target_info,
      maybe_util_writer
    )?;

    cmake_configurer.write_cmakelists()?;
    
    if project_data.is_root_project() {
      cmake_configurer.write_cmake_config_in()?;
    }
  }

  Ok(())
}

fn compiler_matcher_string(compiler_specifier: &SpecificCompilerSpecifier) -> &str {
  match compiler_specifier {
    SpecificCompilerSpecifier::GCC => "USING_GCC",
    SpecificCompilerSpecifier::Clang => "USING_CLANG",
    SpecificCompilerSpecifier::MSVC => "USING_MSVC",
    SpecificCompilerSpecifier::Emscripten => "USING_EMSCRIPTEN"
  }
}

fn quote_escaped_string(some_str: &str) -> String {
  some_str.replace('"', "\\\"")
}

fn on_or_off_str(value: bool) -> &'static str {
  return if value
    { "ON" }
    else { "OFF" };
}

fn flattened_defines_list_string(spacer: &str, defines: &Vec<CompilerDefine>) -> String {
  defines.iter()
    .map(|CompilerDefine { system_spec, def_string }| {
      let escaped: String = system_constraint_expression(
        system_spec,
        &quote_escaped_string(def_string.trim())
      );

      format!("\"{}\"", escaped)
    })
    .collect::<Vec<String>>()
    .join(&format!("\n{}", spacer))
}

fn flattened_compiler_flags_string(spacer: &str, compiler_flags: &Vec<CompilerFlag>) -> String {
  let flattened_string: String = compiler_flags.iter()
    .map(|CompilerFlag { system_spec, flag_string }| {
      let escaped: String = system_constraint_expression(
        system_spec,
        &quote_escaped_string(flag_string.trim())
      );

      format!("\"{}\"", escaped)
    })
    .collect::<Vec<String>>()
    .join(&format!("\n{}", spacer));

  return format!(" {} ", flattened_string)
}

fn flattened_linker_flags_string(linker_flags: &Vec<LinkerFlag>) -> String {
  let comma_separated_flags: String = linker_flags.iter()
    .map(|LinkerFlag { system_spec, flag_string }|
      system_constraint_expression(
        system_spec,
        &quote_escaped_string(flag_string.trim())
      )
    )
    .collect::<Vec<String>>()
    .join(",");

  format!("\"LINKER:{}\"", comma_separated_flags)
}

struct CMakeListsWriter<'a> {
  dep_graph: Rc<RefCell<DependencyGraph<'a>>>,
  sorted_target_info: &'a OrderedTargetInfo<'a>,
  project_data: Rc<FinalProjectData>,
  util_writer: Option<CMakeUtilWriter>,
  cmakelists_file: File,
  cmake_config_in_file: Option<File>
}

impl<'a> CMakeListsWriter<'a> {
  fn new(
    dep_graph: Rc<RefCell<DependencyGraph<'a>>>,
    sorted_target_info: &'a OrderedTargetInfo<'a>,
    util_writer: Option<CMakeUtilWriter>
  ) -> io::Result<Self> {
    let borrowed_graph = dep_graph.as_ref().borrow();
    let project_data: Rc<FinalProjectData> = match borrowed_graph.project_wrapper() {
      ProjectWrapper::NormalProject(normal_project) => Rc::clone(normal_project),
      ProjectWrapper::GCMakeDependencyRoot(gcmake_dep) => match gcmake_dep.project_status() {
        GCMakeDependencyStatus::Available(normal_project) => Rc::clone(normal_project),
        GCMakeDependencyStatus::NotDownloaded(_) => {
          return Err(io::Error::new(
            ErrorKind::Other,
            format!(
              "Tried to write a CMakeLists configuration for GCMake dependency '{}' which hasn't been cloned yet.",
              borrowed_graph.project_mangled_name()
            )
          ));
        }
      },
      ProjectWrapper::PredefinedDependency(_) => {
        return Err(io::Error::new(
          ErrorKind::Other,
          format!(
            "Tried to write a CMakeLists configuration for predefined dependency '{}'.",
            borrowed_graph.project_mangled_name()
          )
        ));
      }
    };

    let cmakelists_file_name: String = format!("{}/CMakeLists.txt", project_data.get_project_root());
    let cmake_config_in_file_name: String = format!("{}/Config.cmake.in", project_data.get_project_root());

    drop(borrowed_graph);

    Ok(Self {
      dep_graph,
      sorted_target_info: sorted_target_info,
      cmake_config_in_file: if project_data.is_root_project()
        { Some(File::create(cmake_config_in_file_name)?) } 
        else { None },
      project_data,
      util_writer,
      cmakelists_file: File::create(cmakelists_file_name)?
    })
  }

  // This function is only run when using a root project.
  fn write_cmake_config_in(&mut self) -> io::Result<()> {
    let config_in_file: &mut File = self.cmake_config_in_file.as_mut().unwrap();
    writeln!(config_in_file, "@PACKAGE_INIT@\n")?;

    writeln!(
      config_in_file,
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
    let mut needed_find_components_module_targets: HashMap<String, (Vec<String>, CMakeModuleType)> = HashMap::new();
    let mut needed_find_modules: HashMap<String, CMakeModuleType> = HashMap::new();
    let mut needed_public_gcmake_projects: HashSet<String> = HashSet::new();


    for target in self.sorted_target_info.targets_in_link_order() {
      let borrowed_target: &TargetNode = &target.as_ref().borrow();

      if borrowed_target.should_be_searched_in_package_config() {
        let borrowed_container_graph = borrowed_target.container_project();
        let container_graph = borrowed_container_graph.as_ref().borrow();
        let container_graph_name: String = container_graph.project_mangled_name().to_string();

        match container_graph.project_wrapper() {
          ProjectWrapper::NormalProject(_) => {
            // If the target is built by a GCMake dependency
            if container_graph.root_project_id() != self.dep_graph.as_ref().borrow().root_project_id() {
              needed_public_gcmake_projects.insert(container_graph_name);
            }
          },
          ProjectWrapper::GCMakeDependencyRoot(_) => {
            needed_public_gcmake_projects.insert(container_graph_name);
          },
          ProjectWrapper::PredefinedDependency(predef_dep) => match predef_dep.predefined_dep_info() {
            FinalPredepInfo::Subdirectory(_) => {
              // Nothing needs to happen here, since subdirectory dependencies and their targets are already
              // installed as part of this project.
            },
            FinalPredepInfo::CMakeComponentsModule(components_dep) => {
              let target_name: &str = borrowed_target.get_name();

              needed_find_components_module_targets.entry(container_graph_name)
                .and_modify(|(used_target_list, _)| {
                  // I'm not sure why the compiler is making me use contains with a &String instead
                  // of just &str. I thought &str is fine in most cases.
                  if !used_target_list.contains(&target_name.to_string()) {
                    used_target_list.push(target_name.to_string());
                  }
                })
                .or_insert(
                  (
                    vec![target_name.to_string()],
                    components_dep.module_type().clone()
                  )
                );
            },
            FinalPredepInfo::CMakeModule(module_dep) => {
              if !needed_find_modules.contains_key(&container_graph_name) {
                needed_find_modules.insert(
                  container_graph_name,
                  module_dep.module_type().clone()
                );
              }
            }
          }
        }
      }
    }

    for (dep_name, (ordered_components, module_type)) in needed_find_components_module_targets {
      if ordered_components.is_empty() {
        continue;
      }

      let module_type_string: &str = match module_type {
        CMakeModuleType::ConfigFile => "CONFIG",
        CMakeModuleType::FindModule => "MODULE",
      };

      write!(config_in_file,
        "find_dependency( {} {} COMPONENTS",
        dep_name,
        module_type_string
      )?;

      for component_name in ordered_components {
        write!(config_in_file,
          " {}",
          component_name
        )?;
      }

      writeln!(config_in_file, " )")?;
    }

    for (dep_name, module_type) in needed_find_modules {
      let module_type_string: &str = match module_type {
        CMakeModuleType::ConfigFile => "CONFIG",
        CMakeModuleType::FindModule => "MODULE",
      };

      writeln!(config_in_file,
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

    for gcmake_dep_name in needed_public_gcmake_projects {
      writeln!(config_in_file,
        "find_dependency( {} \n\tPATHS\n\t\t\"${{CMAKE_CURRENT_LIST_DIR}}/../{}\"\n)",
        gcmake_dep_name,
        gcmake_dep_name
      )?
    }

    writeln!(config_in_file,
      "include( \"${{CMAKE_CURRENT_LIST_DIR}}/{}Targets.cmake\" )",
      self.project_data.get_full_namespaced_project_name()
    )?;
    Ok(())
  }

  fn write_cmakelists(&mut self) -> io::Result<()> {
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

    if self.project_data.is_root_project() {
      self.write_apply_dependencies()?;

      self.write_section_header("Language Configuration")?;
      self.write_language_config()?;

      self.write_section_header("Build Configurations")?;
      self.write_build_config_section()?;
    }

    if self.project_data.has_tests() {
      self.write_section_header("Tests Configuration")?;
      self.write_test_config_section()?;
    }

    self.write_project_order_dependent_info()?;

    if !self.project_data.is_test_project() {
      self.write_section_header("Installation and Export configuration")?;
      self.write_installation_and_exports()?;
    }

    if self.project_data.is_root_project() {
      self.write_newline()?;
      self.write_toplevel_cpack_config()?;
    }

    Ok(())
  }

  fn write_pre_build_and_outputs(&self) -> io::Result<()> {
    self.write_section_header("Pre-build script configuration")?;
    self.write_prebuild_script_use()?;

    self.write_section_header("'resources' build-time directory copier")?;
    self.write_resource_dir_copier()?;

    self.write_section_header("Outputs")?;
    self.write_outputs()?;

    Ok(())
  }

  fn write_project_order_dependent_info(&self) -> io::Result<()> {
    if !self.project_data.is_root_project() {
      self.write_pre_build_and_outputs()?;
    }
    else {
      let ordered_projects_in_this_tree = self.sorted_target_info.project_order
        .iter()
        .filter(|wrapped_project| {
          let project_ref = wrapped_project.as_ref().borrow();
          // This works because this "write_subproject_includes" function is only run when
          // self's graph is the project root.
          project_ref.root_project_id() == self.dep_graph_ref().project_id()
        });

      let root_project_info = self.dep_graph_ref().project_wrapper().clone().unwrap_normal_project();
      let root_project_root_path: &str = root_project_info.get_project_root();

      for some_project_graph in ordered_projects_in_this_tree {
        let borrowed_graph = some_project_graph.as_ref().borrow();
        let subproject_data: Rc<FinalProjectData> = borrowed_graph.project_wrapper().clone().unwrap_normal_project();

        if borrowed_graph.project_id() == self.dep_graph_ref().project_id() {
          self.write_pre_build_and_outputs()?;
        }
        else if !subproject_data.is_test_project() {
          writeln!( &self.cmakelists_file,
            "configure_subproject(\n\t\"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\"\n)",
            relative_to_project_root(root_project_root_path, PathBuf::from(subproject_data.get_project_root()))
          )?;
        }
      }
    }

    Ok(())
  }

  fn write_project_header(&self) -> io::Result<()> {
    if self.project_data.is_root_project() {
      // CMake Version header
      // 3.23: FILE_SET functionality is used.
      // 3.24: Updated FindwxWidgets find module needed to use new wxWidgets 3.2.0 release.
      writeln!(&self.cmakelists_file, "cmake_minimum_required( VERSION 3.24 )")?;
    }

    // Project metadata
    writeln!(&self.cmakelists_file,
      "project( {}\n\tVERSION {}\n\tDESCRIPTION \"{}\"\n)",
      self.project_data.get_full_namespaced_project_name(),
      self.project_data.version.to_string(),
      self.project_data.get_description()
    )?;

    Ok(())
  }

  fn write_toplevel_tweaks(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "ensure_gcmake_config_dirs_exist()")?;

    let project_supports_emscripten: bool = self.project_data.supports_emscripten();
    
    writeln!(&self.cmakelists_file, "if( USING_EMSCRIPTEN )")?;
    writeln!(&self.cmakelists_file,
      "\tconfigure_emscripten_mode( Browser )"
    )?;
    writeln!(&self.cmakelists_file, "endif()")?;

    if !self.project_data.supports_emscripten() {
      self.set_basic_option(
        "",
        "GCMAKE_OVERRIDE_EMSCRIPTEN_COMPILATION",
        "OFF",
        "When ON, force-allows Emscripten compilation for projects which don't obviously support copmilation with Emscripten."
      )?;

      writeln!(&self.cmakelists_file,
        "err_if_using_emscripten( GCMAKE_OVERRIDE_EMSCRIPTEN_COMPILATION )"
      )?;
    }

    if !self.project_data.can_trivially_cross_compile() {
      self.set_basic_option(
        "",
        "GCMAKE_OVERRIDE_CROSS_COMPILATION",
        "OFF",
        "When ON, force-allows cross compilation for projects which don't support trivial cross-compilation."
      )?;

      if project_supports_emscripten {
        write!(&self.cmakelists_file, "if( NOT USING_EMSCRIPTEN )\n\t")?;
      }

      writeln!(&self.cmakelists_file,
        "err_if_cross_compiling( GCMAKE_OVERRIDE_CROSS_COMPILATION )"
      )?;

      if project_supports_emscripten {
        writeln!(&self.cmakelists_file, "endif()")?;
      }
    }

    self.write_newline()?;
    writeln!(&self.cmakelists_file, "get_property( isMultiConfigGenerator GLOBAL PROPERTY GENERATOR_IS_MULTI_CONFIG)")?;

    writeln!(&self.cmakelists_file,
      "set( GCMAKE_SANITIZER_FLAGS \"\" CACHE STRING \"SEMICOLON SEPARATED list of sanitizer flags to build the project with. These are included in both compiler flags and linker flags\" )"
    )?;

    writeln!(&self.cmakelists_file,
      "set( GCMAKE_ADDITIONAL_COMPILER_FLAGS \"\" CACHE STRING \"SEMICOLON SEPARATED list of additional compiler flags to build the project with. Useful for static analyzers or flags like -march which shouldn't be included by default\" )"
    )?;

    writeln!(&self.cmakelists_file,
      "set( GCMAKE_ADDITIONAL_LINK_TIME_FLAGS \"\" CACHE STRING \"SEMICOLON SEPARATED list of additional link-time flags to build the project with\" )"
    )?;

    writeln!(&self.cmakelists_file,
      "set( GCMAKE_ADDITIONAL_LINKER_FLAGS \"\" CACHE STRING \"SEMICOLON SEPARATED list of additional linker flags to build the project with\" )"
    )?;

    // Change the default install COMPONENT to play nice with NSIS installers.
    self.set_basic_var(
      "",
      "CMAKE_INSTALL_DEFAULT_COMPONENT_NAME",
      "Dependencies"  // Make sure this isn't all caps. All CAPS names were causing issues with
                      // multi-component NSIS installers.
    )?;

    self.set_basic_var(
      "",
      "LOCAL_TOPLEVEL_PROJECT_NAME", 
      &format!("\"{}\"", self.project_data.get_full_namespaced_project_name())
    )?;
    self.set_basic_var(
      "",
      "TOPLEVEL_INCLUDE_PREFIX",
      self.project_data.get_base_include_prefix()
    )?;
    self.set_basic_var(
      "",
      "TOPLEVEL_PROJECT_DIR",
      "\"${CMAKE_CURRENT_SOURCE_DIR}\""
    )?;
    self.write_newline()?;

    self.set_basic_var(
      "",
      "LOCAL_BUILD_SHARED_LIBS_DOC_STRING",
      "\"Build compiled libraries as SHARED when their type is not explicitly specified\""
    )?;

    self.set_basic_var(
      "",
      "LOCAL_BUILD_STATIC_LIBS_DOC_STRING",
      "\"Build compiled libraries as STATIC when their type is not explicitly specified\""
    )?;

    self.set_basic_var(
      "",
      "LOCAL_CMAKE_BUILD_TYPE_DOC_STRING",
      "\"Which project configuration to build\""
    )?;

    writeln!(&self.cmakelists_file,
      "initialize_lib_type_options( {} )",
      match self.project_data.get_default_compiled_lib_type() {
        DefaultCompiledLibType::Shared => "SHARED",
        DefaultCompiledLibType::Static => "STATIC",
      }
    )?;

    self.set_basic_option(
      "",
      "BUILD_TESTING",
      "OFF",
      "Build the testing tree for all non-GCMake projects. Testing trees for GCMake projects are enabled per-project. For example, this project uses the ${LOCAL_TOPLEVEL_PROJECT_NAME}_BUILD_TESTS variable."
    )?;

    writeln!(&self.cmakelists_file, "\ninitialize_build_tests_var()")?;

    let config_names: Vec<&'static str> = self.project_data.get_build_configs()
      .iter()
      .map(|(build_type, _)| build_type.name_str())
      .collect();

    writeln!(&self.cmakelists_file,
      "if( NOT ${{isMultiConfigGenerator}} )"
    )?;

    writeln!(&self.cmakelists_file,
      "\tset_property( CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS {} )",
      config_names.join(" ")
    )?;

    writeln!(&self.cmakelists_file,
      "\n\tif( \"${{CMAKE_BUILD_TYPE}}\" STREQUAL \"\")\n\t\tset( CMAKE_BUILD_TYPE \"{}\" CACHE STRING \"${{LOCAL_CMAKE_BUILD_TYPE_DOC_STRING}}\" FORCE )\n\tendif()",
      self.project_data.get_default_build_config().name_str()
    )?;
    self.write_newline()?;

    self.write_message("\t", "Building configuration: ${CMAKE_BUILD_TYPE}")?;
    writeln!(&self.cmakelists_file, "endif()")?;
    self.write_newline()?;

    self.set_basic_var("", "MY_RUNTIME_OUTPUT_DIR", "\"${CMAKE_BINARY_DIR}/bin/$<CONFIG>\"")?;
    self.set_basic_var("", "MY_LIBRARY_OUTPUT_DIR", "\"${CMAKE_BINARY_DIR}/lib/$<CONFIG>\"")?;
    self.write_newline()?;

    writeln!(&self.cmakelists_file,
      "if( \"${{CMAKE_CURRENT_SOURCE_DIR}}\" STREQUAL \"${{CMAKE_SOURCE_DIR}}\" )"
    )?;

    // For 'Unix Makefiles' and 'Ninja' generators, CMake will create a compile_commands.json
    // file in the build directory.
    // https://cmake.org/cmake/help/latest/variable/CMAKE_EXPORT_COMPILE_COMMANDS.html
    // https://clang.llvm.org/docs/JSONCompilationDatabase.html
    // This allows clangd (and likely other tools) to more easily work with a project.
    self.set_basic_var("\t", "CMAKE_EXPORT_COMPILE_COMMANDS", "TRUE")?;

    self.set_basic_var("\t", "CMAKE_RUNTIME_OUTPUT_DIRECTORY", RUNTIME_BUILD_DIR_VAR)?;
    self.set_basic_var("\t", "CMAKE_LIBRARY_OUTPUT_DIRECTORY", LIB_BUILD_DIR_VAR)?;
    self.set_basic_var("\t", "CMAKE_ARCHIVE_OUTPUT_DIRECTORY", LIB_BUILD_DIR_VAR)?;

    writeln!(&self.cmakelists_file, "endif()")?;
    self.write_newline()?;

    {
      let ipo_default_status_str: &str = if self.project_data.ipo_enabled_by_default()
        { "ON" }
        else { "OFF" };

      writeln!(&self.cmakelists_file,
        "initialize_ipo_defaults( {} )",
        ipo_default_status_str
      )?;
    }

    writeln!(&self.cmakelists_file,
      "initialize_pgo_defaults()"
    )?;

    Ok(())
  }

  fn include_utils(&self) -> io::Result<()> {
    self.write_newline()?;

    if self.project_data.needs_fetchcontent() {
      writeln!(&self.cmakelists_file,
        "if( NOT DEFINED FETCHCONTENT_QUIET )"
      )?;
      self.set_basic_var("", "FETCHCONTENT_QUIET", "OFF CACHE BOOL \"Enables QUIET option for all content population\"")?;
      writeln!(&self.cmakelists_file, "endif()\n")?;
      writeln!(&self.cmakelists_file, "include(FetchContent)")?;
    }

    if self.project_data.is_root_project() {
      writeln!(&self.cmakelists_file, "include(GenerateExportHeader)")?;
      assert!(
        self.util_writer.is_some(),
        "A CMakeListsWriter for a root project should always have a util_writer."
      );

      for CMakeUtilFile { util_name, .. } in self.util_writer.as_ref().unwrap().get_utils() {
        writeln!(&self.cmakelists_file,
          "include( cmake/{}.cmake )",
          util_name
        )?;
      }
    }

    writeln!(&self.cmakelists_file, "initialize_target_list()")?;
    writeln!(&self.cmakelists_file, "initialize_needed_bin_files_list()")?;
    writeln!(&self.cmakelists_file, "initialize_install_list()")?;
    writeln!(&self.cmakelists_file, "initialize_generated_export_headers_list()")?;
    
    if self.project_data.is_root_project() {
      writeln!(&self.cmakelists_file, "initialize_uncached_dep_list()")?;
      writeln!(&self.cmakelists_file, "initialize_actual_dep_list()")?;
    }

    Ok(())
  }

  fn write_prebuild_script_use(&self) -> io::Result<()> {
    writeln!(
      &self.cmakelists_file,
      "initialize_prebuild_step( \"{}\" )\n",
      self.project_data.prebuild_script_name()
    )?;
    
    if let Some(prebuild_script) = self.project_data.get_prebuild_script() {
      match prebuild_script {
        PreBuildScript::Exe(exe_info) => {
          assert!(
            self.dep_graph_ref().get_pre_build_node().is_some(),
            "When a FinalProjectData contains a pre-build script, the matching dependency graph for the project must contain a pre-build script node."
          );

          let script_target_name: String = self.write_executable(
            exe_info,
            self.dep_graph_ref().get_pre_build_node().as_ref().unwrap(),
            &self.project_data.prebuild_script_name(),
            "UNUSED",
            "UNUSED",
            "UNUSED",
            "UNUSED"
          )?;

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
      "copy_resource_dir_if_exists(\n\t${{CMAKE_CURRENT_SOURCE_DIR}}/resources\n\t{}/resources\n)",
      RUNTIME_BUILD_DIR_VAR
    )?;

    Ok(())
  }

  fn write_language_config(&self) -> io::Result<()> {
    let LanguageConfigMap { c, cpp } = self.project_data.get_language_info();

    self.write_newline()?;
    self.set_basic_var(
      "",
      "PROJECT_C_LANGUAGE_STANDARD",
      &c.standard.to_string()
    )?;

    self.set_basic_var(
      "",
      "PROJECT_CXX_LANGUAGE_STANDARD",
      &cpp.standard.to_string()
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
    items: &Vec<String>
  ) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "{}add_compile_definitions(",
      spacer
    )?;
 
    for fully_processed_def_string in items {
      writeln!(&self.cmakelists_file,
        "{}\t{}",
        spacer,
        fully_processed_def_string
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
    for (build_type, build_config) in self.project_data.get_build_configs() {
      if let Some(all_compilers_config) = build_config.get(&BuildConfigCompilerSpecifier::AllCompilers) {
        if all_compilers_config.has_compiler_defines() {
          writeln!(&self.cmakelists_file,
            "\nlist( APPEND {}_LOCAL_DEFINES\n\t{}\n)",
            build_type.name_str().to_uppercase(),
            flattened_defines_list_string("\t", &all_compilers_config.defines)
          )?;
        }
      }
    }

    Ok(())
  }

  fn write_predefined_dependencies(&self) -> io::Result<()> {
    for (dep_name, predep_graph) in self.dep_graph_ref().get_predefined_dependencies() {
      let dep_info: Rc<FinalPredefinedDependencyConfig> = predep_graph.as_ref().borrow().project_wrapper().clone().unwrap_predef_dep();

      if let Some(pre_load) = dep_info.pre_load_script() {
        writeln!(&self.cmakelists_file, "{}", pre_load.contents_ref())?;
      }

      match dep_info.predefined_dep_info() {
        FinalPredepInfo::CMakeModule(find_module_dep) => {
          self.write_predefined_cmake_module_dep(
            dep_name,
            predep_graph,
            find_module_dep
          )?;
        },
        FinalPredepInfo::CMakeComponentsModule(components_dep) => {
          self.write_predefined_cmake_components_module_dep(
            dep_name,
            predep_graph,
            components_dep
          )?;
        },
        FinalPredepInfo::Subdirectory(subdir_dep) => {
          self.write_predefined_subdirectory_dependency(
            dep_name,
            subdir_dep,
            dep_info.is_auto_fetchcontent_ready()
          )?;
        }
      }
    }

    Ok(())
  }

  fn write_predefined_cmake_module_dep(
    &self,
    dep_name: &str,
    _predep_graph: &Rc<RefCell<DependencyGraph>>,
    dep_info: &PredefinedCMakeModuleDep
  ) -> io::Result<()> {
    let search_type_spec: &str = match dep_info.module_type() {
      CMakeModuleType::FindModule => "MODULE",
      CMakeModuleType::ConfigFile => "CONFIG"
    };

    let is_dep_internally_supported_by_emscripten: bool = dep_info.is_internally_supported_by_emscripten();

    let indent: &str;

    if is_dep_internally_supported_by_emscripten {
      writeln!(&self.cmakelists_file, "if( NOT USING_EMSCRIPTEN )\n")?;
      indent = "\t";
    }
    else {
      indent = "";
    }

    writeln!(&self.cmakelists_file,
      "{}find_package( {} {} REQUIRED )",
      indent,
      dep_name,
      search_type_spec
    )?;

    writeln!(&self.cmakelists_file,
      "{}if( NOT {} )\n\t{}{}message( FATAL_ERROR \"{}\")\n{}endif()",
      indent,
      dep_info.found_varname(),
      indent,
      indent,
      // TODO: Make a better error message. Include links to relevant pages if possible.
      format!("Dependency '{}' was not found on the system. Please make sure the library is installed on the system.", dep_name),
      indent
    )?;

    if is_dep_internally_supported_by_emscripten {
      writeln!(&self.cmakelists_file, "endif()")?;
    }

    Ok(())
  }

  fn write_predefined_cmake_components_module_dep(
    &self,
    dep_name: &str,
    predep_graph: &Rc<RefCell<DependencyGraph>>,
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

    let needed_component_names: Vec<String> = self.sorted_target_info.targets_in_build_order
      .iter()
      .filter(|target|
        target.as_ref().borrow().container_project_id() == predep_graph.as_ref().borrow().project_id() 
      )
      .map(|target| target.as_ref().borrow().get_name().to_string())
      // Targets are iterated in build order, meaning targets are listed AFTER all their dependencies.
      // However, for compilers where link order matters (i.e. GCC), targets must be listed BEFORE their
      // dependencies. That's why this list is reversed.
      .rev()
      .collect();

    // TODO: I'm not sure if this is enforced. If it isn't, just don't write anythin for the unused library.
    assert!(
      !needed_component_names.is_empty(),
      "At least one component should be used from an imported compnent library"
    );

    for component_name in needed_component_names {
      write!(&self.cmakelists_file,
        "{} ",
        component_name
      )?;
    }

    writeln!(&self.cmakelists_file, ")\n")?;

    Ok(())
  }

  fn write_dep_clone_code(
    &self,
    dep_name: &str,
    uses_emscripten_link_flag: bool,
    git_revison: &GitRevisionSpecifier,
    repo_url: &str,
    is_auto_fetchcontent_ready: bool
  ) -> io::Result<()> {
    let git_revision_spec: String = match git_revison {
      GitRevisionSpecifier::Tag(tag_string) => {
        format!("\tGIT_TAG {}", tag_string)
      },
      GitRevisionSpecifier::CommitHash(hash_string) => {
        format!("\tGIT_TAG {}", hash_string)
      }
    };

    let hashed_cache_dep_dir: String = format!("{}/{}", dep_name, encoded_repo_url(repo_url));

    writeln!(&self.cmakelists_file,
      "if( NOT IS_DIRECTORY \"${{GCMAKE_DEP_CACHE_DIR}}/{}\" )",
      hashed_cache_dep_dir
    )?;
    writeln!(&self.cmakelists_file,
      "\tFetchContent_Declare(\n\t\tgcmake_cached_{}\n\t\tSOURCE_DIR \"${{GCMAKE_DEP_CACHE_DIR}}/{}\"\n\t\tGIT_REPOSITORY {}\n\t\t{}\n\t\tGIT_PROGRESS TRUE\n\t\tGIT_SHALLOW FALSE\n\t\tGIT_SUBMODULES_RECURSE TRUE\n\t)",
      dep_name,
      hashed_cache_dep_dir,
      repo_url,
      git_revision_spec
    )?;

    if uses_emscripten_link_flag {
      write!(&self.cmakelists_file,
        "\tif( NOT USING_EMSCRIPTEN )\n\t"
      )?;
    }

    writeln!(&self.cmakelists_file,
      "\tappend_to_uncached_dep_list( gcmake_cached_{} )",
      dep_name
    )?;

    if uses_emscripten_link_flag {
      writeln!(&self.cmakelists_file,
        "\tendif()"
      )?;
    }

    writeln!(&self.cmakelists_file, "endif()")?;
    self.write_newline()?;

    writeln!(&self.cmakelists_file,
      "FetchContent_Declare(\n\t{}\n\tSOURCE_DIR ${{CMAKE_CURRENT_SOURCE_DIR}}/dep/{}\n\tGIT_REPOSITORY \"${{GCMAKE_DEP_CACHE_DIR}}/{}\"\n\t{}\n\tGIT_PROGRESS TRUE\n\tGIT_SUBMODULES_RECURSE TRUE\n)",
      dep_name,
      dep_name,
      hashed_cache_dep_dir,
      git_revision_spec
    )?;

    if is_auto_fetchcontent_ready {
      if uses_emscripten_link_flag {
        write!(&self.cmakelists_file,
          "if( NOT USING_EMSCRIPTEN )\n\t"
        )?;
      }

      writeln!(&self.cmakelists_file,
        "append_to_actual_dep_list( {} )",
        dep_name
      )?;

      if uses_emscripten_link_flag {
        writeln!(&self.cmakelists_file,
          "endif()"
        )?;
      }
    }

    Ok(())
  }

  fn write_predefined_subdirectory_dependency(
    &self,
    dep_name: &str,
    dep_info: &PredefinedSubdirDep,
    is_auto_fetchcontent_ready: bool
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

    /* For now, install subdirectory dependencies by default unless the configuration file
        specifies otherwise.
      Subdirectory dependencies need to be installed in two scenarios:
        1. The dependency is built as a shared library, and a project output depends on that shared
            library.
        2. The library headers need to be installed because one of our project output libraries
            lists the dependency as a PUBLIC dependency (meaning the need for the dependency's
            headers (and library binaries, in some cases) are transitive.)

      #2 will be semi-solved once more libraries migrate to using CMake's FILE_SET HEADERS. However,
      FILE_SET was just added in CMake 3.23 (current version is 3.24.1 as of September 12th 2022), so
      it will be a while before that happens. Also, there are at least three or four different ways
      that different libraries use to implement header installation at the moment due to legacy
      CMake support (and the fact that several methods work well).

      #1 isn't something I can reliably solve at the moment either. There are so many ways to select
      either a static or shared version of a library in CMake, and so many libraries implement that
      differently. Sometimes a library target might hide the static/shared library behind and INTERFACE
      target, or more commonly an ALIAS target. Some are suffixed with '-shared' or '-static'. Some
      depend on the value of CMake's built-in BUILD_SHARED_LIBS and BUILD_STATIC_LIBS variables.

      For the above reasons, most libraries will be set to install by default. This means that a project
      installation should work out of the box even when it contains dependencies, because all dependencies
      will be included by default. However, this means that the default installation is likely to contain
      more headers (and possibly library files) than needed. For a minimal installation, just manually
      turn off any unneeded installation steps (if possible. Some libraries don't allow this.).
    */
    if let Some(installation_details) = dep_info.get_installation_details() {
      let mut default_value: bool = installation_details.should_install_by_default;

      if installation_details.is_inverse {
        default_value = !default_value;
      }

      self.set_basic_option(
        "",
        &installation_details.var_name,
        on_or_off_str(default_value),
        &format!("Whether to install {}. GCMake sets this to {} by default.", dep_name, on_or_off_str(default_value))
      )?;
    }

    self.write_dep_clone_code(
      dep_name,
      dep_info.uses_emscripten_link_flag(),
      dep_info.revision(),
      dep_info.repo_url(),
      is_auto_fetchcontent_ready
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
        // GCMake projects just link using their targets as usual, since Emscripten
        // doesn't explicitly specify support for projects we just made ourselves. Makes sense.
        false,
        dep_info.revision(),
        dep_info.repo_url(),
        true // All GCMake projects are FetchContent-ready.
      )?;
    }

    Ok(()) 
  }

  fn write_apply_dependencies(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "expose_uncached_deps()")?;

    if self.project_data.needs_fetchcontent() {
      writeln!(&self.cmakelists_file, "\nFetchContent_MakeAvailable( ${{ACTUAL_DEP_LIST}} )")?;
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

      if let Some(post_load) = combined_dep_info.post_load_script() {
        writeln!(&self.cmakelists_file, "{}", post_load.contents_ref())?;
      }
    }

    Ok(())
  }

  fn write_build_config_section(&self) -> io::Result<()> {
    self.write_newline()?;

    writeln!(&self.cmakelists_file,
      "initialize_build_config_vars()"
    )?;
    
    if self.project_data.has_global_defines() {
      writeln!(&self.cmakelists_file,
        "set( ALL_CONFIGS_LOCAL_DEFINES\n\t{}\n)",
        flattened_defines_list_string("\t", self.project_data.get_global_defines())
      )?;

      writeln!(&self.cmakelists_file,
        "propagate_all_configs_local_defines()"
      )?;
    }

    self.write_global_config_specific_defines()?;
    self.write_newline()?;
    self.write_build_configs()?;
    
    Ok(())
  }

  fn write_build_configs(&self) -> io::Result<()> {
    /*
      Compiler
        - <Build/Release...>
          - flags
          - defines
    */

    let mut simplified_map: HashMap<SpecificCompilerSpecifier, HashMap<&BuildType, &FinalBuildConfig>> = HashMap::new();

    for (build_type, build_config) in self.project_data.get_build_configs() {
      for (build_config_compiler, specific_config) in build_config {
        let converted_compiler_specifier: SpecificCompilerSpecifier = match *build_config_compiler {
          BuildConfigCompilerSpecifier::GCC => SpecificCompilerSpecifier::GCC,
          BuildConfigCompilerSpecifier::Clang => SpecificCompilerSpecifier::Clang,
          BuildConfigCompilerSpecifier::MSVC => SpecificCompilerSpecifier::MSVC,
          BuildConfigCompilerSpecifier::Emscripten => SpecificCompilerSpecifier::Emscripten,
          BuildConfigCompilerSpecifier::AllCompilers => continue
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
          let uppercase_config_name: String = config_name.name_str().to_uppercase();

          // Write compiler flags per compiler for each config.
          if build_config.has_compiler_flags() {
            writeln!(&self.cmakelists_file,
              "\tlist( APPEND {}_LOCAL_COMPILER_FLAGS\n\t\t{}\n\t)",
              &uppercase_config_name,
              &flattened_compiler_flags_string("\t\t", &build_config.compiler_flags)
            )?;

            if let SpecificCompilerSpecifier::Emscripten = compiler {
              // Same as in the write_flags_and_define_vars_for_output(...) function,
              // an optimal Emscripten build specifies compiler flags during the link step as well.
              writeln!(&self.cmakelists_file,
                "\tlist( APPEND {}_LOCAL_LINK_FLAGS\n\t\t{}\n\t)",
                &uppercase_config_name,
                &flattened_compiler_flags_string("\t\t", &build_config.compiler_flags)
              )?;
            }
          }

          if build_config.has_link_time_flags() {
            writeln!(&self.cmakelists_file,
              "\tlist( APPEND {}_LOCAL_LINK_FLAGS\n\t\t{}\n\t)",
              &uppercase_config_name,
              &flattened_compiler_flags_string("\t\t", &build_config.link_time_flags)
            )?;
          }

          // Write linker flags per "compiler" for each config
          if build_config.has_linker_flags() {
            writeln!(&self.cmakelists_file,
              "\tlist( APPEND {}_LOCAL_LINK_FLAGS\n\t\t{}\n\t)",
              uppercase_config_name,
              &flattened_linker_flags_string(&build_config.linker_flags)
            )?;
          }

          if build_config.has_compiler_defines() {
            writeln!(&self.cmakelists_file,
              "\tlist( APPEND {}_LOCAL_DEFINES ${{ALL_CONFIGS_LOCAL_DEFINES}}\n\t\t{}\n\t)",
              uppercase_config_name,
              &flattened_defines_list_string("\t\t", &build_config.defines)
            )?;
          }
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

  fn set_basic_option(
    &self,
    spacer: &str,
    var_name: &str,
    default_value: &str,
    description: &str
  ) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "{}option( {} \"{}\" {} )",
      spacer,
      var_name,
      description,
      default_value
    )?;
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
    let project_name: &str = self.project_data.get_project_base_name();

    let src_root_varname: String = format!("{}_SRC_ROOT", project_name);
    let include_root_varname: String = format!("{}_HEADER_ROOT", project_name);
    let template_impls_root_varname: String = format!("{}_TEMPLATE_IMPLS_ROOT", project_name);
    
    let project_include_dir_varname: String = format!("{}_INCLUDE_DIR", project_name);

    let src_var_name: String = format!("{}_SOURCES", project_name);
    let includes_var_name: String = format!("{}_HEADERS", project_name);
    let template_impls_var_name: String = format!("{}_TEMPLATE_IMPLS", project_name);

    self.write_newline()?;

    // Variables shared between all targets in the current project
    self.set_basic_var("", "PROJECT_INCLUDE_PREFIX", &format!("\"{}\"", self.project_data.get_full_include_prefix()))?;
    self.set_basic_var("", &src_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/{}/${{PROJECT_INCLUDE_PREFIX}}", SRC_DIR))?;
    self.set_basic_var("", &include_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/{}/${{PROJECT_INCLUDE_PREFIX}}", INCLUDE_DIR))?;
    self.set_basic_var("", &template_impls_root_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/{}/${{PROJECT_INCLUDE_PREFIX}}", TEMPLATE_IMPL_DIR))?;
    self.set_basic_var("", &project_include_dir_varname, &format!("${{CMAKE_CURRENT_SOURCE_DIR}}/{}", INCLUDE_DIR))?;

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

    // for output_target in self.sorted_target_info.targets_with_project_id(self.dep_graph_ref().project_id()) {
    for output_target in self.sorted_target_info.regular_targets_with_project_id(self.dep_graph_ref().project_id()) {
      // let output_name: String = borrowed_target.get_name().to_string();
      // let matching_output: &CompiledOutputItem = self.project_data.get_outputs().get(&output_name).unwrap();

      let output_name: String;
      let matching_output: &CompiledOutputItem;

      {
        let borrowed_target = output_target.as_ref().borrow();

        output_name = borrowed_target.get_name().to_string(); 
        matching_output = self.project_data.get_outputs().get(&output_name).unwrap();
      }

      let unwrapped_target = output_target.unwrap();

      match matching_output.get_output_type() {
        OutputItemType::Executable => {
          self.write_executable(
            matching_output,
            &unwrapped_target,
            &output_name,
            &src_var_name,
            &includes_var_name,
            &template_impls_var_name,
            &project_include_dir_varname
          )?;
        },
        OutputItemType::StaticLib
          | OutputItemType::SharedLib
          | OutputItemType::HeaderOnlyLib =>
        {
          self.write_defined_type_library(
            matching_output,
            &unwrapped_target,
            &output_name,
            &src_var_name,
            &includes_var_name,
            &template_impls_var_name,
            &project_include_dir_varname
          )?;
        },
        OutputItemType::CompiledLib => {
          self.write_toggle_type_library(
            matching_output,
            &unwrapped_target,
            &output_name,
            &src_var_name,
            &includes_var_name,
            &template_impls_var_name,
            &project_include_dir_varname
          )?;
        }
      }
    }

    Ok(())
  }

  // Append to defines, linker flags, and compiler flags on a per-target basis.
  fn append_to_target_build_config_options(
    &self,
    spacer: &str,
    output_name: &str,
    build_type: &BuildType,
    build_config: &FinalBuildConfig,
    compiler: Option<SpecificCompilerSpecifier>
  ) -> io::Result<()> {
    let build_type_name_upper: String = build_type.name_str().to_uppercase();

    if build_config.has_compiler_flags() {
      let flattened_flags_list: String = flattened_compiler_flags_string("\t", &build_config.compiler_flags);
      writeln!(&self.cmakelists_file,
        "{}list( APPEND {}_{}_OWN_COMPILER_FLAGS {} )",
        spacer,
        output_name,
        &build_type_name_upper,
        &flattened_flags_list
      )?;

      if let Some(SpecificCompilerSpecifier::Emscripten) = compiler {
        // According to this doc link:
        // https://emscripten.org/docs/compiling/Building-Projects.html#building-projects-with-optimizations
        // An optimal emscripten build needs to pass the same flags during both the compilation and link
        // phase.
        writeln!(&self.cmakelists_file,
          "{}list( APPEND {}_{}_OWN_LINK_FLAGS {} )",
          spacer,
          output_name,
          &build_type_name_upper,
          &flattened_flags_list
          // &flattened_linker_flags_string(&build_config.compiler_flags)
        )?;
      }
    }

    if build_config.has_link_time_flags() {
      writeln!(&self.cmakelists_file,
        "{}list( APPEND {}_{}_OWN_LINK_FLAGS {} )",
        spacer,
        output_name,
        &build_type_name_upper,
        flattened_linker_flags_string(&build_config.link_time_flags)
      )?;
    }

    if build_config.has_linker_flags() {
      writeln!(&self.cmakelists_file,
        "{}list( APPEND {}_{}_OWN_LINK_FLAGS {} )",
        spacer,
        output_name,
        &build_type_name_upper,
        flattened_linker_flags_string(&build_config.linker_flags)
      )?;
    }

    if build_config.has_compiler_defines() {
      writeln!(&self.cmakelists_file,
        "{}list( APPEND {}_{}_OWN_DEFINES\n{}\t{}\n)",
        spacer,
        output_name,
        &build_type_name_upper,
        spacer,
        flattened_defines_list_string(&format!("{}\t", spacer), &build_config.defines)
      )?;
    }

    Ok(())
  }

  fn write_flag_and_define_vars_for_output(
    &self,
    variable_base_name: &str,
    output_data: &CompiledOutputItem
  ) -> io::Result<()> {
    // This works because the toplevel project build config is passed down
    // to all the subprojects.
    // Need to set these here for all allowed configs.
    for (build_type, _) in self.project_data.get_build_configs() {
      let build_type_name_upper: String = build_type.name_str().to_uppercase();

      let inherited_linker_flags: String;
      let inherited_compiler_flags: String;

      if output_data.is_header_only_type() {
        inherited_linker_flags = String::from("");
        inherited_compiler_flags = String::from("");
      }
      else {
        inherited_linker_flags = format!("\"${{{}_LOCAL_LINK_FLAGS}}\"", &build_type_name_upper);
        inherited_compiler_flags = format!("\"${{{}_LOCAL_COMPILER_FLAGS}}\"", &build_type_name_upper);
      }

      self.set_basic_var(
        "",
        &format!("{}_{}_OWN_LINK_FLAGS", variable_base_name, &build_type_name_upper),
        &inherited_linker_flags
      )?;

      self.set_basic_var(
        "",
        &format!("{}_{}_OWN_COMPILER_FLAGS", variable_base_name, &build_type_name_upper),
        &inherited_compiler_flags
      )?;

      self.set_basic_var(
        "",
        &format!("{}_{}_OWN_DEFINES", variable_base_name, &build_type_name_upper),
        &format!("\"${{{}_LOCAL_DEFINES}}\"", &build_type_name_upper)
      )?;
    }

    if let Some(build_config_map) = output_data.get_build_config_map() {
      if !build_config_map.is_empty() {

        // All configs and all compilers
        if let Some(config_by_compiler) = build_config_map.get(&TargetSpecificBuildType::AllConfigs) {
          if let Some(always_applicable_config) = config_by_compiler.get(&BuildConfigCompilerSpecifier::AllCompilers) {
            for (build_type, _) in self.project_data.get_build_configs() {
              self.append_to_target_build_config_options(
                "",
                variable_base_name,
                build_type,
                always_applicable_config,
                None
              )?;
            }
          }
        }

        let mut any_compiler_config: HashMap<BuildType, &FinalBuildConfig> = HashMap::new();
        let mut by_compiler: HashMap<SpecificCompilerSpecifier, HashMap<TargetSpecificBuildType, &FinalBuildConfig>> = HashMap::new();

        for (build_type, config_by_compiler) in build_config_map {
          for (compiler_or_all, build_config) in config_by_compiler {
            match compiler_or_all {
              BuildConfigCompilerSpecifier::AllCompilers => {
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
            variable_base_name,
            build_type,
            config_for_any_compiler,
            None
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
                  variable_base_name,
                  build_type,
                  build_config,
                  Some(specific_compiler)
                )?;
              }
            }
            else {
              // Settings for a single build type, for a specific compiler
              self.append_to_target_build_config_options(
                "\t",
                variable_base_name,
                &given_build_type.to_general_build_type().unwrap(),
                build_config,
                Some(specific_compiler)
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
            variable_base_name,
            build_type,
            build_config,
            None
          )?;
        }
      }
    }

    Ok(())
  }

  fn write_defines_for_output(
    &self,
    variable_base_name: &str,
    output_data: &CompiledOutputItem,
    target_name: &str
  ) -> io::Result<()> {
    let inheritance_method: &str = match output_data.get_output_type() {
      OutputItemType::HeaderOnlyLib => "INTERFACE",
      // For executables, defines are set on the receiver lib.
      OutputItemType::Executable => "INTERFACE",
      // I'm not sure if compiled library defines should be public or private, but for now I'm
      // making them public because global defines might be referenced in header files. 
      // TODO: 
      // _compiled_lib_type => "PRIVATE"
      _compiled_lib_type => "PUBLIC"
    };

    // Compiler defines
    writeln!(&self.cmakelists_file,
      "target_compile_definitions( {}\n\t{} ",
      target_name,
      inheritance_method
    )?;

    for (config, _) in self.project_data.get_build_configs() {
      writeln!(&self.cmakelists_file,
        "\t\t\"$<$<CONFIG:{}>:${{{}_{}_OWN_DEFINES}}>\"",
        config.name_str(),
        variable_base_name,
        config.name_str().to_uppercase()
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
    variable_base_name: &str,
    output_data: &CompiledOutputItem,
    target_name: &str
  ) -> io::Result<()> {
    let inheritance_method: &str = match output_data.get_output_type() {
      OutputItemType::HeaderOnlyLib => "INTERFACE",
      _ => "PRIVATE"
    };

    // Linker flags
    writeln!(&self.cmakelists_file,
      "target_link_options( {}\n\t{} ",
      target_name,
      inheritance_method
    )?;

    for (config, _) in self.project_data.get_build_configs() {
      writeln!(&self.cmakelists_file,
        "\t\t\"$<$<CONFIG:{}>:${{{}_{}_OWN_LINK_FLAGS}}>\"",
        config.name_str(),
        variable_base_name,
        config.name_str().to_uppercase()
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
    variable_base_name: &str,
    output_data: &CompiledOutputItem,
    target_name: &str
  ) -> io::Result<()> {
    let inheritance_method: &str = match output_data.get_output_type() {
      OutputItemType::HeaderOnlyLib => "INTERFACE",
      _ => "PRIVATE"
    };

    // Compiler flags
    writeln!(&self.cmakelists_file,
      "target_compile_options( {}\n\t{} ",
      target_name,
      inheritance_method
    )?;

    for (config, _) in self.project_data.get_build_configs() {
      writeln!(&self.cmakelists_file,
        "\t\t\"$<$<CONFIG:{}>:${{{}_{}_OWN_COMPILER_FLAGS}}>\"",
        config.name_str(),
        variable_base_name,
        config.name_str().to_uppercase()
      )?;
    }

    writeln!(&self.cmakelists_file,
      ")"
    )?;
    self.write_newline()?;

    // Language standard and extensions config
    writeln!(&self.cmakelists_file,
      "target_compile_features( {}\n\t{} ",
      target_name,
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
    output_data: &CompiledOutputItem,
    output_target_node: &Rc<RefCell<TargetNode<'a>>>
  ) -> io::Result<()> {
    let borrowed_output_target_node = output_target_node.as_ref().borrow();

    if borrowed_output_target_node.has_links() {
      // The dependency graph already ensures there are no duplicate links. However, some libraries
      // in CMake are linked using a variable instead of targets (ex: ${wxWidgets_LIBRARIES}). That
      // variable is considered the "namespaced output target" for each target in the predefined
      // dependency. Therefore this set is used to ensure that variable is not written multiple times.
      let mut already_written: HashSet<String> = HashSet::new();

      let mut emscripten_link_flags_to_apply: HashMap<String, Vec<EmscriptenLinkFlagInfo>> = HashMap::new();

      writeln!(&self.cmakelists_file,
        "target_link_libraries( {} ",
        output_name
      )?;

      for (given_link_mode, dep_node_list) in self.sorted_target_info.regular_dependencies_by_mode(output_target_node) {
        assert!(
          !dep_node_list.is_empty(),
          "If a link category for a target's dependencies exists in the map, then the target should have at least one dependency in that category."
        );

        let inheritance_method: &str = match output_data.get_output_type() {
          // Every executable now has a "receiver INTERFACE library" which contains
          // the target's defines, code files (except for the entry file) and linked libraries.
          // In theory this should make testing much easier, since test executables can
          // just inherit all needed code, libraries, and defines for testing executables
          // by linking to the "receiver library".
          OutputItemType::Executable => "INTERFACE",
          _ => match given_link_mode {
            LinkMode::Public => "PUBLIC",
            LinkMode::Private => "PRIVATE",
            LinkMode::Interface => "INTERFACE",
          }
        };

        writeln!(&self.cmakelists_file,
          "\t{}",
          inheritance_method
        )?;

        // For compilers where link order matters, libraries must be listed before the libraries they depend on.
        for dependency_node in dep_node_list.iter().rev() {
          let borrowed_node: &TargetNode = &dependency_node.as_ref().borrow();
          
          let linkable_target_name: &str = match borrowed_node.simple_output_type() {
            SimpleNodeOutputType::Executable => borrowed_node.get_internal_receiver_name(),
            SimpleNodeOutputType::Library => borrowed_node.get_cmake_namespaced_target_name()
          };

          if !already_written.contains(linkable_target_name) {
            let matching_link: &Link = borrowed_output_target_node
              .get_link_by_id(borrowed_node.unique_target_id())
              .unwrap();

            let mut normal_link_constraint: SystemSpecifierWrapper = matching_link.get_system_spec_info().clone();

            if borrowed_node.is_internally_supported_by_emscripten() {
              normal_link_constraint = normal_link_constraint.intersection(
                &parse_leading_system_spec("((not emscripten))")
                  .unwrap()
                  .unwrap()
                  .value
              );
            }

            writeln!(&self.cmakelists_file,
              "\t\t{}",
              system_constraint_expression(
                &normal_link_constraint,
                linkable_target_name
              )
            )?;

            already_written.insert(String::from(linkable_target_name));
          }

          if let Some(mut emscripten_link_flag_info) = borrowed_node.emscripten_link_flag() {
            let emscripten_constraint: SystemSpecifierWrapper = parse_leading_system_spec("((emscripten))")
              .unwrap()
              .unwrap()
              .value;
            
            emscripten_link_flag_info.full_flag_expression = system_constraint_expression(
              &emscripten_constraint,
              &emscripten_link_flag_info.full_flag_expression
            );

            emscripten_link_flags_to_apply
              .entry(inheritance_method.to_string())
              .and_modify(|flag_list| {
                if !flag_list.contains(&emscripten_link_flag_info) {
                  flag_list.push(emscripten_link_flag_info.clone());
                }
              })
              .or_insert(vec![emscripten_link_flag_info]);
          }
        }
      }

      writeln!(&self.cmakelists_file,
        ")"
      )?;

      if !emscripten_link_flags_to_apply.is_empty() {
        let cmake_commands = [
          ("target_compile_options", false),
          ("target_link_options", true)
        ];

        for (flags_command, is_command_link_time) in cmake_commands {
          writeln!(&self.cmakelists_file,
            "{}( {}",
            flags_command,
            output_name
          )?;

          for (inheritance_method, emscripten_flag_expression_list) in &emscripten_link_flags_to_apply {
            writeln!(&self.cmakelists_file,
              "\t{}",
              inheritance_method
            )?;

            for EmscriptenLinkFlagInfo { full_flag_expression, supports_link_time_only } in emscripten_flag_expression_list {
              if !(*supports_link_time_only && !is_command_link_time) {
                writeln!(&self.cmakelists_file,
                  "\t\t\"{}\"",
                  full_flag_expression
                )?;
              }
            }
          }

          writeln!(&self.cmakelists_file, ")\n")?;
        }
      }
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

  fn write_depends_on_pre_build(&self, target_name: &str) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      // TODO: Move pre-build macro target name into its own variable (in this source code)
      "add_depends_on_pre_build( {} )",
      target_name
    )?;

    Ok(())
  }

  fn write_defined_type_library(
    &self,
    output_data: &CompiledOutputItem,
    output_target_node: &Rc<RefCell<TargetNode<'a>>>,
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
      "make_normal_lib( {} {} )",
      output_name,
      lib_type_string
    )?;

    if let OutputItemType::SharedLib = output_data.get_output_type() {
      writeln!(&self.cmakelists_file,
        "shared_lib_add_relative_install_rpath( {} )",
        output_name
      )?;
    }

    self.write_general_library_data(
      output_data,
      output_target_node,
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
    output_target_node: &Rc<RefCell<TargetNode<'a>>>,
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
      "DEFAULT"
    )?;

    self.write_general_library_data(
      output_data,
      output_target_node,
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
    output_target_node: &Rc<RefCell<TargetNode<'a>>>,
    output_name: &str,
    project_include_dir_varname: &str,
    includes_var_name: &str,
    template_impls_var_name: &str,
    src_var_name: &str
  ) -> io::Result<()> {
    let target_name: String;
    let alias_name: String;

    {
      let borrowed_node = output_target_node.as_ref().borrow();
      target_name = borrowed_node.get_cmake_target_base_name().to_string();
      alias_name = output_target_node.as_ref().borrow().get_cmake_namespaced_target_name().to_string();
    }

    let lib_spec_string: &str = if output_data.is_header_only_type()
      { "HEADER_ONLY_LIB" }
      else { "COMPILED_LIB" };

    writeln!(&self.cmakelists_file,
      "add_library( {} ALIAS {} )",
      alias_name,
      target_name
    )?;
    self.write_newline()?;

    writeln!(&self.cmakelists_file, "if( USING_EMSCRIPTEN )")?;
    writeln!(&self.cmakelists_file,
      "\tapply_emscripten_specifics( {} {} )",
      target_name,
      target_name
    )?;
    writeln!(&self.cmakelists_file, "endif()")?;

    if output_data.is_compiled_library_type() {
      writeln!(&self.cmakelists_file,
        "generate_and_install_export_header( {} )",
        target_name
      )?;
    }

    writeln!(&self.cmakelists_file,
      "add_to_target_installation_list( {} )",
      target_name
    )?;
    self.write_newline()?;

    writeln!(&self.cmakelists_file,
      "apply_lib_files( {} {} \"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\" \"${{{}}}\" \"${{{}}}\" \"${{{}}}\" )",
      target_name,
      lib_spec_string,
      output_data.get_entry_file().replace("./", ""),
      src_var_name,
      includes_var_name,
      template_impls_var_name
    )?;

    writeln!(&self.cmakelists_file,
      "apply_include_dirs( {} {} \"${{{}}}\" )",
      target_name,
      lib_spec_string,
      &project_include_dir_varname
    )?;
    self.write_newline()?;

    self.write_properties_for_output(
      &target_name,
      &HashMap::from([
        (String::from("RUNTIME_OUTPUT_DIRECTORY"), String::from(RUNTIME_BUILD_DIR_VAR)),
        (String::from("LIBRARY_OUTPUT_DIRECTORY"), String::from(LIB_BUILD_DIR_VAR)),
        (String::from("ARCHIVE_OUTPUT_DIRECTORY"), String::from(LIB_BUILD_DIR_VAR)),
        (String::from("C_EXTENSIONS"), String::from("OFF")),
        (String::from("CXX_EXTENSIONS"), String::from("OFF"))
      ])
    )?;
    self.write_newline()?;

    self.write_depends_on_pre_build(&target_name)?;

    self.write_flag_and_define_vars_for_output(output_name, output_data)?;
    self.write_defines_for_output(output_name, output_data, &target_name)?;
    self.write_target_link_options_for_output(output_name, output_data, &target_name)?;
    self.write_target_compile_options_for_output(output_name, output_data, &target_name)?;
    self.write_newline()?;

    self.write_links_for_output(&target_name, output_data, output_target_node)?;
    Ok(())
  }

  fn write_executable(
    &self,
    output_data: &CompiledOutputItem,
    output_target_node: &Rc<RefCell<TargetNode<'a>>>,
    output_name: &str,
    src_var_name: &str,
    includes_var_name: &str,
    template_impls_var_name: &str,
    project_include_dir_varname: &str
  ) -> io::Result<String> {
    let borrowed_node: &TargetNode = &output_target_node.as_ref().borrow();
    let target_name: &str = borrowed_node.get_cmake_target_base_name();
    let receiver_lib_name: &str = borrowed_node.get_internal_receiver_name();
    let is_pre_build_script: bool = borrowed_node.is_pre_build();

    self.write_output_title(&output_name)?;

    writeln!(&self.cmakelists_file,
      "add_library( {} INTERFACE )",
      receiver_lib_name
    )?;
    self.write_newline()?;

    if is_pre_build_script {
      writeln!(&self.cmakelists_file,
        "add_executable( {} ${{CMAKE_CURRENT_SOURCE_DIR}}/{} )",
        target_name,
        output_data.get_entry_file().replace("./", "")
      )?;
    }
    else {
      writeln!(&self.cmakelists_file,
        "add_executable( {} )",
        target_name
      )?;

      writeln!(&self.cmakelists_file,
        "add_executable( {} ALIAS {} )",
        borrowed_node.get_cmake_namespaced_target_name(),
        target_name
      )?;
    }
    self.write_newline()?;

    writeln!(&self.cmakelists_file, "if( USING_EMSCRIPTEN )")?;
    writeln!(&self.cmakelists_file,
      "\tapply_emscripten_specifics( {} {} )",
      receiver_lib_name,
      target_name
    )?;
    writeln!(&self.cmakelists_file, "endif()")?;

    if let Some(windows_icon_relative_path) = &output_data.windows_icon_relative_to_root_project {
      writeln!(&self.cmakelists_file,
        "generate_rc_file_for_windows_exe( {}\n\tICON_PATH \"${{TOPLEVEL_PROJECT_DIR}}/{}\"\n)",
        borrowed_node.get_cmake_namespaced_target_name(),
        windows_icon_relative_path.to_str().unwrap()
      )?;
    }

    if !is_pre_build_script {
      writeln!(&self.cmakelists_file,
        "exe_add_lib_relative_install_rpath( {} )",
        target_name
      )?;

      writeln!(&self.cmakelists_file,
        "add_to_target_installation_list( {} )",
        target_name
      )?;

      writeln!(&self.cmakelists_file,
        "apply_include_dirs( {} EXE_RECEIVER \"${{{}}}\" )",
        receiver_lib_name,
        &project_include_dir_varname
      )?;

      writeln!(&self.cmakelists_file,
        "apply_exe_files( {} {} \n\t\"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\"\n\t\"${{{}}}\"\n\t\"${{{}}}\"\n\t\"${{{}}}\"\n)",
        target_name,
        receiver_lib_name,
        output_data.get_entry_file().replace("./", ""),
        src_var_name,
        includes_var_name,
        template_impls_var_name
      )?;
      self.write_newline()?;
    }

    // TODO: Might need to write these for the receiver lib too. Not sure though.
    self.write_properties_for_output(
      target_name,
      &HashMap::from([
        (String::from("RUNTIME_OUTPUT_DIRECTORY"), String::from(RUNTIME_BUILD_DIR_VAR)),
        (String::from("C_EXTENSIONS"), String::from("OFF")),
        (String::from("CXX_EXTENSIONS"), String::from("OFF"))
      ])
    )?;
    self.write_newline()?;

    if !is_pre_build_script {
      self.write_depends_on_pre_build(receiver_lib_name)?;
      self.write_depends_on_pre_build(target_name)?;
    }

    self.write_flag_and_define_vars_for_output(output_name, output_data)?;
    self.write_defines_for_output(output_name, output_data, receiver_lib_name)?;
    self.write_target_link_options_for_output(output_name, output_data, target_name)?;
    self.write_target_compile_options_for_output(output_name, output_data, target_name)?;
    self.write_newline()?;

    self.write_links_for_output(receiver_lib_name, output_data, output_target_node)?;

    writeln!(&self.cmakelists_file,
      "target_link_libraries( {} PRIVATE {} )",
      target_name,
      receiver_lib_name
    )?;

    if self.project_data.is_test_project() {
      assert!(
        self.project_data.get_test_framework().is_some(),
        "A test framework is defined for a test project."
      );

      match self.project_data.get_test_framework().as_ref().unwrap() {
        FinalTestFramework::Catch2(_) => {
          writeln!(&self.cmakelists_file,
            "catch_discover_tests( {} )",
            target_name
          )?;
        },
        FinalTestFramework::DocTest(_) => {
          writeln!(&self.cmakelists_file,
            "doctest_discover_tests( {} )",
            target_name
          )?;
        },
        FinalTestFramework::GoogleTest(_) => {
          writeln!(&self.cmakelists_file,
            "gtest_discover_tests( {} )",
            target_name
          )?;
        }
      }
    }

    return Ok(target_name.to_string());
  }


  // See this page for help and a good example:
  // https://cmake.org/cmake/help/latest/guide/tutorial/Adding%20Export%20Configuration.html
  fn write_installation_and_exports(&self) -> io::Result<()> {
    if self.project_data.is_root_project() {
      let mut extra_targets_to_install: HashMap<String, String> = HashMap::new();

      for used_target in &self.sorted_target_info.targets_in_build_order {
        let borrowed_target: &TargetNode = &used_target.as_ref().borrow();
        let namespaced_target_name = borrowed_target.get_cmake_namespaced_target_name().to_string();
        let container_project_name = borrowed_target.container_project().as_ref().borrow().project_mangled_name().to_string();

        if borrowed_target.must_be_additionally_installed() {
          extra_targets_to_install.insert(
            namespaced_target_name,
            container_project_name
          );
        }
      }

      for (namespaced_target, container_lib) in extra_targets_to_install {
        writeln!(&self.cmakelists_file,
          "add_to_install_list( {} \"${{{}_RELATIVE_DEP_PATH}}\" )",
          namespaced_target,
          container_lib
        )?;
      }
    }

    writeln!(&self.cmakelists_file, "clean_target_list()")?;
    writeln!(&self.cmakelists_file, "clean_needed_bin_files_list()")?;
    writeln!(&self.cmakelists_file, "clean_install_list()")?;

    match &self.project_data.get_project_type() {
      FinalProjectType::Root => {
        // writeln!(&self.cmakelists_file, "if( \"${{CMAKE_SOURCE_DIR}}\" STREQUAL \"${{CMAKE_CURRENT_SOURCE_DIR}}\" )")?;
        // writeln!(&self.cmakelists_file, "\tconfigure_installation( LOCAL_PROJECT_COMPONENT_NAME )")?;
        // writeln!(&self.cmakelists_file, "else()")?;
        // writeln!(&self.cmakelists_file, "\traise_target_list()")?;
        // writeln!(&self.cmakelists_file, "\traise_needed_bin_files_list()")?;
        // writeln!(&self.cmakelists_file, "\traise_install_list()")?;
        // writeln!(&self.cmakelists_file, "endif()")?;

        writeln!(&self.cmakelists_file, "if( GCMAKE_INSTALL AND \"${{CMAKE_CURRENT_SOURCE_DIR}}\" STREQUAL \"${{TOPLEVEL_PROJECT_DIR}}\" )")?;
        writeln!(&self.cmakelists_file, "\tconfigure_installation( LOCAL_PROJECT_COMPONENT_NAME )")?;
        writeln!(&self.cmakelists_file, "endif()")?;
      },
      FinalProjectType::Subproject { } => {
        writeln!(&self.cmakelists_file, "raise_target_list()")?;
        writeln!(&self.cmakelists_file, "raise_needed_bin_files_list()")?;
        writeln!(&self.cmakelists_file, "raise_install_list()")?;
        writeln!(&self.cmakelists_file, "raise_generated_export_headers_list()")?;
      },
      FinalProjectType::Test { .. } => {
        // NOTE: I don't think anything needs to happen here since tests are never installed
        // and all dependencies are now specified in the root project.
      }
    }

    Ok(())
  }

  fn write_toplevel_cpack_config(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "if( GCMAKE_INSTALL AND \"${{CMAKE_SOURCE_DIR}}\" STREQUAL \"${{CMAKE_CURRENT_SOURCE_DIR}}\" )"
    )?;

    let joined_shortcut_map: String = self.project_data.get_installer_shortcuts_config()
      .iter()
      .map(|(exe_name, shortcut_config)| {
        format!("{};{}", exe_name, quote_escaped_string(&shortcut_config.shortcut_name))
      })
      .collect::<Vec<String>>()
      .join(";");

    writeln!(&self.cmakelists_file,
      "\tgcmake_configure_cpack(\n\t\tVENDOR \"{}\"\n\t\tPROJECT_COMPONENT ${{LOCAL_PROJECT_COMPONENT_NAME}}\n\t\tINSTALLER_TITLE \"{}\"\n\t\tINSTALLER_DESCRIPTION \"{}\"\n\t\tINSTALLER_EXE_PREFIX \"{}\"\n\t\tSHORTCUT_MAP \"{}\"\n\t)",
      self.project_data.get_vendor(),
      self.project_data.get_installer_title(),
      self.project_data.get_installer_description(),
      self.project_data.get_installer_name_prefix(),
      joined_shortcut_map
    )?;

    writeln!(&self.cmakelists_file, "endif()")?;
    Ok(())
  }

  // Is only run if the project has tests
  fn write_test_config_section(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "if( ${{LOCAL_TOPLEVEL_PROJECT_NAME}}_BUILD_TESTS )"
    )?;

    if self.project_data.is_root_project() {
      writeln!(&self.cmakelists_file,
        "\tinclude( CTest )\n\tif( NOT BUILD_TESTING )\n\t\tenable_testing()\n\tendif()"
      )?;

      if self.project_data.is_root_project() {
        assert!(
          self.project_data.get_test_framework().is_some(),
          "When tests are being written for a project, the toplevel project has specified a test framework."
        );

        let test_framework: &FinalTestFramework = self.project_data.get_test_framework().as_ref().unwrap();

        match test_framework {
          FinalTestFramework::Catch2(_) => {
            writeln!(&self.cmakelists_file,
              "\n\tinclude( Catch )"
            )?;
          },
          FinalTestFramework::DocTest(_) => {
            writeln!(&self.cmakelists_file,
              "\n\tinclude( \"${{TOPLEVEL_PROJECT_DIR}}/dep/{}/scripts/cmake/doctest.cmake\" )",
              test_framework.project_dependency_name()
            )?;
          },
          FinalTestFramework::GoogleTest(_) => {
            writeln!(&self.cmakelists_file,
              "\n\tinclude( GoogleTest )"
            )?;
          },
        }
      }
    }

    for (test_name, _) in self.project_data.get_test_projects() {
      writeln!(&self.cmakelists_file,
        "\tadd_subdirectory( \"${{CMAKE_CURRENT_SOURCE_DIR}}/tests/{}\" )",
        test_name
      )?;
    }

    writeln!(&self.cmakelists_file, "endif()")?;

    Ok(())
  }

  fn dep_graph_ref(&self) -> Ref<DependencyGraph<'a>> {
    return self.dep_graph.as_ref().borrow();
  }
}
