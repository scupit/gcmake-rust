use std::{collections::HashMap, fs::{self}, io, iter::FromIterator, path::{PathBuf}};


pub struct CMakeUtilFile {
  pub util_name: &'static str,
  pub util_contents: &'static str
}

pub struct CMakeUtilWriter {
  cmake_utils_path: PathBuf,
  utils: Vec<CMakeUtilFile>
}

impl CMakeUtilWriter {
  pub fn new(cmake_utils_path: PathBuf) -> Self {
    return Self {
      cmake_utils_path,
      // TODO: Make all these their own *.cmake files, so they are easier to maintain.
      // Load them here using a pre-build script.
      utils: vec![
        CMakeUtilFile {
          util_name: "gcmake-variables",
          util_contents: USEFUL_VARIABLES_TEXT
        },
        CMakeUtilFile {
          util_name: "toggle-lib-util",
          util_contents: TOGGLE_LIB_UTIL_TEXT
        },
        CMakeUtilFile {
          util_name: "pre-build-configuration-utils",
          util_contents: PREBUILD_STEP_UTILS_TEXT
        },
        CMakeUtilFile {
          util_name: "resource-copy-util",
          util_contents: RESOURCE_COPY_UTIL_TEXT
        },
        CMakeUtilFile {
          util_name: "general-utils",
          util_contents: GENERAL_FUNCTIONS_UTIL_TEXT
        },
        CMakeUtilFile {
          util_name: "installation-utils",
          util_contents: INSTALLATION_CONFIGURE_TEXT
        },
        CMakeUtilFile {
          util_name: "gcmake-cpack-utils",
          util_contents: GCMAKE_CPACK_CONFIGURE_TEXT
        }
      ]
    }
  }

  pub fn write_cmake_utils(&self) -> io::Result<()> {
    if !self.cmake_utils_path.is_dir() {
      fs::create_dir(&self.cmake_utils_path)?;
    }

    // for (util_name, util_contents) in &self.utils {
    for CMakeUtilFile {util_name, util_contents} in &self.utils {
      let mut util_file_path = self.cmake_utils_path.join(util_name);
      util_file_path.set_extension("cmake");

      fs::write(
        util_file_path,
        util_contents
      )?;
    }

    Ok(())
  }

  pub fn get_utils(&self) -> &Vec<CMakeUtilFile> {
    &self.utils
  }
}

const USEFUL_VARIABLES_TEXT: &'static str =
r#"if( "${CMAKE_C_COMPILER_ID}" MATCHES "GNU" OR "${CMAKE_CXX_COMPILER_ID}" MATCHES "GNU" )
  set( USING_GCC TRUE )
else()
  set( USING_GCC FALSE )
endif()

if( "${CMAKE_C_COMPILER_ID}" MATCHES "Clang" OR "${CMAKE_CXX_COMPILER_ID}" MATCHES "Clang" )
  set( USING_CLANG TRUE )
else()
  set( USING_CLANG FALSE )
endif()

set( USING_MSVC ${MSVC} )

if( CMAKE_HOST_UNIX AND NOT CMAKE_HOST_APPLE )
  set( CURRENT_SYSTEM_IS_LINUX TRUE )
else()
  set( CURRENT_SYSTEM_IS_LINUX FALSE )
endif()

set( CURRENT_SYSTEM_IS_WINDOWS ${CMAKE_HOST_WIN32} )
set( CURRENT_SYSTEM_IS_APPLE ${CMAKE_HOST_APPLE} )

if( UNIX AND NOT APPLE )
  set( TARGET_SYSTEM_IS_LINUX TRUE )
else()
  set( TARGET_SYSTEM_IS_LINUX FALSE )
endif()

set( TARGET_SYSTEM_IS_WINDOWS ${WIN32} )
set( TARGET_SYSTEM_IS_APPLE ${APPLE} )
"#;

const GENERAL_FUNCTIONS_UTIL_TEXT: &'static str = 
r#"function( clean_list
  content
  output_var
)
  string( REGEX REPLACE "(^ *;)|(; *$)" "" cleaned_list_out "${content}" )
  # string( REGEX REPLACE ";" " " cleaned_list_out "${cleaned_list_out}" )
  set( ${output_var} "${cleaned_list_out}" PARENT_SCOPE )
endfunction()

