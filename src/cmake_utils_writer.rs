use std::{collections::HashMap, fs::{self}, io, iter::FromIterator, path::{PathBuf}};


pub struct CMakeUtilWriter {
  cmake_utils_path: PathBuf,
  utils: HashMap<&'static str, &'static str>
}

impl CMakeUtilWriter {
  pub fn new(cmake_utils_path: PathBuf) -> Self {
    return Self {
      cmake_utils_path,
      utils: HashMap::from_iter([
        ("toggle-lib-util", TOGGLE_LIB_UTIL_TEXT),
        ("pre-build-configuration-utils", PREBUILD_STEP_UTILS_TEXT),
        ("resource-copy-util", RESOURCE_COPY_UTIL_TEXT),
        ("general-utils", GENERAL_FUNCTIONS_UTIL_TEXT),
        ("installation-utils", INSTALLATION_CONFIGURE_TEXT)
      ])
    }
  }

  pub fn write_cmake_utils(&self) -> io::Result<()> {
    if !self.cmake_utils_path.is_dir() {
      fs::create_dir(&self.cmake_utils_path)?;
    }

    for (util_name, util_contents) in &self.utils {
      let mut util_file_path = self.cmake_utils_path.join(util_name);
      util_file_path.set_extension("cmake");

      fs::write(
        util_file_path,
        util_contents
      )?;
    }

    Ok(())
  }

  pub fn get_utils(&self) -> &HashMap<&'static str, &'static str> {
    &self.utils
  }
}

const GENERAL_FUNCTIONS_UTIL_TEXT: &'static str = 
r#"function( get_without_source_dir_prefix
  all_files
  receiving_var
)
  string( REPLACE "${CMAKE_CURRENT_SOURCE_DIR}/" "" with_removed_prefix "${all_files}" )
  string( REPLACE "./" "" with_removed_prefix "${with_removed_prefix}" )
  set( ${receiving_var} "${with_removed_prefix}" PARENT_SCOPE )
endfunction()

function( apply_exe_files
  exe_target
  entry_file
  sources
  headers
  template_impls
)
  set( all_sources "${entry_file};${sources}" )
  get_without_source_dir_prefix( "${all_sources}" all_sources_install_interface )

  target_sources( ${exe_target} PUBLIC
    "$<BUILD_INTERFACE:${all_sources}>"
    "$<INSTALL_INTERFACE:${all_sources_install_interface}>"
  )

  list( JOIN headers template_impls all_headers )

  if( NOT "${all_headers}" STREQUAL "" )
    string( REPLACE "${CMAKE_CURRENT_SOURCE_DIR}/" "" install_interface_all_headers "${all_headers}")
    get_without_source_dir_prefix( "${all_headers}" all_headers_install_interface )

    target_sources( ${exe_target} PUBLIC FILE_SET HEADERS
      FILES
        "$<BUILD_INTERFACE:${all_headers}>"
        "$<INSTALL_INTERFACE:${all_headers_install_interface}>"
    )
  endif()
endfunction()

function( apply_lib_files
  lib_target
  entry_file
  sources
  headers
  template_impls
)
  if( NOT "${sources}" STREQUAL "" )
    get_without_source_dir_prefix( "${sources}" all_sources_install_interface )
    target_sources( ${lib_target} PUBLIC
      "$<BUILD_INTERFACE:${sources}>"
      "$<INSTALL_INTERFACE:${all_sources_install_interface}>"
    )
  endif()

  set( all_headers "${entry_file}" )
  list( JOIN all_headers headers all_headers )
  list( JOIN all_headers template_impls all_headers )

  get_without_source_dir_prefix( "${all_headers}" all_headers_install_interface )

  target_sources( ${lib_target} PUBLIC FILE_SET HEADERS
    FILES
      "$<BUILD_INTERFACE:${all_headers}>"
      "$<INSTALL_INTERFACE:${all_headers_install_interface}>"
  )
endfunction()

function( apply_include_dirs
  target
  target_type
  project_include_dir
)
  if( "${target_type}" STREQUAL "COMPILED_LIB" )
    set( BUILD_INTERFACE_INCLUDE_DIRS "${CMAKE_CURRENT_SOURCE_DIR};${project_include_dir}")
  elseif( "${target_type}" STREQUAL "EXE" )
    set( BUILD_INTERFACE_INCLUDE_DIRS "${project_include_dir}")
  else()
    message( FATAL_ERROR "Invalid target_type '${target_type}' given to function 'apply_include_dirs'" )
  endif()

  target_include_directories( ${target}
    PUBLIC
      "$<BUILD_INTERFACE:${BUILD_INTERFACE_INCLUDE_DIRS}>"
      "$<INSTALL_INTERFACE:include/${PROJECT_NAME}>"
  )
