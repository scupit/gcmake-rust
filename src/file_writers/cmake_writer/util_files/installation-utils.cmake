function( configure_installation )
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

macro( add_to_target_installation_list
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
