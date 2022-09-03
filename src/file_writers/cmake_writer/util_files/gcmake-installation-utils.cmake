function( configure_installation
  project_component_name_var
)
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
  list( LENGTH MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE has_generated_export_headers )
  list( LENGTH MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE has_generated_export_install_headers )

  if( NOT has_generated_export_headers EQUAL has_generated_export_install_headers )
    message( FATAL_ERROR "Number of generated export headers for build interface (${has_generated_export_headers}) doesn't match the number of generated export headers for install interface (${has_generated_export_install_headers})" )
  endif()

  # NOTE: Don't make this all CAPS. It doesn't play nice when creating a multi-component NSIS
  # installer. Either that or the default "Unspecified" component doesn't play nice.
  set( project_component_name ProjectOutputs )
  set( ${project_component_name_var} ${project_component_name} PARENT_SCOPE )

  if( NOT has_targets_to_install )
    message( FATAL_ERROR "ERROR: This project (${PROJECT_NAME}) doesn't install any targets." )
  endif()

  install( TARGETS ${targets_installing}
    EXPORT ${PROJECT_NAME}Targets
    RUNTIME 
      DESTINATION bin
    LIBRARY
      DESTINATION lib
    ARCHIVE
      DESTINATION lib
      # DESTINATION lib/static
    COMPONENT ${project_component_name}
    FILE_SET HEADERS
      DESTINATION "include/${PROJECT_INCLUDE_PREFIX}"
  )

  if( has_files_to_install )
    install( FILES ${bin_files_installing}
      DESTINATION bin
    )
  endif()

  if( has_generated_export_headers )
    foreach( generated_file installed_file IN ZIP_LISTS MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE )
      cmake_path( REMOVE_FILENAME installed_file OUTPUT_VARIABLE installed_file_dir )
      install( FILES ${generated_file}
        DESTINATION "${installed_file_dir}"
        COMPONENT ${project_component_name}
      )
    endforeach()
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
    COMPONENT ${project_component_name}
  )

  install( EXPORT ${PROJECT_NAME}Targets
    FILE ${PROJECT_NAME}Targets.cmake
    NAMESPACE "${PROJECT_NAME}::"
    DESTINATION "lib/cmake/${PROJECT_NAME}"
    COMPONENT ${project_component_name}
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
    COMPONENT ${project_component_name}
  )

  export( EXPORT ${PROJECT_NAME}Targets
    FILE "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Targets.cmake"
    NAMESPACE "${PROJECT_NAME}::"
  )
endfunction()

# NOTE: Assumes include(GenerateExportHeader) has already been called.
function( generate_and_install_export_header
  target_name
)
  set( the_export_header_file "${CMAKE_BINARY_DIR}/generated_export_headers/${PROJECT_INCLUDE_PREFIX}/${target_name}_export.h" )
  set( installed_header_location "include/${PROJECT_INCLUDE_PREFIX}/${target_name}_export.h" )

  generate_export_header( ${target_name}
    EXPORT_FILE_NAME "${the_export_header_file}"
  )

  target_include_directories( inner-lib
    PUBLIC
      "$<BUILD_INTERFACE:${CMAKE_BINARY_DIR}/generated_export_headers>"
  )

  target_sources( ${target_name}
    PUBLIC
      "$<BUILD_INTERFACE:${the_export_header_file}>"
      "$<INSTALL_INTERFACE:${installed_header_location}>"
  )

  add_to_generated_export_headers_list_parent_scope(
    "${the_export_header_file}"
    "${installed_header_location}"
  )
endfunction()

# ================================================================================
# Generated export headers list: Auto-generated header files containing the
# "Export macros" used when making a DLL on MSVC. These macros will be used to
# prefix everything in ANY LIBRARY'S header files which is part of the library's
# public interface.
# ================================================================================
macro( initialize_generated_export_headers_list )
  set( MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE )
  set( MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE )
endmacro()

macro( clean_generated_export_headers_list )
  clean_list( "${MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE}" MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE )
  clean_list( "${MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE}" MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE )
endmacro()

macro( add_to_generated_export_headers_list_parent_scope
  build_interface_file
  install_interface_file
)
  set( MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE "${MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE}" ${build_interface_file} PARENT_SCOPE )
  set( MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE "${MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE}" ${install_interface_file} PARENT_SCOPE )
endmacro()

macro( raise_generated_export_headers_list )
  set( LATEST_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE_LIST "${MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE}" PARENT_SCOPE )
  set( LATEST_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE_LIST "${MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE}" PARENT_SCOPE )
endmacro()

# ================================================================================
# TODO: This name is misleading. Change the name to 'additional_dependency_install_targets'
# or something similar
# 
# Install list: Non-GCMake dependency targets which must be listed in our install tree
# because they are depended on by one of our project's library outputs.
# ================================================================================
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

# ================================================================================
# Target list: These are your project outputs
# ================================================================================
macro( initialize_target_list )
  set( MY_INSTALLABLE_TARGETS "" )
endmacro()

macro( clean_target_list )
  clean_list( "${MY_INSTALLABLE_TARGETS}" MY_INSTALLABLE_TARGETS )
endmacro()

macro( add_to_target_installation_list
  target_name
)
  set( MY_INSTALLABLE_TARGETS "${MY_INSTALLABLE_TARGETS}" "${target_name}" )
endmacro()

macro( raise_target_list )
  set( LATEST_SUBPROJECT_TARGET_LIST "${MY_INSTALLABLE_TARGETS}" PARENT_SCOPE )
endmacro()

# ================================================================================
# Needed bin files: Any needed DLLs which are retrieved from outside the project,
# but must be distributed with the project (such as SDL2.dll or the WxWidgets DLLs).
# ================================================================================
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

  if( NOT "${LATEST_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE_LIST}" STREQUAL "" )
    if( "${MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE}" STREQUAL "" )
      set( combined_list "${LATEST_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE_LIST}" )
    else()
      set( combined_list "${MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE}" "${LATEST_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE_LIST}" )
    endif()

    set( MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE "${combined_list}" PARENT_SCOPE )
  endif()

  if( NOT "${LATEST_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE_LIST}" STREQUAL "" )
    if( "${MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE}" STREQUAL "" )
      set( combined_list "${LATEST_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE_LIST}" )
    else()
      set( combined_list "${MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE}" "${LATEST_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE_LIST}" )
    endif()

    set( MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE "${combined_list}" PARENT_SCOPE )
  endif()
endfunction()