function( get_without_source_dir_prefix
  all_files
  receiving_var
)
  string( REPLACE "${CMAKE_CURRENT_SOURCE_DIR}/" "" with_removed_prefix "${all_files}" )
  string( REPLACE "./" "" with_removed_prefix "${with_removed_prefix}" )
  clean_list( "${with_removed_prefix}" with_removed_prefix )
  set( ${receiving_var} "${with_removed_prefix}" PARENT_SCOPE )
endfunction()

function( make_generators
  for_build
  for_install
  var_name
)
  foreach( file_for_build IN LISTS for_build )
    set( ${var_name}_b "${${var_name}_b}" "$<BUILD_INTERFACE:${file_for_build}>" )
  endforeach()

  foreach( file_for_install IN LISTS for_install )
    set( ${var_name}_i "${${var_name}_i}" "$<INSTALL_INTERFACE:${the_file_for_install}>" )
  endforeach()

  set( ${var_name}_b "${${var_name}_b}" PARENT_SCOPE )
  set( ${var_name}_i "${${var_name}_i}" PARENT_SCOPE )
endfunction()

function( apply_exe_files
  exe_target
  entry_file
  sources
  headers
  template_impls
)
  set( all_sources ${entry_file};${sources} )
  clean_list( "${all_sources}" all_sources )
  get_without_source_dir_prefix( "${all_sources}" all_sources_install_interface )

  make_generators( "${all_sources}" "${all_sources_install_interface}" source_gens )
  target_sources( ${exe_target} PUBLIC
    ${source_gens_b}
    ${source_gens_i}
  )

  set( all_headers "${headers};${template_impls}" )
  clean_list( "${all_headers}" all_headers )

  if( NOT "${all_headers}" STREQUAL "" )
    get_without_source_dir_prefix( "${all_headers}" all_headers_install_interface )

    make_generators( "${all_headers}" "${all_headers_install_interface}" header_gens )
    target_sources( ${exe_target} PUBLIC FILE_SET HEADERS
      FILES
        ${header_gens_b}
        ${header_gens_i}
    )
  endif()
endfunction()

function( apply_lib_files
  lib_target
  lib_type_spec
  entry_file
  sources
  headers
  template_impls
)
  if( NOT "${lib_type_spec}" STREQUAL "COMPILED_LIB" AND NOT "${lib_type_spec}" STREQUAL "HEADER_ONLY_LIB" )
    message( FATAL_ERROR "Invalid lib type spec '${lib_type_spec}' given to apply_lib_files(...)" )
  endif()

  if( "${lib_type_spec}" STREQUAL "COMPILED_LIB" )
    clean_list( "${sources}" all_sources)

    if( NOT "${all_sources}" STREQUAL "" )
      get_without_source_dir_prefix( "${all_sources}" all_sources_install_interface )

      make_generators( "${all_sources}" "${all_sources_install_interface}" source_gens )
      target_sources( ${lib_target} PUBLIC
        ${source_gens_b}
        ${source_gens_i}
      )
    endif()
  endif()

  set( all_headers "${entry_file};${headers};${template_impls}" )
  clean_list( "${all_headers}" all_headers )

  get_without_source_dir_prefix( "${all_headers}" all_headers_install_interface )

  if( "${lib_type_spec}" STREQUAL "HEADER_ONLY_LIB" )
    set( header_inheritance_mode INTERFACE )
  else()
    set( header_inheritance_mode PUBLIC )
  endif()

  make_generators( "${all_headers}" "${all_headers_install_interface}" header_gens )
  target_sources( ${lib_target} ${header_inheritance_mode}
    FILE_SET HEADERS
      FILES
        ${header_gens_b}
        ${header_gens_i}
  )
endfunction()