endfunction()
"#;


const TOGGLE_LIB_UTIL_TEXT: &'static str = 
r#"function( make_toggle_lib
  lib_name
  default_lib_type
)
  if (NOT "${default_lib_type}" STREQUAL "STATIC" AND NOT "${default_lib_type}" STREQUAL "SHARED")
    message( FATAL_ERROR "Invalid default lib type '${default_lib_type}' given to type toggleable library ${lib_name}" )
  endif()

  if( NOT ${lib_name}_LIB_TYPE )
    set( ${lib_name}_LIB_TYPE ${default_lib_type} CACHE STRING "Library type for ${lib_name}" )
  endif()

  set_property( CACHE ${lib_name}_LIB_TYPE PROPERTY STRINGS "STATIC" "SHARED" )

  if ( ${lib_name}_LIB_TYPE STREQUAL STATIC )
    add_library( ${lib_name} STATIC )
  elseif( ${lib_name}_LIB_TYPE STREQUAL SHARED )
    add_library( ${lib_name} SHARED )
  endif()
endfunction()
"#;

const PREBUILD_STEP_UTILS_TEXT: &'static str =
r#"function( initialize_prebuild_step )
  add_custom_target( ${PROJECT_NAME}-pre-build-step
    ALL
    COMMENT "Beginning pre-build processing"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
  )
endfunction()

function( use_executable_prebuild_script
  pre_build_executable_target
)
  add_custom_command(
    TARGET ${PROJECT_NAME}-pre-build-step
    PRE_BUILD
    COMMAND ${pre_build_executable_target}
    COMMENT "Running ${PROJECT_NAME} pre-build executable script"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
  )
endfunction()

function( use_python_prebuild_script
  python_prebuild_file
)
  include( FindPython3 )
  find_package( Python3 COMPONENTS Interpreter )

  if( ${Python3_FOUND} AND ${Python3_Interpreter_FOUND} )
    add_custom_command(
      TARGET ${PROJECT_NAME}-pre-build-step
      PRE_BUILD
      COMMAND Python3::Interpreter ${python_prebuild_file}
      COMMENT "Running ${PROJECT_NAME} pre-build python script"
      WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    )
  else()
    if( NOT ${Python3_Interpreter_FOUND} )
      message( FATAL_ERROR "A Python 3 interpreter is needed to run the pre-build script for project ${PROJECT_NAME}, however a valid interpreter was not found." )
    else()
      message( FATAL_ERROR "Unable to find a valid Python 3 configuration when configuring project ${PROJECT_NAME}" )
    endif()
  endif()
endfunction()
"#;

const RESOURCE_COPY_UTIL_TEXT: &'static str =
r#"function( copy_resource_dir_if_exists
  resources_dir
  build_time_resource_dir_location
  pre_build_step_target
)
  if( EXISTS ${resources_dir} )
    add_custom_command(
      TARGET ${PROJECT_NAME}-pre-build-step
      PRE_BUILD
      COMMAND ${CMAKE_COMMAND}
        -E copy_directory ${resources_dir} ${build_time_resource_dir_location}
      COMMENT "Copying ${PROJECT_NAME} resources"
      VERBATIM
    )
  endif()
endfunction()
"#;

const INSTALLATION_CONFIGURE_TEXT: &'static str = r#"function( configure_installation
  project_version
  targets_installing
)
  if( NOT "${targets_installing}" STREQUAL "" )
    install( TARGETS ${targets_installing}
      EXPORT ${PROJECT_NAME}Targets
      RUNTIME 
        DESTINATION bin
      LIBRARY
        DESTINATION lib
      ARCHIVE
        DESTINATION lib/static
      FILE_SET HEADERS
        DESTINATION .
    )
  
    install( EXPORT ${PROJECT_NAME}Targets
      FILE ${PROJECT_NAME}Targets.cmake
      DESTINATION "lib/cmake/${PROJECT_NAME}"
    )

    include( CMakePackageConfigHelpers )

    configure_package_config_file( "${CMAKE_CURRENT_SOURCE_DIR}/Config.cmake.in"
      "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake"
      INSTALL_DESTINATION "lib/cmake"
    )

    # TODO: Allow configuration of COMPATIBILITY
    write_basic_package_version_file(
      "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}ConfigVersion.cmake"
      VERSION "${PROJECT_VERSION}"
      COMPATIBILITY AnyNewerVersion
    )

    install( FILES 
      "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake"
      "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}ConfigVersion.cmake"
      DESTINATION "lib/cmake/${PROJECT_NAME}"
    )

    export( EXPORT ${PROJECT_NAME}Targets
      FILE "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Targets.cmake"
    )
  endif()

endfunction()
"#;