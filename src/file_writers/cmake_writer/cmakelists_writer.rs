use std::{collections::{HashSet, BTreeMap, BTreeSet }, fs::File, io::{self, Write, ErrorKind}, path::{PathBuf, Path}, rc::Rc, cell::{RefCell, Ref}, iter::FromIterator};

use crate::{project_info::{final_project_data::{FinalProjectData, CppFileGrammar}, path_manipulation::{relative_to_project_root}, final_dependencies::{GitRevisionSpecifier, PredefinedCMakeComponentsModuleDep, PredefinedSubdirDep, PredefinedCMakeModuleDep, FinalPredepInfo, GCMakeDependencyStatus, FinalPredefinedDependencyConfig, base64_encoded, PredefinedDepFunctionality, FinalDownloadMethod, FinalDebianPackagesConfig}, raw_data_in::{BuildType, BuildConfigCompilerSpecifier, SpecificCompilerSpecifier, OutputItemType, LanguageConfigMap, TargetSpecificBuildType, dependencies::internal_dep_config::{CMakeModuleType}, DefaultCompiledLibType}, FinalProjectType, CompiledOutputItem, LinkMode, FinalTestFramework, dependency_graph_mod::dependency_graph::{DependencyGraph, OrderedTargetInfo, ProjectWrapper, TargetNode, SimpleNodeOutputType, Link, EmscriptenLinkFlagInfo, ContainedItem}, SystemSpecifierWrapper, CompilerDefine, FinalBuildConfig, CompilerFlag, LinkerFlag, gcmake_constants::{SRC_DIR, INCLUDE_DIR}, platform_spec_parser::parse_leading_system_spec, CodeFileInfo, RetrievedCodeFileType, PreBuildScriptType, FinalDocGeneratorName}, file_writers::cmake_writer::cmake_writer_helpers::system_constraint_generator_expression};

use super::{cmake_utils_writer::{CMakeUtilFile, CMakeUtilWriter}, cmake_writer_helpers::system_contstraint_conditional_expression};
use colored::*;

const RUNTIME_BUILD_DIR_VAR: &'static str = "${MY_RUNTIME_OUTPUT_DIR}";
const LIB_BUILD_DIR_VAR: &'static str = "${MY_LIBRARY_OUTPUT_DIR}";

struct SingleUsageConditional<'a> {
  // public_conditional represents whether the library will be needed transitively. Therefore it
  // includes both Public and Interface links.
  public_constraint: Option<SystemSpecifierWrapper>,
  private_constraint: Option<SystemSpecifierWrapper>,
  target_rc: Rc<RefCell<TargetNode<'a>>>
}

struct CodeFileTransformOptions {
  should_retain_cpp2_paths: bool
}

impl Default for CodeFileTransformOptions {
  fn default() -> Self {
    Self {
      should_retain_cpp2_paths: false
    }
  }
}

struct UsageConditionalGroup<'a> {
  all_conditionals: Vec<SingleUsageConditional<'a>>
}

impl<'a> UsageConditionalGroup<'a> {
  pub fn was_used(&self) -> bool {
    return self.all_conditionals
      .iter()
      .any(|SingleUsageConditional { public_constraint, private_constraint, .. } |
        public_constraint.is_some() || private_constraint.is_some()
      );
  }

  pub fn full_conditional_string_or(&self, used_by_default: bool) -> String {
    return self.all_conditionals.iter()
      .map(|single_conditional| {
        let full_constraint: Option<SystemSpecifierWrapper> = union_maybe_specs(
          single_conditional.public_constraint.as_ref(),
          single_conditional.private_constraint.as_ref()
        );

        let constraint_string: String = match full_constraint {
          None => used_by_default.to_string().to_uppercase(),
          Some(conditional) => system_contstraint_conditional_expression(&conditional)
        };

        let borrowed_target = single_conditional.target_rc.as_ref().borrow();
        let is_test_target: bool = borrowed_target
          .container_project().as_ref().borrow()
          .project_wrapper()
          .clone()
          .unwrap_normal_project()
          .is_test_project();

        if borrowed_target.is_regular_node() && !is_test_target {
          format!(
            "(DEFINED TARGET_{}_INSTALL_MODE AND ({}))",
            borrowed_target.get_name(),
            constraint_string
          )
        }
        else {
          format!("({})", constraint_string)
        }
      })
      .collect::<Vec<String>>()
      .join(" OR ")
  }
}

struct NormalLinkInfo {
  is_installed_with_project: bool,
  linkable_name: String,
  unaliased_lib_var: String,
  link_constraint: SystemSpecifierWrapper
}

enum DownloadMethodInfo {
  GitMethod {
    repo_url: String,
    revision: GitRevisionSpecifier
  },
  UrlMethod {
    windows_url: String,
    unix_url: String
  }
}