function( apply_include_dirs
  target
  target_type
  project_include_dir
)
  if( "${target_type}" STREQUAL "COMPILED_LIB" OR "${target_type}" STREQUAL "HEADER_ONLY_LIB" )
    set( BUILD_INTERFACE_INCLUDE_DIRS "${CMAKE_CURRENT_SOURCE_DIR};${project_include_dir}")
  elseif( "${target_type}" STREQUAL "EXE" )
    set( BUILD_INTERFACE_INCLUDE_DIRS "${project_include_dir}")
  else()
    message( FATAL_ERROR "Invalid target_type '${target_type}' given to function 'apply_include_dirs'" )
  endif()

  if( "${target_type}" STREQUAL "HEADER_ONLY_LIB" )
    set( include_dir_inheritance_mode INTERFACE )
  else()
    set( include_dir_inheritance_mode PUBLIC )
  endif()

  target_include_directories( ${target}
    ${include_dir_inheritance_mode}
      "$<BUILD_INTERFACE:${BUILD_INTERFACE_INCLUDE_DIRS}>"
      "$<INSTALL_INTERFACE:include>"
      "$<INSTALL_INTERFACE:include/${TOPLEVEL_INCLUDE_PREFIX}/include>"
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

  if( NOT ${LOCAL_TOPLEVEL_PROJECT_NAME}__${lib_name}_LIB_TYPE )
    if( "${LOCAL_TOPLEVEL_PROJECT_NAME}" STREQUAL "${PROJECT_NAME}" )
      set( PROJECT_SPECIFIER "'${LOCAL_TOPLEVEL_PROJECT_NAME}'")
    else()
      set( PROJECT_SPECIFIER "'${PROJECT_NAME}' (part of ${LOCAL_TOPLEVEL_PROJECT_NAME})")
    endif()

    set( ${LOCAL_TOPLEVEL_PROJECT_NAME}__${lib_name}_LIB_TYPE ${default_lib_type} CACHE STRING "Library type for '${lib_name}' in project ${PROJECT_SPECIFIER}" )
  endif()

  set_property( CACHE ${LOCAL_TOPLEVEL_PROJECT_NAME}__${lib_name}_LIB_TYPE PROPERTY STRINGS "STATIC" "SHARED" )

  if ( ${LOCAL_TOPLEVEL_PROJECT_NAME}__${lib_name}_LIB_TYPE STREQUAL STATIC )
    add_library( ${lib_name} STATIC )
  elseif( ${LOCAL_TOPLEVEL_PROJECT_NAME}__${lib_name}_LIB_TYPE STREQUAL SHARED )
    add_library( ${lib_name} SHARED )
    set_target_properties( ${lib_name}
      PROPERTIES
        WINDOWS_EXPORT_ALL_SYMBOLS TRUE
    )
  else()
    # This shouldn't happen, but it's worth keeping the error check just in case.
    message( FATAL_ERROR "${LOCAL_TOPLEVEL_PROJECT_NAME}__${lib_name}_LIB_TYPE was given neither value 'STATIC' or 'SHARED'.")
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

const INSTALLATION_CONFIGURE_TEXT: &'static str = r#"function( configure_installation )
  set( targets_installing "${MY_INSTALLABLE_TARGETS}" )
  set( bin_files_installing "${MY_NEEDED_BIN_FILES}" )

  set( additional_installs "${MY_ADDITIONAL_INSTALL_TARGETS}" )
  list( REMOVE_DUPLICATES additional_installs )

  set( additional_relative_dep_paths "${MY_ADDITIONAL_RELATIVE_DEP_PATHS}" )
  list( TRANSFORM additional_relative_dep_paths PREPEND "include/" )
  list( REMOVE_DUPLICATES additional_relative_dep_paths )

  list( LENGTH targets_installing has_targets_to_install )
  list( LENGTH bin_files_installing has_files_to_install )
  list( LENGTH additional_installs has_additional_installs )

  if( has_targets_to_install )
    install( TARGETS ${targets_installing}
      EXPORT ${PROJECT_NAME}Targets
      RUNTIME 
        DESTINATION bin
      LIBRARY
        DESTINATION lib
      ARCHIVE
        DESTINATION lib
        # DESTINATION lib/static
      FILE_SET HEADERS
        DESTINATION "include/${PROJECT_INCLUDE_PREFIX}"
    )

    if( has_files_to_install )
      install( FILES ${bin_files_installing}
        DESTINATION bin
      )
    endif()

    if( has_additional_installs )
      message( "${PROJECT_NAME} additional installs: ${additional_installs}" )
      install( TARGETS ${additional_installs}
        EXPORT ${PROJECT_NAME}Targets
        RUNTIME 
          DESTINATION bin
        LIBRARY
          DESTINATION lib
        ARCHIVE
          DESTINATION lib
          # DESTINATION lib/static
        FILE_SET HEADERS
          DESTINATION "include"
        INCLUDES DESTINATION
          ${additional_relative_dep_paths}
      )
    endif()

    install( DIRECTORY "${MY_RUNTIME_OUTPUT_DIR}/resources"
      DESTINATION bin
    )
  
    install( EXPORT ${PROJECT_NAME}Targets
      FILE ${PROJECT_NAME}Targets.cmake
      NAMESPACE "${PROJECT_NAME}::"
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
      NAMESPACE "${PROJECT_NAME}::"
    )
  else()
    message( FATAL_ERROR "ERROR: This project (${PROJECT_NAME}) doesn't install any targets." )
  endif()
endfunction()

macro( initialize_install_list )
  set( MY_ADDITIONAL_INSTALL_TARGETS "" )
  set( MY_ADDITIONAL_RELATIVE_DEP_PATHS "" )
endmacro()

macro( clean_install_list )
  clean_list( "${MY_ADDITIONAL_INSTALL_TARGETS}" MY_ADDITIONAL_INSTALL_TARGETS )
  clean_list( "${MY_ADDITIONAL_RELATIVE_DEP_PATHS}" MY_ADDITIONAL_RELATIVE_DEP_PATHS )
endmacro()

macro( add_to_install_list
  target_name
  relative_dep_path
)
  get_target_property( unaliased_lib_name ${target_name} ALIASED_TARGET )

  if( NOT unaliased_lib_name )
    set( unaliased_lib_name ${target_name} )
  endif()

  set( MY_ADDITIONAL_INSTALL_TARGETS "${MY_ADDITIONAL_INSTALL_TARGETS}" ${unaliased_lib_name} )
  set( MY_ADDITIONAL_RELATIVE_DEP_PATHS "${MY_ADDITIONAL_RELATIVE_DEP_PATHS}" "${relative_dep_path}" )
endmacro()

macro( raise_install_list )
  set( LATEST_INSTALL_LIST "${MY_ADDITIONAL_INSTALL_TARGETS}" PARENT_SCOPE )
  set( LATEST_RELATIVE_DEP_PATHS "${MY_ADDITIONAL_RELATIVE_DEP_PATHS}" PARENT_SCOPE )
endmacro()

macro( initialize_target_list )
  set( MY_INSTALLABLE_TARGETS "" )
endmacro()

macro( clean_target_list )
  clean_list( "${MY_INSTALLABLE_TARGETS}" MY_INSTALLABLE_TARGETS )
endmacro()

macro( add_to_target_list
  target_name
)
  set( MY_INSTALLABLE_TARGETS "${MY_INSTALLABLE_TARGETS}" "${target_name}" )
endmacro()

macro( raise_target_list )
  set( LATEST_SUBPROJECT_TARGET_LIST "${MY_INSTALLABLE_TARGETS}" PARENT_SCOPE )
endmacro()

macro( initialize_needed_bin_files_list )
  set( MY_NEEDED_BIN_FILES "" )
endmacro()

macro( clean_needed_bin_files_list )
  clean_list( "${MY_NEEDED_BIN_FILES}" MY_NEEDED_BIN_FILES )
endmacro()

macro( add_to_needed_bin_files_list
  needed_file
)
  set( MY_NEEDED_BIN_FILES "${MY_NEEDED_BIN_FILES}" "${needed_file}" )
endmacro()

macro( raise_needed_bin_files_list)
  set( LATEST_SUBPROJECT_NEEDED_BIN_FILES_LIST "${MY_NEEDED_BIN_FILES}" PARENT_SCOPE )
endmacro()

function( configure_subproject
  subproject_path
)
  add_subdirectory( "${subproject_path}" )

  if( NOT "${LATEST_SUBPROJECT_TARGET_LIST}" STREQUAL "" )
    if( "${MY_INSTALLABLE_TARGETS}" STREQUAL "" )
      set( combined_list "${LATEST_SUBPROJECT_TARGET_LIST}" )
    else()
      set( combined_list "${MY_INSTALLABLE_TARGETS}" "${LATEST_SUBPROJECT_TARGET_LIST}" )
    endif()

    set( MY_INSTALLABLE_TARGETS "${combined_list}" PARENT_SCOPE )
  endif()

  if( NOT "${LATEST_SUBPROJECT_NEEDED_BIN_FILES_LIST}" STREQUAL "" )
    if( "${MY_NEEDED_BIN_FILES}" STREQUAL "" )
      set( combined_list "${LATEST_SUBPROJECT_NEEDED_BIN_FILES_LIST}" )
    else()
      set( combined_list "${MY_NEEDED_BIN_FILES}" "${LATEST_SUBPROJECT_NEEDED_BIN_FILES_LIST}" )
    endif()

    set( MY_NEEDED_BIN_FILES "${combined_list}" PARENT_SCOPE )
  endif()

  if( NOT "${LATEST_INSTALL_LIST}" STREQUAL "" )
    if( "${MY_ADDITIONAL_INSTALL_TARGETS}" STREQUAL "" )
      set( combined_list "${LATEST_INSTALL_LIST}" )
    else()
      set( combined_list "${MY_ADDITIONAL_INSTALL_TARGETS}" "${LATEST_INSTALL_LIST}" )
    endif()

    set( MY_ADDITIONAL_INSTALL_TARGETS "${combined_list}" PARENT_SCOPE )
  endif()

  if( NOT "${LATEST_RELATIVE_DEP_PATHS}" STREQUAL "" )
    if( "${MY_ADDITIONAL_RELATIVE_DEP_PATHS}" STREQUAL "" )
      set( combined_list "${LATEST_RELATIVE_DEP_PATHS}" )
    else()
      set( combined_list "${MY_ADDITIONAL_RELATIVE_DEP_PATHS}" "${LATEST_RELATIVE_DEP_PATHS}" )
    endif()

    set( MY_ADDITIONAL_RELATIVE_DEP_PATHS "${combined_list}" PARENT_SCOPE )
  endif()
endfunction()
"#;

const GCMAKE_CPACK_CONFIGURE_TEXT: &'static str = r#"
# Should only be called from the root project, and only from the toplevel project being built.
# if( "${CMAKE_SOURCE_DIR}" STREQUAL "${CMAKE_CURRENT_SOURCE_DIR}" )
function( gcmake_configure_cpack
  vendor_name
)
  include( ProcessorCount )
  ProcessorCount( num_cpu_cores )

  if( num_cpu_cores EQUAL 0 )
    set( num_cpu_cores 1 )
  endif()

  set( CMAKE_NUM_PACKAGER_THREADS ${num_cpu_cores} CACHE STRING "Number of threads to use for CPack jobs" )

  set( CPACK_THREADS ${CMAKE_NUM_PACKAGER_THREADS} )
  set( CPACK_ARCHIVE_THREADS ${CMAKE_NUM_PACKAGER_THREADS} )

  # Currently don't support Apple because I have no way to test it.
  if( CURRENT_SYSTEM_IS_WINDOWS )
    option( CPACK_WIX_ENABLED "Generate installer using WiX" ON )
    option( CPACK_NSIS_ENABLED "Generate installer using NSIS" OFF )

    set( CPACK_GENERATOR "7Z" "ZIP" )
    set( CPACK_SOURCE_GENERATOR "ZIP" "7Z" )

    if( CPACK_WIX_ENABLED )
      list( APPEND CPACK_GENERATOR "WIX" )
    endif()

    if( CPACK_NSIS_ENABLED )
      list( APPEND CPACK_GENERATOR "NSIS" )
    endif()
  elseif( CURRENT_SYSTEM_IS_LINUX )
    option( CPACK_DEB_ENABLED "Generate DEB installer" OFF )
    option( CPACK_RPM_ENABLED "Generate RPM installer" OFF )
    option( CPACK_FreeBSD_ENABLED "Generate FreeBSD installer" OFF )

    set( CPACK_GENERATOR "TGZ" "TXZ" )

    if( CPACK_DEB_ENABLED )
      list( APPEND CPACK_GENERATOR "DEB" )
    endif()

    if( CPACK_RPM_ENABLED )
      list( APPEND CPACK_GENERATOR "RPM" )
    endif()

    if( CPACK_FreeBSD_ENABLED )
      list( APPEND CPACK_GENERATOR "FreeBSD" )
    endif()

    set( CPACK_SOURCE_GENERATOR "TGZ" "TXZ" "ZIP" "7Z" )
  endif()

  set( CPACK_PACKAGE_VENDOR "${vendor_name}" )
  set( CPACK_SOURCE_IGNORE_FILES "/\\\\.git/" "/\\\\.vscode/" "/build/" "/dep/" "/__pycache__/" "/\\\\.mypy_cache/" )

  include( CPack )
endfunction()
"#;