enum FullCMakeDownloadMethodInfo {
  GitMethod {
    repo_url: String,
    revision_spec_str: String
  },
  UrlMethod {
    _windows_url: String,
    _unix_url: String
  }
}

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
    let cmake_util_path = Path::new(project_data.get_project_root_dir()).join("cmake");
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
      let escaped: String = system_constraint_generator_expression(
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
      let escaped: String = system_constraint_generator_expression(
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
      system_constraint_generator_expression(
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

  include_dir_var: String,

  src_root_var: String,
  header_root_var: String,
  generated_src_root_var: String,
  entry_file_root_var: String
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

    let cmakelists_file_name: String = format!("{}/CMakeLists.txt", project_data.get_project_root_dir());

    drop(borrowed_graph);

    let project_name: &str = project_data.get_project_base_name();

    Ok(Self {
      src_root_var: format!("{}_SRC_ROOT", project_name),
      include_dir_var: format!("{}_INCLUDE_DIR", project_name),
      header_root_var: format!("{}_HEADER_ROOT", project_name),
      entry_file_root_var: format!("{}_ENTRY_ROOT", project_name),
      // Make sure this is the same in gcmake-cppfront-utils.cmake::gcmake_transform_cppfront_files
      generated_src_root_var: format!("{}_GENERATED_SOURCE_ROOT", project_name),
      dep_graph,
      sorted_target_info: sorted_target_info,
      project_data,
      util_writer,
      cmakelists_file: File::create(cmakelists_file_name)?
    })
  }

  fn write_cmakelists(&mut self) -> io::Result<()> {
    self.write_project_header()?;

    self.include_utils()?;
    self.write_newline()?;

    if self.project_data.is_root_project() {
      self.write_section_header("Toplevel-project-only configuration")?;

      writeln!(&self.cmakelists_file, "gcmake_begin_config_file()")?;
      self.write_toplevel_tweaks()?;
      self.write_features()?;
    }

    if self.project_data.has_predefined_dependencies() {
      self.write_section_header("Predefined dependency config")?;
      self.write_predefined_dependencies()?;
    }

    if self.project_data.has_gcmake_dependencies() {
      self.write_section_header("GCMake dependency config")?;
      self.write_gcmake_dependencies()?;
    }

    if self.project_data.is_root_project() {
      self.write_apply_dependencies()?;

      // This is the location dependency libraries should be installed to.
      // On Windows, this is just lib/. On non-Windows systems, this is
      // lib/dependencies/${PROJECT_NAME}
      // Where PROJECT_NAME is the name of the topmost GCMake project.
      self.set_basic_var(
        "",
        "DEPENDENCY_INSTALL_LIBDIR",
        "\"${CMAKE_INSTALL_LIBDIR}\""
      )?;

      // Make sure the libdir is unmodified when building the actual project.
      writeln!(&self.cmakelists_file,
        "if( NOT TARGET_SYSTEM_IS_WINDOWS )\n\t{}\nendif()",
        "set( CMAKE_INSTALL_LIBDIR \"${ORIGINAL_CMAKE_INSTALL_LIBDIR}\" CACHE PATH \"Library installation dir\" FORCE )"
      )?;

      self.write_section_header("Language Configuration")?;
      self.write_language_config()?;

      self.write_section_header("Build Configurations")?;
      self.write_build_config_section()?;
    }

    self.write_root_vars()?;

    if self.project_data.has_tests() {
      self.write_section_header("Tests Configuration")?;
      self.write_test_config_section()?;
    }

    self.write_project_order_dependent_info()?;

    // Tests must be created after all project targets have been created.
    // This is because tests always depend on a project target, but never vice-versa.
    // tests also never depend on each other.
    if self.project_data.has_tests() {
      self.write_use_test_projects()?;
    }

    if !self.project_data.is_test_project() {
      self.write_section_header("Documentation Generation")?;
      self.write_documentation_generation()?;

      self.write_section_header("Installation and Export configuration")?;
      writeln!(&self.cmakelists_file, "gcmake_end_config_file()")?;
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
    self.write_init_pre_build_step()?;

    // Must be written after all dependencies are imported AND the pre-build script has
    // been configured, since the pre-build script might generate files needed here.
    if self.project_data.any_files_contain_cpp2_grammar() {
      self.write_section_header("Transform .cpp2 files with CppFront")?;
      self.write_cppfront_transform()?
    }

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
      let root_project_root_path: &str = root_project_info.get_project_root_dir();

      for some_project_graph in ordered_projects_in_this_tree {
        let borrowed_graph = some_project_graph.as_ref().borrow();
        let subproject_data: Rc<FinalProjectData> = borrowed_graph.project_wrapper().clone().unwrap_normal_project();

        if borrowed_graph.project_id() == self.dep_graph_ref().project_id() {
          self.write_pre_build_and_outputs()?;
        }
        else if !subproject_data.is_test_project() {
          writeln!( &self.cmakelists_file,
            "gcmake_configure_subproject(\n\t\"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\"\n)",
            relative_to_project_root(root_project_root_path, PathBuf::from(subproject_data.get_project_root_dir())).to_str().unwrap()
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
    self.set_basic_var("", "IN_GCMAKE_CONTEXT", "TRUE")?;
    writeln!(&self.cmakelists_file, "ensure_gcmake_config_dirs_exist()")?;

    let project_supports_emscripten: bool = self.project_data.supports_emscripten();
    
    writeln!(&self.cmakelists_file, "if( USING_EMSCRIPTEN )")?;
    writeln!(&self.cmakelists_file,
      "\tconfigure_emscripten_mode( WITH_HTML )"
    )?;

    if !self.project_data.supports_emscripten() {
      self.set_basic_option(
        "\t",
        "GCMAKE_OVERRIDE_EMSCRIPTEN_COMPILATION",
        "OFF",
        "When ON, force-allows Emscripten compilation for projects which don't obviously support copmilation with Emscripten."
      )?;

      writeln!(&self.cmakelists_file,
        "\terr_if_using_emscripten( GCMAKE_OVERRIDE_EMSCRIPTEN_COMPILATION )"
      )?;
    }

    writeln!(&self.cmakelists_file, "endif()")?;

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

    writeln!(&self.cmakelists_file,
      "if( USING_MINGW )\n\tinitialize_mingw_dll_install_options()\nendif()"
    )?;

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

    writeln!(&self.cmakelists_file,
      "list( APPEND CMAKE_MODULE_PATH \"${{TOPLEVEL_PROJECT_DIR}}/cmake/modules\" )"
    )?;

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

    // These CMake functions are defined in gcmake-general-utils.cmake.
    writeln!(&self.cmakelists_file, "\ninitialize_build_tests_var()")?;
    writeln!(&self.cmakelists_file, "\ngcmake_initialize_build_docs_var()")?;
    
    if let Some(doc_config) = self.project_data.get_documentation_config() {
      self.set_basic_option(
        "",
        "${PROJECT_NAME}_DOCUMENT_HEADERS_ONLY",
        on_or_off_str(doc_config.headers_only),
        "When ON, only header files are documented. When OFF, implementation like .c and .cpp will also be documented."
      )?;
    }

    let config_names: Vec<&'static str> = self.project_data.get_build_configs()
      .iter()
      .map(|(build_type, _)| build_type.name_str())
      .collect();

    self.set_basic_var(
      "",
      "ALL_VALID_BUILD_TYPES",
      &enum_iterator::all::<BuildType>()
        .map(|build_type| build_type.name_str())
        .collect::<Vec<&str>>()
        .join(" ")
    )?;

    self.set_basic_var(
      "",
      "PROJECT_VALID_BUILD_TYPES",
      &config_names.join(" ")
    )?;

    writeln!(&self.cmakelists_file,
      "if( ${{isMultiConfigGenerator}} )"
    )?;

    self.set_basic_var("", "CMAKE_CONFIGURATION_TYPES", "${PROJECT_VALID_BUILD_TYPES}")?;

    writeln!(&self.cmakelists_file,
      "else()"
    )?;

    writeln!(&self.cmakelists_file,
      "\tset_property( CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS ${{PROJECT_VALID_BUILD_TYPES}} )",
    )?;

    // We use ALL_VALID_BUILD_TYPES instead of PROJECT_VALID_BUILD_TYPES here so we don't mess with
    // a build when this project is being used as a subdirectory dependency.
    writeln!(&self.cmakelists_file,
      "\n\tif( NOT \"${{CMAKE_BUILD_TYPE}}\" IN_LIST ALL_VALID_BUILD_TYPES )\n\t\tset( CMAKE_BUILD_TYPE \"{}\" CACHE STRING \"${{LOCAL_CMAKE_BUILD_TYPE_DOC_STRING}}\" FORCE )\n\tendif()",
      self.project_data.get_default_build_config().name_str()
    )?;
    self.write_newline()?;

    self.write_message("\t", "Building configuration: ${CMAKE_BUILD_TYPE}")?;
    writeln!(&self.cmakelists_file, "endif()")?;
    self.write_newline()?;

    self.set_basic_var("", "MY_RUNTIME_OUTPUT_DIR", "\"${CMAKE_BINARY_DIR}/${CMAKE_INSTALL_BINDIR}/$<CONFIG>\"")?;
    self.set_basic_var("", "MY_LIBRARY_OUTPUT_DIR", "\"${CMAKE_BINARY_DIR}/${CMAKE_INSTALL_LIBDIR}/$<CONFIG>\"")?;
    self.write_newline()?;

    writeln!(&self.cmakelists_file,
      "if( CMAKE_SOURCE_DIR STREQUAL TOPLEVEL_PROJECT_DIR )",
    )?;

    // IN_GCMAKE_CONTEXT determines whethwer the toplevel project is a GCMake project.
    // This is necessary for deciding whether or not to build libraries
    self.set_basic_var(
      "\t",
      "IN_GCMAKE_CONTEXT",
      "TRUE"
    )?;

    writeln!(&self.cmakelists_file,
      "\tcmake_path( GET CMAKE_INSTALL_INCLUDEDIR STEM _the_includedir_stem )"
    )?;
    // Install headers to include/PROJECT_NAME so they don't collide with existing system headers.
    writeln!(&self.cmakelists_file,
      "\tif( NOT _the_includedir_stem STREQUAL PROJECT_NAME )\n{}\n\tendif()",
      "\t\tset( CMAKE_INSTALL_INCLUDEDIR \"${CMAKE_INSTALL_INCLUDEDIR}/${PROJECT_NAME}\" CACHE PATH \"Header file installation dir\" FORCE )"
    )?;

    writeln!(&self.cmakelists_file,
      "\tcmake_path( GET CMAKE_INSTALL_LIBDIR STEM _the_libdir_stem )"
    )?;
    self.set_basic_var(
      "\t",
      "ORIGINAL_CMAKE_INSTALL_LIBDIR",
      "\"${CMAKE_INSTALL_LIBDIR}\""
    )?;

    writeln!(&self.cmakelists_file,
      "\tif( NOT _the_libdir_stem STREQUAL PROJECT_NAME )"
    )?;
    // This is modified for the dependency loading step so that dependency libraries are installed to
    // a location which won't conflict with already installed libraries.
    writeln!(&self.cmakelists_file,
      "\t\tif( NOT TARGET_SYSTEM_IS_WINDOWS ){}\nendif()",
      "\n\t\t\tset( CMAKE_INSTALL_LIBDIR \"${CMAKE_INSTALL_LIBDIR}/dependencies/${PROJECT_NAME}\" CACHE PATH \"Library installation dir\" FORCE )"
    )?;
    writeln!(&self.cmakelists_file,
      "\tendif()"
    )?;

    writeln!(&self.cmakelists_file, "endif()")?;

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
    self.set_basic_var("\t", "CMAKE_PDB_OUTPUT_DIRECTORY", RUNTIME_BUILD_DIR_VAR)?;
    self.set_basic_var("\t", "CMAKE_COMPILE_PDB_OUTPUT_DIRECTORY", LIB_BUILD_DIR_VAR)?;
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
      "initialize_pgo_defaults()\ninitialize_install_mode()"
    )?;

    let reverse_project_targets = self.sorted_target_info
      .regular_targets_with_root_project_id(self.dep_graph_ref().project_id())
      .into_iter()
      .rev()
      .filter(|target|
        // We never install test projects, so they should be filtered out.
        !target.as_ref().borrow()
          .container_project().as_ref().borrow()
          .project_wrapper().clone().unwrap_normal_project().is_test_project()
      );
    
    for output_node in reverse_project_targets {
      let borrowed_target = output_node.as_ref().borrow();

      let output_type_dependent_conditional: &str;

      // When the project is top level, targets whose type matches the install mode should
      // be fully installed.
      if borrowed_target.maybe_regular_output().unwrap().is_executable_type() {
        output_type_dependent_conditional = "GCMAKE_INSTALL_MODE STREQUAL \"NORMAL\" OR GCMAKE_INSTALL_MODE STREQUAL \"EXE_ONLY\" ";
        writeln!(&self.cmakelists_file,
          "if( TOPLEVEL_PROJECT_DIR STREQUAL CMAKE_SOURCE_DIR AND ( {} ) )",
          output_type_dependent_conditional
        )?;
      }
      else {
        output_type_dependent_conditional = "GCMAKE_INSTALL_MODE STREQUAL \"NORMAL\" OR GCMAKE_INSTALL_MODE STREQUAL \"LIB_ONLY\"";
        writeln!(&self.cmakelists_file,
          // Libraries should all be built by default if this project is being used by a
          // non-GCMake project.
          "if( (NOT IN_GCMAKE_CONTEXT) OR (TOPLEVEL_PROJECT_DIR STREQUAL CMAKE_SOURCE_DIR AND ( {} )) )",
          output_type_dependent_conditional
        )?;
      }

      writeln!(&self.cmakelists_file,
        "\tmark_gcmake_target_usage( {} FULL )",
        borrowed_target.get_name()
      )?;
      writeln!(&self.cmakelists_file, "endif()")?;

      // If the output should be installed, ensure its dependencies are installed the proper way too.
      writeln!(&self.cmakelists_file,
        "if( ({}) AND (( NOT CMAKE_SOURCE_DIR STREQUAL TOPLEVEL_PROJECT_DIR AND DEFINED TARGET_{}_INSTALL_MODE ) OR (CMAKE_SOURCE_DIR STREQUAL TOPLEVEL_PROJECT_DIR AND ({})) ) )",
        system_contstraint_conditional_expression(borrowed_target.get_system_spec_info()),
        borrowed_target.get_name(),
        output_type_dependent_conditional,
      )?;

      for (link_mode, dep_list) in self.sorted_target_info.regular_dependencies_by_mode(&output_node.0) {
        for dependency_node in dep_list {
          let borrowed_dependency = dependency_node.as_ref().borrow();
          let usage_mode: &str = match &link_mode {
            LinkMode::Public | LinkMode::Interface => "FULL",
            LinkMode::Private => "MINIMAL"
          };

          match dependency_node.as_ref().borrow().get_contained_item() {
            ContainedItem::PreBuild(_) => unreachable!("Pre-build scripts are filtered out when iterating over \"regular\" dependencies."),
            ContainedItem::CompiledOutput(_) => {
              writeln!(&self.cmakelists_file,
                "\tmark_gcmake_target_usage( {} {} )",
                borrowed_dependency.get_name(),
                usage_mode
              )?;
            },
            ContainedItem::PredefinedLibrary { .. } => {
              let container_project_name: String = borrowed_dependency
                .container_project().as_ref().borrow()
                .root_project().as_ref().borrow()
                .project_identifier_name().to_string();

              writeln!(&self.cmakelists_file,
                "\tmark_gcmake_project_usage( {} {} )",
                // We're using the predefined dependency name this time instead of per-target name
                container_project_name,
                usage_mode
              )?;
            }
          }
        }
      }

      writeln!(&self.cmakelists_file,
        "endif()",
      )?;
    }

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
      writeln!(&self.cmakelists_file, "include(GNUInstallDirs)")?;
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

    writeln!(&self.cmakelists_file, "initialize_deb_list()")?;
    writeln!(&self.cmakelists_file, "initialize_minimal_installs()")?;
    writeln!(&self.cmakelists_file, "initialize_target_list()")?;
    writeln!(&self.cmakelists_file, "initialize_needed_bin_files_list()")?;
    writeln!(&self.cmakelists_file, "initialize_additional_dependency_install_list()")?;
    writeln!(&self.cmakelists_file, "initialize_generated_export_headers_list()")?;
    writeln!(&self.cmakelists_file, "gcmake_init_documentable_files_list()")?;
    
    if self.project_data.is_root_project() {
      writeln!(&self.cmakelists_file, "initialize_uncached_dep_list()")?;
      writeln!(&self.cmakelists_file, "initialize_actual_dep_list()")?;
      writeln!(&self.cmakelists_file, "initialize_custom_find_modules_list()")?;
    }

    Ok(())
  }

  fn write_init_pre_build_step(&self) -> io::Result<()> {
    writeln!(
      &self.cmakelists_file,
      "initialize_prebuild_step( \"{}\" )\n",
      self.project_data.prebuild_script_name()
    )?;
    
    if let Some(prebuild_script) = self.project_data.get_prebuild_script() {
      self.set_code_file_collection(
        "pre_build_generated_sources",
        &self.project_data.get_src_dir_relative_to_project_root(),
        &self.src_root_var,
        &self.generated_src_root_var,
        &prebuild_script.generated_sources(),
        &CodeFileTransformOptions {
          should_retain_cpp2_paths: true
        }
      )?;

      self.set_code_file_collection(
        "pre_build_generated_headers",
        &self.project_data.get_include_dir_relative_to_project_root(),
        &self.header_root_var,
        &self.generated_src_root_var,
        &prebuild_script.generated_headers(),
        &CodeFileTransformOptions {
          should_retain_cpp2_paths: true
        }
      )?;

      self.set_code_file_collection(
        "pre_build_generated_template_impls",
        &self.project_data.get_include_dir_relative_to_project_root(),
        &self.header_root_var,
        &self.generated_src_root_var,
        &prebuild_script.generated_template_impls(),
        &CodeFileTransformOptions {
          should_retain_cpp2_paths: true
        }
      )?;

      self.set_basic_var(
        "",
        "pre_build_generated_files_list",
        "${pre_build_generated_sources} ${pre_build_generated_headers} ${pre_build_generated_template_impls}"
      )?;

      match prebuild_script.get_type() {
        PreBuildScriptType::Exe(exe_info) => {
          assert!(
            self.dep_graph_ref().get_pre_build_node().is_some(),
            "When a FinalProjectData contains a pre-build script, the matching dependency graph for the project must contain a pre-build script node."
          );
          
          let entry_file: &CodeFileInfo = exe_info.get_entry_file();

          if entry_file.uses_cpp2_grammar() {
            self.set_code_file_collection(
              "pre_build_entry_dummy_list",
              "./",
              &self.entry_file_root_var,
              &self.generated_src_root_var,
              &BTreeSet::from_iter([entry_file]),
              &CodeFileTransformOptions {
                should_retain_cpp2_paths: true
              }
            )?;

            writeln!(&self.cmakelists_file,
              "gcmake_transform_cppfront_files( pre_build_entry_dummy_list )"
            )?;
          }

          let script_target_name: String = self.write_executable(
            exe_info,
            self.dep_graph_ref().get_pre_build_node().as_ref().unwrap(),
            &self.project_data.prebuild_script_name(),
            "UNUSED",
            "UNUSED",
            "UNUSED"
          )?;

          writeln!(&self.cmakelists_file,
            "use_executable_prebuild_script( {} pre_build_generated_files_list )",
            script_target_name
          )?;
        },
        PreBuildScriptType::Python(python_script_path) => {
          writeln!(&self.cmakelists_file,
            "use_python_prebuild_script( \"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\" pre_build_generated_files_list )",
            python_script_path.to_str().unwrap()
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
    for wrapped_graph in &self.sorted_target_info.project_order {
      let borrowed_graph = wrapped_graph.as_ref().borrow();

      if borrowed_graph.project_wrapper().contains_predef_dep() {
        if let Some((dep_name, predep_graph)) = self.dep_graph_ref().get_predefined_dependencies().get_key_value(borrowed_graph.project_identifier_name()) {
          let dep_info: Rc<FinalPredefinedDependencyConfig> = predep_graph.as_ref().borrow().project_wrapper().clone().unwrap_predef_dep();
          let usage_conditional: UsageConditionalGroup = self.get_usage_conditional_for_dependency(&wrapped_graph.0);

          if !usage_conditional.was_used() {
            println!(
              "{} loading project [{}]: No targets from predefined dependency '{}' are ever actually linked to an output.",
              "Warning".yellow(),
              self.project_data.get_name_for_error_messages(),
              borrowed_graph.project_debug_name()
            );
          }

          // Usage conditional
          writeln!(&self.cmakelists_file,
            "if( {} )",
            usage_conditional.full_conditional_string_or(true)
          )?;

          if let Some(pre_load) = dep_info.pre_load_script() {
            writeln!(&self.cmakelists_file, "{}", pre_load.contents_ref())?;
          }

          if let Some(custom_find_module) = dep_info.custom_find_module_file() {
            assert!(
              self.util_writer.is_some(),
              "Utility writer must exist for project \"{}\" because it is a root project.",
              self.project_data.get_name_for_error_messages()
            );

            self.util_writer.as_ref().unwrap().copy_custom_find_file(&custom_find_module.file_path)?;

            writeln!(&self.cmakelists_file,
              "add_to_custom_find_modules_list( {} )",
              dep_name
            )?;
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
                &wrapped_graph.0,
                dep_info.is_auto_fetchcontent_ready()
              )?;
            }
          }

          if dep_info.as_common().debian_packages_config().has_packages() {
            let FinalDebianPackagesConfig {
              runtime: runtime_packages,
              dev: dev_packages
            } = dep_info.as_common().debian_packages_config();

            writeln!(&self.cmakelists_file,
              "\tif( DEFINED PROJECT_{}_INSTALL_MODE )",
              dep_name
            )?;

            for runtime_package_name in runtime_packages {
              writeln!(&self.cmakelists_file,
                "\t\tadd_to_deb_list( \"{}\" )",
                runtime_package_name
              )?;
            }

            if !dev_packages.is_empty() {
              writeln!(&self.cmakelists_file,
                "\t\tif( \"${{PROJECT_{}_INSTALL_MODE}}\" STREQUAL \"FULL\" )",
                dep_name
              )?;

              for dev_package_name in dev_packages {
                writeln!(&self.cmakelists_file,
                  "add_to_deb_list( \"{}\" )",
                  dev_package_name
                )?;
              }

              writeln!(&self.cmakelists_file,
                "\t\tendif()",
              )?;
            }

            writeln!(&self.cmakelists_file, "\tendif()")?;
          }


          // End usage conditional
          writeln!(&self.cmakelists_file,
            "endif()"
          )?;
        }
      }
    }

    Ok(())
  }

  fn write_cppfront_transform(&self) -> io::Result<()> {
    let all_cpp2_source_list: BTreeSet<&CodeFileInfo> = self.project_data.all_sources_by_grammar(CppFileGrammar::Cpp2, false)
      .into_iter()
      .collect();

    self.set_code_file_collection(
      "all_cpp2_files",
      "./",
      &self.entry_file_root_var,
      &self.generated_src_root_var,
      &all_cpp2_source_list,
      &CodeFileTransformOptions {
        should_retain_cpp2_paths: true
      }
    )?;

    writeln!(&self.cmakelists_file,
      "gcmake_transform_cppfront_files( all_cpp2_files )"
    )?;

    Ok(())
  }

  fn write_predefined_cmake_module_dep(
    &self,
    dep_name: &str,
    _predep_graph: &Rc<RefCell<DependencyGraph>>,
    dep_info: &PredefinedCMakeModuleDep
  ) -> io::Result<()> {
    let search_type_spec: &str = match dep_info.module_type() {
      CMakeModuleType::BuiltinFindModule => "MODULE",
      CMakeModuleType::CustomFindModule => "MODULE",
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
      "{}find_package( {} {} )",
      indent,
      dep_info.find_module_base_name(),
      search_type_spec
    )?;

    writeln!(&self.cmakelists_file,
      "{}if( NOT {} )\n\t{}{}message( FATAL_ERROR \"{}\")\n{}endif()",
      indent,
      dep_info.found_varname(),
      indent,
      indent,
      format!(
        "Dependency '{}' was not found on the system. See {} for installation instructions and common issues.",
        dep_name,
        dep_info.get_gcmake_readme_link()
      ),
      indent
    )?;

    writeln!(&self.cmakelists_file,
      "{}if( \"${{PROJECT_{}_INSTALL_MODE}}\" STREQUAL \"FULL\" )",
      indent,
      dep_name
    )?;
    writeln!(&self.cmakelists_file,
      "{}\tgcmake_config_file_add_contents( \"find_dependency( {} {} )\" )",
      indent,
      dep_info.find_module_base_name(),
      search_type_spec
    )?;
    writeln!(&self.cmakelists_file,
      "{}endif()",
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
      CMakeModuleType::BuiltinFindModule => "MODULE",
      CMakeModuleType::CustomFindModule => "MODULE",
      CMakeModuleType::ConfigFile => "CONFIG"
    };

    write!(&self.cmakelists_file,
      "find_package( {} {} COMPONENTS ",
      dep_info.find_module_base_name(),
      search_type_spec
    )?;

    let needed_component_names: Vec<String> = self.sorted_target_info.targets_in_build_order
      .iter()
      .filter(|target|
        target.as_ref().borrow().container_project_id() == predep_graph.as_ref().borrow().project_id() 
      )
      .map(|target| target.as_ref().borrow().get_cmake_target_base_name().to_string())
      // Targets are iterated in build order, meaning targets are listed AFTER all their dependencies.
      // However, for compilers where link order matters (i.e. GCC), targets must be listed BEFORE their
      // dependencies. That's why this list is reversed.
      .rev()
      .collect();

    // TODO: I'm not sure if this is enforced. If it isn't, just don't write anything for the unused library.
    assert!(
      !needed_component_names.is_empty(),
      "At least one component should be used from an imported component library"
    );

    for component_name in needed_component_names {
      write!(&self.cmakelists_file,
        "{} ",
        component_name
      )?;
    }

    writeln!(&self.cmakelists_file, ")\n")?;

    writeln!(&self.cmakelists_file,
      "if( NOT {} )\n\tmessage( FATAL_ERROR \"{}\")\nendif()",
      dep_info.found_varname(),
      format!(
        "Dependency '{}' was not found on the system. See {} installation instructions and common issues.",
        dep_name,
        dep_info.get_gcmake_readme_link()
      )
    )?;

    writeln!(&self.cmakelists_file,
      "if( \"${{PROJECT_{}_INSTALL_MODE}}\" STREQUAL \"FULL\" )",
      dep_name
    )?;

    let used_targets_in_this_dep: String = self.sorted_target_info.targets_in_link_order()
      .filter(|target|
        target.as_ref().borrow().container_project_id() == predep_graph.as_ref().borrow().root_project_id()
      )
      .map(|target| target.as_ref().borrow().get_cmake_target_base_name().to_string())
      .collect::<Vec<String>>()
      .join(" ");

    writeln!(&self.cmakelists_file,
      "\tgcmake_config_file_add_contents( \"find_dependency( {} {} COMPONENTS {} )\" )",
      dep_info.find_module_base_name(),
      search_type_spec,
      // For now, assume a components module will be installed with all components. Later, it would be nice
      // to only list the components which are transitively needed (i.e. PUBLIC or INTERFACE linked to an
      // installed output), but for now this is fine.
      used_targets_in_this_dep
    )?;

    writeln!(&self.cmakelists_file, "endif()")?;

    Ok(())
  }

  fn write_dep_clone_code(
    &self,
    dep_name: &str,
    is_internally_supported_by_emscripten: bool,
    download_method: DownloadMethodInfo,
    is_auto_fetchcontent_ready: bool
  ) -> io::Result<()> {
    let download_method_name: &str = match &download_method {
      DownloadMethodInfo::GitMethod { .. } => "git",
      DownloadMethodInfo::UrlMethod { .. } => "url",
    };

    let cached_dep_name: String = format!("_gcmake_cached_{}_{}_mode", dep_name, download_method_name);
    let download_url_var: String = format!("{}_DOWNLOAD_URL", dep_name);

    let hashed_cache_dep_dir: String = match &download_method {
      DownloadMethodInfo::GitMethod { repo_url, .. } => {
        format!("git_repo/{}", base64_encoded(repo_url))
      },
      DownloadMethodInfo::UrlMethod { windows_url, unix_url } => {
        let cached_location_var: String = format!("{}_CACHE_DESTINATION_DIR", dep_name);

        writeln!(&self.cmakelists_file,
          "if( CURRENT_SYSTEM_IS_WINDOWS )"
        )?;

        self.set_basic_var(
          "\t",
          &download_url_var,
          &format!("\"{}\"", windows_url)
        )?;

        self.set_basic_var(
          "\t",
          &cached_location_var,
          &format!("\"archive_file/{}\"", base64_encoded(windows_url))
        )?;

        writeln!(&self.cmakelists_file, "else()")?;

        self.set_basic_var(
          "\t",
          &download_url_var,
          &format!("\"{}\"", unix_url)
        )?;

        self.set_basic_var(
          "\t",
          &cached_location_var,
          &format!("\"archive_file/{}\"", base64_encoded(unix_url))
        )?;

        writeln!(&self.cmakelists_file, "endif()")?;

        format!("${{{}}}", cached_location_var)
      }
    };

    let destination_cache_dir: String = format!("${{GCMAKE_DEP_CACHE_DIR}}/{}/{}", dep_name, hashed_cache_dep_dir);
    let temp_url_download_dir: String = format!("${{GCMAKE_DEP_CACHE_DIR}}/{}/temp_url_download", dep_name);

    let full_download_info: FullCMakeDownloadMethodInfo = match download_method {
      DownloadMethodInfo::GitMethod { repo_url, revision } => FullCMakeDownloadMethodInfo::GitMethod {
        repo_url,
        revision_spec_str: match revision {
          GitRevisionSpecifier::CommitHash(commit_hash) => format!("GIT_TAG \"{}\"", commit_hash),
          GitRevisionSpecifier::Tag(tag) => format!("GIT_TAG \"{}\"", tag)
        }
      },
      DownloadMethodInfo::UrlMethod { windows_url, unix_url } => FullCMakeDownloadMethodInfo::UrlMethod {
        _windows_url: windows_url,
        _unix_url: unix_url
      }
    };

    writeln!(&self.cmakelists_file,
      "if( NOT IS_DIRECTORY \"{}\" )",
      destination_cache_dir
    )?;

    match &full_download_info {
      FullCMakeDownloadMethodInfo::GitMethod { repo_url, revision_spec_str, .. } => {
        writeln!(&self.cmakelists_file,
          "\tFetchContent_Declare(\n\t\t{}\n\t\tSOURCE_DIR \"{}\"\n\t\tGIT_REPOSITORY \"{}\"\n\t\t{}\n\t\tGIT_PROGRESS TRUE\n\t\tGIT_SHALLOW FALSE\n\t\tGIT_SUBMODULES_RECURSE TRUE\n\t)",
          cached_dep_name,
          destination_cache_dir,
          repo_url,
          revision_spec_str
        )?;
      },
      FullCMakeDownloadMethodInfo::UrlMethod { .. } => {
        writeln!(&self.cmakelists_file,
          "\tFetchContent_Declare(\n\t\t{}\n\t\tSOURCE_DIR \"{}\"\n\t\tDOWNLOAD_DIR \"{}\"\n\t\tURL \"${{{}}}\"\n\t)",
          cached_dep_name,
          destination_cache_dir,
          temp_url_download_dir,
          download_url_var
        )?;
      }
    }

    if is_internally_supported_by_emscripten {
      write!(&self.cmakelists_file,
        "\tif( NOT USING_EMSCRIPTEN )\n\t"
      )?;
    }

    writeln!(&self.cmakelists_file,
      "\tappend_to_uncached_dep_list( {} )",
      cached_dep_name
    )?;

    if is_internally_supported_by_emscripten {
      writeln!(&self.cmakelists_file,
        "\tendif()"
      )?;
    }

    writeln!(&self.cmakelists_file, "endif()")?;
    self.write_newline()?;

    match &full_download_info {
      FullCMakeDownloadMethodInfo::GitMethod { revision_spec_str, .. } => {
        writeln!(&self.cmakelists_file,
          "FetchContent_Declare(\n\t{}\n\tSOURCE_DIR \"${{CMAKE_CURRENT_SOURCE_DIR}}/dep/{}\"\n\tGIT_REPOSITORY \"{}\"\n\t{}\n\tGIT_PROGRESS TRUE\n\tGIT_SUBMODULES_RECURSE TRUE\n)",
          dep_name,
          dep_name,
          destination_cache_dir,
          revision_spec_str
        )?;
      },
      FullCMakeDownloadMethodInfo::UrlMethod { .. } => {
        writeln!(&self.cmakelists_file,
          "cmake_path( GET {} FILENAME the_archive_file )",
          download_url_var
        )?;

        writeln!(&self.cmakelists_file,
          "FetchContent_Declare(\n\t{}\n\tSOURCE_DIR \"${{CMAKE_CURRENT_SOURCE_DIR}}/dep/{}\"\n\tURL \"{}/${{the_archive_file}}\"\n)",
          dep_name,
          dep_name,
          temp_url_download_dir
        )?;
      }
    }

    if is_auto_fetchcontent_ready {
      if is_internally_supported_by_emscripten {
        write!(&self.cmakelists_file,
          "if( NOT USING_EMSCRIPTEN )\n\t"
        )?;
      }

      writeln!(&self.cmakelists_file,
        "append_to_actual_dep_list( {} )",
        dep_name
      )?;

      if is_internally_supported_by_emscripten {
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
    graph_for_dependency: &Rc<RefCell<DependencyGraph<'a>>>,
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
      if !installation_details.should_install_by_default {
        let default_value: bool = installation_details.actual_value_for(installation_details.should_install_by_default);

        self.set_basic_option(
          "",
          &installation_details.var_name,
          on_or_off_str(default_value),
          &format!("Whether to install {}. GCMake sets this to {} by default.", dep_name, on_or_off_str(default_value))
        )?;
      }
      else {
        // Here, the library should be installed by default. That means for libraries with an installation variable
        // We can choose whether or not do do a full installation, or install on a per-library basis.
        let usage_conditional: UsageConditionalGroup = self.get_usage_conditional_for_dependency(graph_for_dependency);
        let var_default_name: String = format!("{}_DEFAULT_VALUE", &installation_details.var_name);

        if !usage_conditional.was_used() {
          self.set_basic_var(
            "",
            &installation_details.var_name,
            on_or_off_str(installation_details.actual_value_for(false))
          )?;
        }
        else {
          writeln!(&self.cmakelists_file,
            "if( \"${{PROJECT_{}_INSTALL_MODE}}\" STREQUAL \"FULL\" )",
            dep_name
          )?;

          self.set_basic_var(
            "\t",
            &var_default_name,
            on_or_off_str(installation_details.actual_value_for(true))
          )?;

          writeln!(&self.cmakelists_file,
            "elseif( \"${{PROJECT_{}_INSTALL_MODE}}\" STREQUAL \"MINIMAL\" )",
            dep_name
          )?;

          self.set_basic_var(
            "\t",
            &var_default_name,
            on_or_off_str(installation_details.actual_value_for(false))
          )?;

          // We can write this endif because at least one of the above conditionals is guaranteed to be written.
          writeln!(&self.cmakelists_file, "endif()")?;
        }

        self.set_basic_option(
          "",
          &installation_details.var_name,
          &format!("${{{}}}", &var_default_name),
          &format!("Whether to install {}. GCMake sets this to ${{{}}} by default.", dep_name, var_default_name)
        )?;
      }
    }

    let download_method: DownloadMethodInfo = match dep_info.download_method() {
      FinalDownloadMethod::GitMode(git_info) => DownloadMethodInfo::GitMethod {
        repo_url: git_info.repo_url.clone(),
        revision: git_info.revision_specifier.clone()
      },
      FinalDownloadMethod::UrlMode(url_info) => DownloadMethodInfo::UrlMethod {
        windows_url: url_info.windows_url(),
        unix_url: url_info.unix_url()
      }
    };

    self.write_dep_clone_code(
      dep_name,
      dep_info.is_internally_supported_by_emscripten(),
      download_method,
      is_auto_fetchcontent_ready
    )?;
    Ok(())
  }

  fn write_gcmake_dependencies(&self) -> io::Result<()> {
    for wrapped_graph in &self.sorted_target_info.project_order {
      let borrowed_graph = wrapped_graph.as_ref().borrow();

      if let Some(dep_info) = borrowed_graph.project_wrapper().maybe_gcmake_dep() {
        let dep_name: &str = borrowed_graph.project_identifier_name();
        let usage_conditional: UsageConditionalGroup = self.get_usage_conditional_for_dependency(&wrapped_graph.0);

        if !usage_conditional.was_used() {
          println!(
            "{} loading project [{}]: No targets from gcmake dependency '{}' are ever actually linked to an output.",
            "Warning".yellow(),
            self.project_data.get_name_for_error_messages(),
            borrowed_graph.project_debug_name()
          );
        }

        // Usage conditional
        writeln!(&self.cmakelists_file,
          "if( {} )",
          usage_conditional.full_conditional_string_or(true)
        )?;

        self.set_basic_var(
          "\n",
          &format!("{}_RELATIVE_DEP_PATH", dep_name),
          &format!("dep/{}", dep_name)
        )?;

        if !dep_info.is_using_default_features() {
          writeln!(&self.cmakelists_file,
            "\tgcmake_set_use_default_features( \"{}\" OFF )",
            dep_info.project_base_name()
          )?;
        }

        for feature_name in dep_info.specified_features() {
          writeln!(&self.cmakelists_file,
            "\tgcmake_mark_for_enable( \"{}\" \"{}\" )",
            dep_info.project_base_name(),
            feature_name
          )?;
        }

        self.write_dep_clone_code(
          dep_name,
          // GCMake projects just link using their targets as usual, since Emscripten
          // doesn't explicitly specify support for projects we just made ourselves. Makes sense.
          false,
          // GCMake projects currently only support downloading each other using the Git method.
          DownloadMethodInfo::GitMethod {
            repo_url: dep_info.repo_url().to_string(),
            revision: dep_info.revision().clone(),
          },
          true // All GCMake projects are FetchContent-ready.
        )?;


        writeln!(&self.cmakelists_file,
          "gcmake_config_file_add_contents( \"find_dependency( {} \n\tPATHS\n\t\t\\\"${{CMAKE_CURRENT_LIST_DIR}}/../{}\\\"\n)\" )",
          dep_name,
          dep_name
        )?;

        // End usage conditional
        writeln!(&self.cmakelists_file, "endif()")?;
      }
    }

    Ok(()) 
  }

  fn write_apply_dependencies(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "expose_uncached_deps()")?;

    if self.project_data.needs_fetchcontent() {
      writeln!(&self.cmakelists_file,
        "list( LENGTH ACTUAL_DEP_LIST actual_dep_list_length )\nif( actual_dep_list_length )"
      )?;
      writeln!(&self.cmakelists_file, "\n\tFetchContent_MakeAvailable( ${{ACTUAL_DEP_LIST}} )\nendif()")?;
    }

    for wrapped_graph in &self.sorted_target_info.project_order {
      let borrowed_graph = wrapped_graph.as_ref().borrow();

      if self.dep_graph_ref().get_predefined_dependencies().get(borrowed_graph.project_identifier_name()).is_none() {
        continue;
      }

      if let Some(combined_dep_info) = borrowed_graph.project_wrapper().maybe_predef_dep() {
        let dep_name: &str = borrowed_graph.project_identifier_name();
        let usage_conditional: UsageConditionalGroup = self.get_usage_conditional_for_dependency(&wrapped_graph.0);

        if !usage_conditional.was_used() {
          println!(
            "{} loading project [{}]: No targets from predefined dependency '{}' are ever actually linked to an output.",
            "Warning".yellow(),
            self.project_data.get_name_for_error_messages(),
            borrowed_graph.project_debug_name()
          );
        }

        // Usage conditional
        writeln!(&self.cmakelists_file,
          "if( {} )",
          usage_conditional.full_conditional_string_or(true)
        )?;


        if let FinalPredepInfo::Subdirectory(dep_info) = combined_dep_info.predefined_dep_info() {
          if dep_info.requires_custom_fetchcontent_populate() {
            let is_dep_internally_supported_by_emscripten: bool = dep_info.is_internally_supported_by_emscripten();
            if is_dep_internally_supported_by_emscripten {
              writeln!(&self.cmakelists_file,
                "if( NOT USING_EMSCRIPTEN )"
              )?;
            }

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

            if is_dep_internally_supported_by_emscripten {
              writeln!(&self.cmakelists_file, "endif()")?;
            }
          }
        }

        if let Some(post_load) = combined_dep_info.post_load_script() {
          writeln!(&self.cmakelists_file, "{}", post_load.contents_ref())?;
        }

        // End usage conditional
        writeln!(&self.cmakelists_file,
          "endif()"
        )?;
      }
    }

    Ok(())
  }

  fn write_root_vars(&self) -> io::Result<()> {
    // Variables shared between all targets in the current project
    self.set_basic_var("", "PROJECT_INCLUDE_PREFIX", &format!("\"{}\"", self.project_data.get_full_include_prefix()))?;
    self.set_basic_var("", "PROJECT_BASE_NAME", self.project_data.get_project_base_name())?;
    self.set_basic_var("", &self.entry_file_root_var, "\"${CMAKE_CURRENT_SOURCE_DIR}\"")?;
    // src_root path always has to be prefixed inside the entry file root for gcmake_copy_mirrored to work
    // when transforming cppfront (.cpp2) files. Luckily, this is always the case since entry files are
    // always in the project root.
    self.set_basic_var("", &self.src_root_var, &format!("\"${{{}}}/{}/${{PROJECT_INCLUDE_PREFIX}}\"", self.entry_file_root_var, SRC_DIR))?;
    self.set_basic_var("", &self.generated_src_root_var, &format!("\"${{CMAKE_CURRENT_BINARY_DIR}}/GENERATED_SOURCES\""))?;
    self.set_basic_var("", &self.header_root_var, &format!("\"${{CMAKE_CURRENT_SOURCE_DIR}}/{}/${{PROJECT_INCLUDE_PREFIX}}\"", INCLUDE_DIR))?;
    self.set_basic_var("", &self.include_dir_var, &format!("\"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\"", INCLUDE_DIR))?;

    self.write_newline()?;

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

    let mut simplified_map: BTreeMap<SpecificCompilerSpecifier, BTreeMap<&BuildType, &FinalBuildConfig>> = BTreeMap::new();

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
          simplified_map.insert(converted_compiler_specifier, BTreeMap::new());
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

  fn cmake_absolute_entry_file_path(
    &self,
    code_file_info: &CodeFileInfo
  ) -> String {
    return self.cmake_absolute_code_file_path(
      "./",
      code_file_info,
      &self.entry_file_root_var,
      &self.generated_src_root_var,
      &CodeFileTransformOptions::default()
    )
  }

  fn cmake_absolute_code_file_path(
    &self,
    file_root_dir_str: &str,
    code_file_info_in: &CodeFileInfo,
    cmake_absolute_dir_prefix: &str,
    cmake_generated_src_dir_prefix: &str,
    options: &CodeFileTransformOptions
  ) -> String {
    let mut fixed_file_path: String = code_file_info_in.get_file_path()
      .to_str()
      .unwrap()
      .to_string();

    if fixed_file_path.starts_with(file_root_dir_str) && !(code_file_info_in.uses_cpp2_grammar() && !options.should_retain_cpp2_paths) {
      fixed_file_path = fixed_file_path.replace(file_root_dir_str, "");
    }

    let used_path_prefix_var: &str = match code_file_info_in.code_file_type() {
      RetrievedCodeFileType::Source { used_grammar: CppFileGrammar::Cpp2 } if !options.should_retain_cpp2_paths => {
        // cppfront (.cpp2) generated files are always .cpp
        fixed_file_path = fixed_file_path.replace(".cpp2", ".cpp");
        cmake_generated_src_dir_prefix
      },
      _ => cmake_absolute_dir_prefix
    };

    let relative_path: &str = if fixed_file_path.starts_with('/')
      { &fixed_file_path[1..] }
      else { &fixed_file_path[..] };

    return format!("\"${{{}}}/{}\"", used_path_prefix_var, relative_path);
  }

  fn set_code_file_collection(
    &self,
    var_name: &str,
    file_location_root: &str,
    cmake_location_prefix: &str,
    cmake_generated_src_dir_prefix: &str,
    file_path_collection: &BTreeSet<impl AsRef<CodeFileInfo>>,
    options: &CodeFileTransformOptions
  ) -> io::Result<()> {
    writeln!(&self.cmakelists_file, "set( {}", var_name)?;
    for code_file_info in file_path_collection {
      writeln!(
        &self.cmakelists_file,
        "\t{}",
        self.cmake_absolute_code_file_path(
          file_location_root,
          code_file_info.as_ref(),
          cmake_location_prefix,
          cmake_generated_src_dir_prefix,
          &options
        )
      )?;
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

    let src_var_name: String = format!("{}_SOURCES", project_name);
    let includes_var_name: String = format!("{}_HEADERS", project_name);

    self.write_newline()?;

    self.set_code_file_collection(
      &src_var_name,
      self.project_data.get_src_dir_relative_to_project_root(),
      &self.src_root_var,
      &self.generated_src_root_var,
      &self.project_data.src_files,
      &CodeFileTransformOptions::default()
    )?;
    self.write_newline()?;

    self.set_code_file_collection(
      &includes_var_name,
      self.project_data.get_include_dir_relative_to_project_root(),
      &self.header_root_var,
      &self.generated_src_root_var,
      &self.project_data.include_files,
      &CodeFileTransformOptions::default()
    )?;
    self.write_newline()?;

    {
      let template_impl_var_name: String = format!(
        "{}_TEMPLATE_IMPLS",
        project_name
      );

      self.set_code_file_collection(
        &template_impl_var_name,
        self.project_data.get_include_dir_relative_to_project_root(),
        &self.header_root_var,
        &self.generated_src_root_var,
        &self.project_data.template_impl_files,
        &CodeFileTransformOptions::default()
      )?;
      self.write_newline()?;

      // Template-impl files are now treated as part of the header files list.
      writeln!(&self.cmakelists_file,
        "list( APPEND {} ${{{}}} )",
        includes_var_name,
        template_impl_var_name
      )?;
    }

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

      if self.project_data.is_test_project() {
        let parent_project_id = self.dep_graph_ref().parent_project().unwrap().as_ref().borrow().project_id();
        
        // Only build the test if every target we're testing is actually also built.
        let parent_target_existence_check: String = self.sorted_target_info.regular_targets_with_project_id(parent_project_id)
          .iter()
          .map(|parent_target|
            format!(
              "DEFINED TARGET_{}_INSTALL_MODE",
              parent_target.0.as_ref().borrow().get_name().to_string()
            )
          )
          .collect::<Vec<String>>()
          .join(" AND ");

        writeln!(&self.cmakelists_file,
          "if( {} AND ({}) )",
          parent_target_existence_check,
          system_contstraint_conditional_expression(unwrapped_target.as_ref().borrow().get_system_spec_info())
        )?;
      }
      else {
        writeln!(&self.cmakelists_file,
          "if( DEFINED TARGET_{}_INSTALL_MODE AND ({}) )",
          output_name,
          system_contstraint_conditional_expression(unwrapped_target.as_ref().borrow().get_system_spec_info())
        )?;
      }

      match matching_output.get_output_type() {
        OutputItemType::Executable => {
          self.write_executable(
            matching_output,
            &unwrapped_target,
            &output_name,
            &src_var_name,
            &includes_var_name,
            &self.include_dir_var
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
            &self.include_dir_var
          )?;
        },
        OutputItemType::CompiledLib => {
          self.write_toggle_type_library(
            matching_output,
            &unwrapped_target,
            &output_name,
            &src_var_name,
            &includes_var_name,
            &self.include_dir_var
          )?;
        }
      }

      writeln!(&self.cmakelists_file,
        "endif()"
      )?;
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

        let mut any_compiler_config: BTreeMap<BuildType, &FinalBuildConfig> = BTreeMap::new();
        let mut by_compiler: BTreeMap<SpecificCompilerSpecifier, BTreeMap<TargetSpecificBuildType, &FinalBuildConfig>> = BTreeMap::new();

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
                  by_compiler.insert(specific_specifier.clone(), BTreeMap::new());
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
    output_target_node: &Rc<RefCell<TargetNode<'a>>>,
    // This essentially means "not a pre-build script or test executable"
    is_output_installed_with_project: bool
  ) -> io::Result<()> {
    let borrowed_output_target_node = output_target_node.as_ref().borrow();

    if borrowed_output_target_node.has_links() {
      // The dependency graph already ensures there are no duplicate links. However, some libraries
      // in CMake are linked using a variable instead of targets (ex: ${wxWidgets_LIBRARIES}). That
      // variable is considered the "namespaced output target" for each target in the predefined
      // dependency. Therefore this set is used to ensure that variable is not written multiple times.
      let mut already_written: HashSet<String> = HashSet::new();

      // Emscripten has special built-in support for some libraries. Instead of linking a local copy
      // of the library, these libraries must be enabled using a '-s' flag variant passed to Emscripten.
      // See this page:
      // https://github.com/emscripten-core/emscripten/blob/main/src/settings.js
      // for a list of -s flags. Example: -sUSE_SDL=2
      let mut emscripten_link_flags_to_apply: BTreeMap<String, Vec<EmscriptenLinkFlagInfo>> = BTreeMap::new();

      let mut additional_installs: Vec<(Rc<RefCell<TargetNode>>, SystemSpecifierWrapper, LinkMode)> = Vec::new();
      let mut libs_to_link: BTreeMap<LinkMode, Vec<NormalLinkInfo>> = BTreeMap::new();

      for (given_link_mode, dep_node_list) in self.sorted_target_info.regular_dependencies_by_mode(output_target_node) {
        assert!(
          !dep_node_list.is_empty(),
          "If a link category for a target's dependencies exists in the map, then the target should have at least one dependency in that category."
        );

        let mut link_info_for_section: Vec<NormalLinkInfo> = Vec::new();

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
                &parse_leading_system_spec("((not emscripten))", None)
                  .unwrap()
                  .unwrap()
                  .value
              );
            }

            if let Some(predef_dep) = borrowed_node.container_project().as_ref().borrow().project_wrapper().maybe_predef_dep() {
              if let FinalPredepInfo::Subdirectory(_) = predef_dep.predefined_dep_info() {
                additional_installs.push(
                  (
                    Rc::clone(&dependency_node.0),
                    normal_link_constraint.clone(),
                    given_link_mode.clone()
                  )
                );
              }
            }

            let is_dep_installed_with_project: bool = match borrowed_node.container_project().as_ref().borrow().project_wrapper() {
              ProjectWrapper::GCMakeDependencyRoot(_) => true,
              ProjectWrapper::NormalProject(_) => true,
              ProjectWrapper::PredefinedDependency(predep_config) => match predep_config.predefined_dep_info() {
                FinalPredepInfo::Subdirectory(_) => true,
                _ => false
              }
            };

            link_info_for_section.push(NormalLinkInfo {
              linkable_name: linkable_target_name.to_string(),
              link_constraint: normal_link_constraint.clone(),
              unaliased_lib_var: format!(
                "_UNALIASED_{}",
                borrowed_node.get_yaml_namespaced_target_name().replace(":", "_")
              ),
              is_installed_with_project: is_dep_installed_with_project
            });

            already_written.insert(String::from(linkable_target_name));
          }

          if let Some(mut emscripten_link_flag_info) = borrowed_node.emscripten_link_flag() {
            let emscripten_constraint: SystemSpecifierWrapper = parse_leading_system_spec("((emscripten))", None)
              .unwrap()
              .unwrap()
              .value;
            
            emscripten_link_flag_info.full_flag_expression = system_constraint_generator_expression(
              &emscripten_constraint,
              &emscripten_link_flag_info.full_flag_expression
            );

            emscripten_link_flags_to_apply
              .entry(get_link_inheritance_method(output_data, given_link_mode.clone()).to_string())
              .and_modify(|flag_list| {
                if !flag_list.contains(&emscripten_link_flag_info) {
                  flag_list.push(emscripten_link_flag_info.clone());
                }
              })
              .or_insert(vec![emscripten_link_flag_info]);
          }
        }

        libs_to_link.insert(given_link_mode.clone(), link_info_for_section);
      }

      for (_, link_info_list) in &libs_to_link {
        for single_dep_info in link_info_list {
          if single_dep_info.is_installed_with_project {
            writeln!(&self.cmakelists_file,
              "if( {} )\n\tgcmake_unaliased_target_name( {} {} )\nendif()",
              system_contstraint_conditional_expression(&single_dep_info.link_constraint),
              single_dep_info.linkable_name,
              &single_dep_info.unaliased_lib_var
            )?;
          }
        }
      }
      
      writeln!(&self.cmakelists_file,
        "target_link_libraries( {} ",
        output_name
      )?;

      for (link_mode, link_info_list) in libs_to_link {
        writeln!(&self.cmakelists_file,
          "\t{}",
          get_link_inheritance_method(output_data, link_mode)
        )?;

        for single_dep_info in link_info_list {
          let inner_expression: String = if single_dep_info.is_installed_with_project {
            let mut final_expression: String = format!(
              "$<BUILD_INTERFACE:{}>",
              &single_dep_info.linkable_name,
            );

            // TODO: This is a hack. I'd rather have a way to specify that cppfront::artifacts
            // can't be installed. However, for now I'll consider cppfront a "plugin" and this a
            // special case. In the future this should be changed though.
            if single_dep_info.linkable_name != "cppfront::artifacts" {
              final_expression.push_str(&format!(
                " $<INSTALL_INTERFACE:${{LOCAL_TOPLEVEL_PROJECT_NAME}}::${{{}}}>",
                &single_dep_info.unaliased_lib_var
              ));
            }

            final_expression
          }
          else {
            single_dep_info.linkable_name
          };

          writeln!(&self.cmakelists_file,
            "\t\t{}",
            system_constraint_generator_expression(
              &single_dep_info.link_constraint,
              inner_expression
            )
          )?;
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

      if is_output_installed_with_project && !additional_installs.is_empty() {
        writeln!(&self.cmakelists_file,
          "if( DEFINED TARGET_{}_INSTALL_MODE )",
          output_name
        )?;

        // These are predefined subdirectory dependency targets which are PUBLIC or INTERFACE linked to
        // one of our project's output libraries. These targets will be transitively needed by any
        // project which makes use of our config file. These targets must be "installed" as part of
        // our project's export set so that the installed configuration knows they exist, and can
        // transitively link their properties correctly.
        // That is the only reason these are installed here. Since these libraries have been linked
        // as PUBLIC or INTERFACE, their whole project will actually install as well.
        for (dependency_node, constraint, link_mode) in additional_installs {
          writeln!(&self.cmakelists_file,
            "\tif( {} )",
            system_contstraint_conditional_expression(&constraint),
          )?;

          let namespaced_name: String = dependency_node.as_ref().borrow().get_cmake_namespaced_target_name().to_string();
          let base_name = dependency_node.as_ref().borrow().container_project().as_ref().borrow().root_project().as_ref().borrow().project_identifier_name().to_string();

          match &link_mode {
            LinkMode::Public | LinkMode::Interface => {
              writeln!(&self.cmakelists_file,
                "\t\tadd_to_additional_dependency_install_list( {} \"${{{}_RELATIVE_DEP_PATH}}\" )",
                namespaced_name,
                base_name
              )?;
            },
            LinkMode::Private => {
              writeln!(&self.cmakelists_file,
                "\t\tadd_to_minimal_installs( {} \"${{{}_RELATIVE_DEP_PATH}}\")",
                namespaced_name,
                base_name
              )?;
            }
          }

          writeln!(&self.cmakelists_file, "\tendif()",)?;
        }

        writeln!(&self.cmakelists_file, "endif()")?;
      }
    }

    Ok(())
  }

  fn write_properties_for_output(
    &self,
    output_name: &str,
    property_map: &BTreeMap<String, String>
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
      "if( \"${{TARGET_{}_INSTALL_MODE}}\" STREQUAL \"FULL\" )",
      target_name
    )?;
    writeln!(&self.cmakelists_file,
      "\tadd_to_target_installation_list( {} )",
      target_name
    )?;
    writeln!(&self.cmakelists_file,
      "elseif( \"${{TARGET_{}_INSTALL_MODE}}\" STREQUAL \"MINIMAL\" )",
      target_name
    )?;
    writeln!(&self.cmakelists_file,
      "\tadd_to_minimal_installs( {} \"\" )",
      target_name
    )?;
    writeln!(&self.cmakelists_file,
      "endif()"
    )?;
    self.write_newline()?;

    let entry_file_varname: String = format!("{}_ENTRY_FILE", target_name);

    self.set_basic_var(
      "",
      &entry_file_varname,
      &self.cmake_absolute_entry_file_path(output_data.get_entry_file())
    )?;

    writeln!(&self.cmakelists_file,
      "gcmake_add_to_documentable_files_list( {} )",
      entry_file_varname
    )?;

    writeln!(&self.cmakelists_file,
      "gcmake_apply_lib_files( {} {} \"${{{}}}\" {} {} )",
      target_name,
      lib_spec_string,
      entry_file_varname,
      src_var_name,
      includes_var_name
    )?;

    writeln!(&self.cmakelists_file,
      "gcmake_apply_include_dirs( {} {} \"${{{}}}\" )",
      target_name,
      lib_spec_string,
      &project_include_dir_varname
    )?;
    self.write_newline()?;

    let language_extensions_on_off: &str = on_or_off_str(self.project_data.are_language_extensions_enabled());

    self.write_properties_for_output(
      &target_name,
      &BTreeMap::from([
        (String::from("RUNTIME_OUTPUT_DIRECTORY"), String::from(RUNTIME_BUILD_DIR_VAR)),
        (String::from("LIBRARY_OUTPUT_DIRECTORY"), String::from(LIB_BUILD_DIR_VAR)),
        (String::from("ARCHIVE_OUTPUT_DIRECTORY"), String::from(LIB_BUILD_DIR_VAR)),
        (String::from("C_EXTENSIONS"), String::from(language_extensions_on_off)),
        (String::from("CXX_EXTENSIONS"), String::from(language_extensions_on_off))
      ])
    )?;
    self.write_newline()?;

    self.write_depends_on_pre_build(&target_name)?;

    self.write_flag_and_define_vars_for_output(output_name, output_data)?;
    self.write_defines_for_output(output_name, output_data, &target_name)?;
    self.write_target_link_options_for_output(output_name, output_data, &target_name)?;
    self.write_target_compile_options_for_output(output_name, output_data, &target_name)?;
    self.write_newline()?;

    self.write_links_for_output(&target_name, output_data, output_target_node, true)?;
    Ok(())
  }

  fn write_executable(
    &self,
    output_data: &CompiledOutputItem,
    output_target_node: &Rc<RefCell<TargetNode<'a>>>,
    output_name: &str,
    src_var_name: &str,
    includes_var_name: &str,
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
        "add_executable( {} {} )",
        target_name,
        self.cmake_absolute_entry_file_path(output_data.get_entry_file())
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

    if let Some(emscripten_html_shell_relative_path) = &output_data.emscripten_html_shell_relative_to_project_root {
      writeln!(&self.cmakelists_file,
        "\tuse_custom_emscripten_shell_file( {} \"${{TOPLEVEL_PROJECT_DIR}}/{}\" )",
        target_name,
        emscripten_html_shell_relative_path.to_str().unwrap()
      )?;
    }

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

      // This is fine for now because executable minimal installs are the same as their
      // full installs.
      writeln!(&self.cmakelists_file,
        "if( DEFINED TARGET_{}_INSTALL_MODE )",
        target_name
      )?;
      writeln!(&self.cmakelists_file,
        "\tadd_to_target_installation_list( {} )",
        target_name
      )?;
      writeln!(&self.cmakelists_file, "endif()")?;

      writeln!(&self.cmakelists_file,
        "gcmake_apply_include_dirs( {} EXE_RECEIVER \"${{{}}}\" )",
        receiver_lib_name,
        &project_include_dir_varname
      )?;

      writeln!(&self.cmakelists_file,
        "gcmake_apply_exe_files( {} {} \n\t{}\n\t{}\n\t{}\n)",
        target_name,
        receiver_lib_name,
        self.cmake_absolute_entry_file_path(output_data.get_entry_file()),
        src_var_name,
        includes_var_name
      )?;
      self.write_newline()?;
    }

    let language_extensions_on_off: &str = on_or_off_str(self.project_data.are_language_extensions_enabled());

    // TODO: Might need to write these for the receiver lib too. Not sure though.
    self.write_properties_for_output(
      target_name,
      &BTreeMap::from([
        (String::from("RUNTIME_OUTPUT_DIRECTORY"), String::from(RUNTIME_BUILD_DIR_VAR)),
        (String::from("C_EXTENSIONS"), String::from(language_extensions_on_off)),
        (String::from("CXX_EXTENSIONS"), String::from(language_extensions_on_off))
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

    self.write_links_for_output(
      receiver_lib_name,
      output_data,
      output_target_node,
      !is_pre_build_script && !self.project_data.is_test_project()
    )?;

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

  fn write_documentation_generation(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "gcmake_add_to_documentable_files_list( ${{PROJECT_BASE_NAME}}_HEADERS )",
    )?;

    writeln!(&self.cmakelists_file, "if( NOT ${{LOCAL_TOPLEVEL_PROJECT_NAME}}_DOCUMENT_HEADERS_ONLY )")?;
    writeln!(&self.cmakelists_file,
      "gcmake_add_to_documentable_files_list( ${{PROJECT_BASE_NAME}}_SOURCES )",
    )?;
    writeln!(&self.cmakelists_file, "endif()\n")?;

    if !self.project_data.is_root_project() && !self.project_data.is_test_project() {
      writeln!(&self.cmakelists_file, "gcmake_raise_documentable_files_list()")?;
    }

    if self.project_data.is_root_project() {
      writeln!(&self.cmakelists_file, "if( ${{PROJECT_NAME}}_BUILD_DOCS )")?;

      if let Some(doc_info) = self.project_data.get_documentation_config() {
        match &doc_info.generator {
          FinalDocGeneratorName::Doxygen => {
            writeln!(&self.cmakelists_file, "gcmake_use_doxygen( DOCUMENTABLE_FILES )")?;
          }
        }
      }

      writeln!(&self.cmakelists_file, "endif()")?;
    }

    Ok(())
  }

  // See this page for help and a good example:
  // https://cmake.org/cmake/help/latest/guide/tutorial/Adding%20Export%20Configuration.html
  fn write_installation_and_exports(&self) -> io::Result<()> {
    let write_raise_functions: &dyn Fn(&str) -> io::Result<()> = &|spacer: &str| {
      writeln!(&self.cmakelists_file, "{}raise_deb_list()", spacer)?;
      writeln!(&self.cmakelists_file, "{}raise_minimal_installs()", spacer)?;
      writeln!(&self.cmakelists_file, "{}raise_target_list()", spacer)?;
      writeln!(&self.cmakelists_file, "{}raise_needed_bin_files_list()", spacer)?;
      writeln!(&self.cmakelists_file, "{}raise_additional_dependency_install_list()", spacer)?;
      writeln!(&self.cmakelists_file, "{}raise_generated_export_headers_list()", spacer)?;
      // NOTE: The call to gcmake_raise_documentable_files_list(...) is done in the
      // write_documentation_generator(...) function in this file.
      Ok(())
    };

    match &self.project_data.get_project_type() {
      FinalProjectType::Root => {
        writeln!(&self.cmakelists_file,
          "if( GCMAKE_INSTALL )\n\t{}\n\t{}",
          "\tconfigure_installation( LOCAL_PROJECT_COMPONENT_NAME )",
          "\tif( NOT \"${CMAKE_CURRENT_SOURCE_DIR}\" STREQUAL \"${TOPLEVEL_PROJECT_DIR}\" )"
        )?;
        write_raise_functions("\t\t")?;
        writeln!(&self.cmakelists_file, "\tendif()\nendif()")?;
      },
      FinalProjectType::Subproject { } => {
        write_raise_functions("")?;
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
      // Make sure this variable is the same when including test projects in write_use_test_projects(...)
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

    writeln!(&self.cmakelists_file, "endif()")?;
    Ok(())
  }

  fn write_use_test_projects(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "if( ${{LOCAL_TOPLEVEL_PROJECT_NAME}}_BUILD_TESTS )"
    )?;

    for (test_name, _) in self.dep_graph_ref().get_test_projects() {
      writeln!(&self.cmakelists_file,
        "\tadd_subdirectory( \"${{CMAKE_CURRENT_SOURCE_DIR}}/tests/{}\" )",
        test_name
      )?;
    }

    writeln!(&self.cmakelists_file, "endif()")?;

    Ok(())
  }

  // Only called by the root project.
  fn write_features(&self) -> io::Result<()> {
    writeln!(&self.cmakelists_file,
      "if( NOT DEFINED ${{LOCAL_TOPLEVEL_PROJECT_NAME}}_USE_DEFAULT_FEATURES )"
    )?;

    writeln!(&self.cmakelists_file,
      "gcmake_set_use_default_features( ${{LOCAL_TOPLEVEL_PROJECT_NAME}} ON )"
    )?;

    writeln!(&self.cmakelists_file, "endif()")?;

    for (feature_name, feature_config) in self.project_data.get_features() {
      let mut dep_enable_pairs: Vec<(&str, &str)> = Vec::new();
      let mut normal_enable_names: Vec<&str> = Vec::new();

      for enabler_config in &feature_config.enables {
        match &enabler_config.dep_name {
          None => normal_enable_names.push(&enabler_config.feature_name),
          Some(enabler_dep_name) => dep_enable_pairs.push(
            (
              enabler_dep_name,
              &enabler_config.feature_name
            )
          )
        }
      }

      write!(&self.cmakelists_file,
        "gcmake_register_feature( NAME {}",
        feature_name
      )?;

      if !normal_enable_names.is_empty() {
        write!(&self.cmakelists_file, "\n\tENABLES")?;

        for enables_feature_name in &normal_enable_names {
          write!(&self.cmakelists_file,
            " {}",
            enables_feature_name
          )?;
        }
      }

      if !dep_enable_pairs.is_empty() {
        writeln!(&self.cmakelists_file, "\n\tDEP_ENABLES")?;

        for (gcmake_dep_identifier, enables_feature_name) in dep_enable_pairs {
          let internal_gcmake_dep_project_name: String = self.dep_graph_ref()
            .root_project().as_ref().borrow()
            .get_gcmake_dependencies()
            .get(gcmake_dep_identifier)
            .unwrap().as_ref().borrow()
            .internal_project_name()
            .to_string();
            
          writeln!(&self.cmakelists_file,
            "\t\t\"{}\" \"{}\"",
            internal_gcmake_dep_project_name,
            enables_feature_name
          )?;
        }
      }

      writeln!(&self.cmakelists_file, ")")?;
    }

    writeln!(&self.cmakelists_file,
      "if( ${{LOCAL_TOPLEVEL_PROJECT_NAME}}_USE_DEFAULT_FEATURES )"
    )?;

    for (feature_name, feature_config) in self.project_data.get_features() {
      if feature_config.is_enabled_by_default {
        writeln!(&self.cmakelists_file,
          "\tgcmake_mark_for_enable( ${{LOCAL_TOPLEVEL_PROJECT_NAME}} {} )",
          feature_name
        )?;
      }
    }

    writeln!(&self.cmakelists_file, "endif()")?;
    self.write_newline()?;

    for (feature_name, _) in self.project_data.get_features() {
      writeln!(&self.cmakelists_file,
        "gcmake_enable_feature_if_marked( {} )",
        feature_name
      )?;
    }

    Ok(())
  }

  fn dep_graph_ref(&self) -> Ref<DependencyGraph<'a>> {
    return self.dep_graph.as_ref().borrow();
  }

  // NOTE: This should only be called from the root project
  fn get_usage_conditional_for_dependency(
    &self,
    graph_for_dependency: &Rc<RefCell<DependencyGraph<'a>>>
  ) -> UsageConditionalGroup<'a> {
    let root_dep_id = graph_for_dependency.as_ref().borrow().root_project_id();
    
    // TODO: Refactor this into something easier to read.
    let constraints_for_used_links: Vec<SingleUsageConditional> = self.sorted_target_info.all_targets_with_root_project_id(self.dep_graph_ref().root_project_id())
      .iter()
      .filter_map(|wrapped_project_target| {
        let dependent_system_specs: Vec<(LinkMode, SystemSpecifierWrapper)> = wrapped_project_target.as_ref().borrow().get_depends_on()
          .iter()
          .filter(|(_, link) | {
            let root_project_id_of_linked_target = link.linked_target().as_ref().borrow().container_project().as_ref().borrow().root_project_id();
            root_project_id_of_linked_target == root_dep_id
          })
          .map(|(_, link)| 
            (
              link.get_link_mode(),
              // TODO: This doesn't directly take into account constraints on the linked dependency target itself;
              // only constraints placed on the link to the target. I think that's fine, since
              // GCMake will issue a warning explaining that the constraint given to the link
              // must be a subset of the constraint given to the linked dependency target (i.e. ((windows)) ).
              // However, it might be worth ANDing these for correctness. I'll have to wait and see. The
              // commented out block below would be what we'd AND this one with.
              link.get_system_spec_info().clone()
              // link.linked_target().as_ref().borrow().get_system_spec_info().clone()
            )
          )
          .collect();

        let project_target_constraint: SystemSpecifierWrapper = wrapped_project_target.as_ref().borrow().get_system_spec_info().clone();

        let public_needed_constraint = dependent_system_specs.iter()
          .filter_map(|(link_mode, system_spec)| match link_mode {
            LinkMode::Public | LinkMode::Interface => Some(system_spec.clone()),
            LinkMode::Private => None
          })
          .reduce(|accum, item| accum.union(&item))
          .map(|links_constraint| links_constraint.intersection(&project_target_constraint));

        let private_needed_constraint = dependent_system_specs.into_iter()
          .filter_map(|(link_mode, system_spec)| match link_mode {
            LinkMode::Private => Some(system_spec),
            LinkMode::Public | LinkMode::Interface => None,
          })
          .reduce(|accum, item| accum.union(&item))
          .map(|links_constraint| links_constraint.intersection(&project_target_constraint));
        
        match (&public_needed_constraint, &private_needed_constraint) {
          (None, None) => None,
          _ => Some(SingleUsageConditional {
            public_constraint: public_needed_constraint,
            private_constraint: private_needed_constraint,
            target_rc: Rc::clone(&wrapped_project_target.0)
          })
        }
      })
      .collect();

    return UsageConditionalGroup {
      all_conditionals: constraints_for_used_links
    }
  }
}

fn union_maybe_specs(
  first: Option<&SystemSpecifierWrapper>,
  second: Option<&SystemSpecifierWrapper>
) -> Option<SystemSpecifierWrapper> {
  match (first, second) {
    (None, None) => None,
    (Some(spec), None) => Some(spec.clone()),
    (None, Some(spec)) => Some(spec.clone()),
    (Some(spec), Some(other_spec)) => Some(spec.union(&other_spec))
  }
}


fn get_link_inheritance_method(
  output_data: &CompiledOutputItem,
  given_link_mode: LinkMode
) -> &str {
  match output_data.get_output_type() {
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
  }
}