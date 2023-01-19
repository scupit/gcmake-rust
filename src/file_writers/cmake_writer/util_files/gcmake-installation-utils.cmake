if( NOT HAS_GCMAKE_INSTALL_DEFAULTS_CONFIG_RUN )
  option( GCMAKE_INSTALL "When ON, CMake will create both install and packaging configurations for the project tree" ON )

  set( HAS_GCMAKE_INSTALL_DEFAULTS_CONFIG_RUN TRUE )
endif()

function( configure_installation
  project_component_name_var
)
  set( additional_installs ${MY_ADDITIONAL_DEPENDENCY_INSTALL_TARGETS} )
  list( REMOVE_DUPLICATES additional_installs )

  set( additional_relative_dep_paths ${MY_ADDITIONAL_RELATIVE_DEP_PATHS} )
  list( TRANSFORM additional_relative_dep_paths PREPEND "${CMAKE_INSTALL_INCLUDEDIR}/" )
  list( REMOVE_DUPLICATES additional_relative_dep_paths )

  list( LENGTH MY_INSTALLABLE_TARGETS has_targets_to_install )
  list( LENGTH MY_NEEDED_BIN_FILES has_files_to_install )
  list( LENGTH additional_installs has_additional_installs )
  list( LENGTH MY_MINIMAL_INSTALLS has_minimal_installs )
  list( LENGTH MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE has_generated_export_headers )
  list( LENGTH MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE has_generated_export_install_headers )
  list( LENGTH MY_CUSTOM_FIND_MODULES has_custom_find_modules )

  if( NOT has_generated_export_headers EQUAL has_generated_export_install_headers )
    message( FATAL_ERROR "Number of generated export headers for build interface (${has_generated_export_headers}) doesn't match the number of generated export headers for install interface (${has_generated_export_install_headers})" )
  endif()

  # NOTE: Don't make this all CAPS. It doesn't play nice when creating a multi-component NSIS
  # installer. Either that or the default "Unspecified" component doesn't play nice.
  set( project_component_name ProjectOutputs )
  set( ${project_component_name_var} ${project_component_name} PARENT_SCOPE )

  # It's fine to not install any targets if this is a dependency project. If it's the main project,
  # we should install at least one. I think this can happen when a GCMake dependency target is
  # linked to a pre-build script or test executable but not an output target.
  if( CMAKE_SOURCE_DIR STREQUAL TOPLEVEL_PROJECT_DIR AND NOT (has_targets_to_install OR has_minimal_installs) )
    message( FATAL_ERROR "ERROR: This project (${PROJECT_NAME}) doesn't install any targets. It's likely that no project targets are being built with the current configuration settings." )
  endif()

  foreach( project_output_to_install IN LISTS MY_INSTALLABLE_TARGETS )
    get_target_property( target_type ${project_output_to_install} TYPE )
    gcmake_unaliased_target_name( ${project_output_to_install} actual_output_name )

    if( target_type STREQUAL "EXECUTABLE" )
      install( TARGETS ${actual_output_name}
        EXPORT ${PROJECT_NAME}Targets
        RUNTIME 
          DESTINATION "${CMAKE_INSTALL_BINDIR}"
          PERMISSIONS
            OWNER_READ OWNER_WRITE OWNER_EXECUTE 
            GROUP_READ GROUP_EXECUTE
            WORLD_READ
        LIBRARY
          DESTINATION "${CMAKE_INSTALL_LIBDIR}"
        ARCHIVE
          DESTINATION "${CMAKE_INSTALL_LIBDIR}"
        COMPONENT ${project_component_name}
      )
    else()
      install( TARGETS ${actual_output_name}
        EXPORT ${PROJECT_NAME}Targets
        RUNTIME 
          DESTINATION "${CMAKE_INSTALL_BINDIR}"
        LIBRARY
          DESTINATION "${CMAKE_INSTALL_LIBDIR}"
        ARCHIVE
          DESTINATION "${CMAKE_INSTALL_LIBDIR}"
        COMPONENT ${project_component_name}
        FILE_SET HEADERS
          DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}/${PROJECT_INCLUDE_PREFIX}"
      )
    endif()
  endforeach()

  foreach( minimal_target_to_install IN LISTS MY_MINIMAL_INSTALLS )
    # cppfront::artifacts (cppfront_artifacts) is always private linked, therefore it should never
    # be installed. The target may be imported if EMBED_CPPFRONT is set to OFF, so we manually filter
    # it out here knowing that doing so won't cause issues.
    if( "${minimal_target_to_install}" STREQUAL "cppfront_artifacts" )
      continue()
    endif()

    # NOTE: Due to limitations with how CMake installs work, a "minimal" install 
    # does the same thing as just installing additional targets. I'd like a way
    # to install only the components of a target needed at runtime, but that doesn't
    # seem possible at the moment. 
    install( TARGETS ${minimal_target_to_install}
      EXPORT ${PROJECT_NAME}Targets
      RUNTIME 
        DESTINATION "${CMAKE_INSTALL_BINDIR}"
      # If we omit the LIBRARY and ARCHIVE sections, the fmt::fmt install is unable to find certain headers?
      # What?? 
      LIBRARY
        DESTINATION "${DEPENDENCY_INSTALL_LIBDIR}"
      ARCHIVE
        DESTINATION "${DEPENDENCY_INSTALL_LIBDIR}"
      COMPONENT ${project_component_name}
      # Apparently targets which have INTERFACE or PUBLIC file sets can't be installed
      # without them even if the target's file_set isn't ever needed. That's really annoying.
      # We'll install these to the _unused directory so we can at least see they aren't needed.
      FILE_SET HEADERS
        DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}/_unused"
      INCLUDES DESTINATION
        # TODO: I might need to separate this into its own variable. I'll leave it for now though, since it
        # isn't causing issues.
        ${additional_relative_dep_paths}
    )
  endforeach()

  if( has_files_to_install )
    install( FILES ${MY_NEEDED_BIN_FILES}
      DESTINATION ${CMAKE_INSTALL_BINDIR}
      COMPONENT ${project_component_name}
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
    # message( "${PROJECT_NAME} additional installs: ${additional_installs}" )
    install( TARGETS ${additional_installs}
      EXPORT ${PROJECT_NAME}Targets
      RUNTIME 
        DESTINATION "${CMAKE_INSTALL_BINDIR}"
      LIBRARY
        # Since additional installs should only be used for dependencies, 
        DESTINATION "${DEPENDENCY_INSTALL_LIBDIR}"
      ARCHIVE
        DESTINATION "${DEPENDENCY_INSTALL_LIBDIR}"
      FILE_SET HEADERS
        DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}"
      INCLUDES DESTINATION
        ${additional_relative_dep_paths}
    )
  endif()

  if( has_custom_find_modules )
    install( FILES ${MY_CUSTOM_FIND_MODULES}
      DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}/modules"
      COMPONENT ${project_component_name}
    )
  endif()

  if( EXISTS "${MY_RUNTIME_OUTPUT_DIR}/resources" )
    install( DIRECTORY "${MY_RUNTIME_OUTPUT_DIR}/resources"
      DESTINATION "${CMAKE_INSTALL_BINDIR}"
      COMPONENT ${project_component_name}
    )
  endif()

  install( EXPORT ${PROJECT_NAME}Targets
    FILE ${PROJECT_NAME}Targets.cmake
    # PROJECT_NAME is the same as LOCAL_TOPLEVEL_PROJECT_NAME here since installations
    # are only invoked from the root of a project.
    NAMESPACE "${PROJECT_NAME}::"
    DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}"
    COMPONENT ${project_component_name}
  )

  include( CMakePackageConfigHelpers )

  configure_package_config_file( "${CMAKE_CURRENT_SOURCE_DIR}/Config.cmake.in"
    "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake"
    INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake"
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
    DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}"
    COMPONENT ${project_component_name}
  )

  if( ${PROJECT_NAME}_BUILD_DOCS )
    install( DIRECTORY
      # PROJECT_DOCS_OUTPUT_DIR is set in the write_documentation_generation(...)
      # function inside cmakelists_writer.rs.
      "${PROJECT_DOCS_OUTPUT_DIR}"
      DESTINATION "${CMAKE_INSTALL_DOCDIR}"
    )
  endif()

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
  set( installed_header_location "${CMAKE_INSTALL_INCLUDEDIR}/${PROJECT_INCLUDE_PREFIX}/${target_name}_export.h" )

  generate_export_header( ${target_name}
    EXPORT_FILE_NAME "${the_export_header_file}"
  )

  target_include_directories( ${target_name}
    PUBLIC
      "$<BUILD_INTERFACE:${CMAKE_BINARY_DIR}/generated_export_headers>"
  )

  target_sources( ${target_name}
    PUBLIC
      "$<BUILD_INTERFACE:${the_export_header_file}>"
      "$<INSTALL_INTERFACE:${installed_header_location}>"
  )

  add_to_generated_export_headers_list(
    "${the_export_header_file}"
    "${installed_header_location}"
    TRUE
  )
endfunction()

# Sets the install mode for a GCMake library
function( mark_gcmake_target_usage
  lib_base_name
  needed_install_mode
)
  set( SHOULD_CHANGE FALSE )

  if( ${needed_install_mode} STREQUAL "FULL" )
    set( SHOULD_CHANGE TRUE )
  elseif( needed_install_mode STREQUAL "MINIMAL" )
    if( NOT "${TARGET_${lib_base_name}_INSTALL_MODE}" STREQUAL "FULL" )
      set( SHOULD_CHANGE TRUE )
    endif()
  else()
    message( FATAL_ERROR "Invalid gcmake usage needed_install_mode \"${needed_install_mode}\" given. Must be either FULL or MINIMAL." )
  endif()

  if( SHOULD_CHANGE )
    set( TARGET_${lib_base_name}_INSTALL_MODE ${needed_install_mode} PARENT_SCOPE )
  endif()
endfunction()

function( mark_gcmake_project_usage
  project_base_name
  needed_install_mode
)
  set( SHOULD_CHANGE FALSE )

  if( needed_install_mode STREQUAL "FULL" )
    set( SHOULD_CHANGE TRUE )
  elseif( needed_install_mode STREQUAL "MINIMAL" )
    if( NOT "${PROJECT_${project_base_name}_INSTALL_MODE}" STREQUAL "FULL" )
      set( SHOULD_CHANGE TRUE )
    endif()
  else()
    message( FATAL_ERROR "Invalid gcmake usage needed_install_mode \"${needed_install_mode}\" given. Must be either FULL or MINIMAL." )
  endif()

  if( SHOULD_CHANGE )
    set( PROJECT_${project_base_name}_INSTALL_MODE ${needed_install_mode} PARENT_SCOPE )
  endif()
endfunction()

macro( initialize_install_mode )
  if( TOPLEVEL_PROJECT_DIR STREQUAL CMAKE_SOURCE_DIR )
    set( VALID_GCMAKE_INSTALL_MODES "NORMAL" "EXE_ONLY" "LIB_ONLY" )
    set( GCMAKE_INSTALL_MODE "NORMAL" CACHE STRING "Build/Installation mode for the project. \"NORMAL\" means both executables and libraries will be built installed with minimum dependencies where possible." )
    set_property( CACHE GCMAKE_INSTALL_MODE PROPERTY STRINGS ${VALID_GCMAKE_INSTALL_MODES} )

    if( NOT GCMAKE_INSTALL_MODE IN_LIST VALID_GCMAKE_INSTALL_MODES )
      message( FATAL_ERROR "Invalid GCMAKE_INSTALL_MODE \"${GCMAKE_INSTALL_MODE}\" given. Must be one of: ${VALID_GCMAKE_INSTALL_MODES}" )
    endif()
  endif()
endmacro()

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

macro( add_to_generated_export_headers_list
  build_interface_file
  install_interface_file
  should_set_in_parent_scope
)
  list( APPEND MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE "${build_interface_file}" )
  list( APPEND MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE "${install_interface_file}" )

  if( ${should_set_in_parent_scope} )
    set( MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE ${MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE} PARENT_SCOPE )
    set( MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE ${MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE} PARENT_SCOPE )
  endif()
endmacro()

macro( raise_generated_export_headers_list )
  set( LATEST_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE_LIST ${MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE} PARENT_SCOPE )
  set( LATEST_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE_LIST ${MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE} PARENT_SCOPE )
endmacro()

# ================================================================================
# Additional Dependency install list: Non-GCMake dependency targets which must be
#   listed in our install tree because they are depended on by one of our project's
#   library outputs.
# ================================================================================
macro( initialize_additional_dependency_install_list )
  set( MY_ADDITIONAL_DEPENDENCY_INSTALL_TARGETS )
  # NOTE: Make sure MY_ADDITIONAL_RELATIVE_DEP_PATHS is the same as the one used by minimal installs.
  set( MY_ADDITIONAL_RELATIVE_DEP_PATHS )
endmacro()

macro( add_to_additional_dependency_install_list
  target_name
  relative_dep_path
)
  gcmake_unaliased_target_name( ${target_name} unaliased_target_name )
  list( APPEND MY_ADDITIONAL_DEPENDENCY_INSTALL_TARGETS ${unaliased_target_name} )
  list( APPEND MY_ADDITIONAL_RELATIVE_DEP_PATHS "${relative_dep_path}" )
endmacro()

macro( raise_additional_dependency_install_list )
  set( LATEST_ADDITIONAL_DEPENDENCY_INSTALL_LIST ${MY_ADDITIONAL_DEPENDENCY_INSTALL_TARGETS} PARENT_SCOPE )
  set( LATEST_RELATIVE_DEP_PATHS ${MY_ADDITIONAL_RELATIVE_DEP_PATHS} PARENT_SCOPE )
endmacro()

# ================================================================================
# Target list: These are your project outputs
# ================================================================================
macro( initialize_target_list )
  set( MY_INSTALLABLE_TARGETS )
endmacro()

macro( add_to_target_installation_list
  target_name
)
  gcmake_unaliased_target_name( ${target_name} unaliased_target_name )
  list( APPEND MY_INSTALLABLE_TARGETS ${unaliased_target_name} )
endmacro()

macro( raise_target_list )
  set( LATEST_SUBPROJECT_TARGET_LIST ${MY_INSTALLABLE_TARGETS} PARENT_SCOPE )
endmacro()

# ================================================================================
# Debian package names which the project depends on.
# ================================================================================
macro( initialize_deb_list )
  set( MY_NEEDED_DEB_PACKAGES )
endmacro()

macro( add_to_deb_list
  deb_package_name
)
  list( APPEND MY_NEEDED_DEB_PACKAGES "${deb_package_name}" )
endmacro()

macro( raise_deb_list )
  set( LATEST_NEEDED_DEB_PACKAGE_LIST MY_NEEDED_DEB_PACKAGES PARENT_SCOPE )
endmacro()

# ================================================================================
# Minimal installs: These are libraries which are used by your outputs, but not
# needed further. They are essentially libraries which could be DLLs, and should
# only have the DLL runtime component installed.
# ================================================================================
macro( initialize_minimal_installs )
  set( MY_MINIMAL_INSTALLS )
endmacro()

macro( add_to_minimal_installs
  target_name
  relative_dep_path
)
  gcmake_unaliased_target_name( ${target_name} unaliased_target_name )

  list( APPEND MY_MINIMAL_INSTALLS ${unaliased_target_name} )
  list( APPEND MY_ADDITIONAL_RELATIVE_DEP_PATHS "${relative_dep_path}" )
endmacro()

macro( raise_minimal_installs )
  set( LATEST_MINIMAL_INSTALLS_LIST ${MY_MINIMAL_INSTALLS} PARENT_SCOPE )
endmacro()

# ================================================================================
# Needed bin files: Any needed DLLs which are retrieved from outside the project,
# but must be distributed with the project (such as SDL2.dll or the WxWidgets DLLs).
# ================================================================================
macro( initialize_needed_bin_files_list )
  set( MY_NEEDED_BIN_FILES )
endmacro()

macro( add_to_needed_bin_files_list
  needed_file
)
  list( APPEND MY_NEEDED_BIN_FILES "${needed_file}" )
endmacro()

macro( raise_needed_bin_files_list)
  set( LATEST_SUBPROJECT_NEEDED_BIN_FILES_LIST ${MY_NEEDED_BIN_FILES} PARENT_SCOPE )
endmacro()

macro( initialize_mingw_dll_install_options )
  if( NOT USING_MINGW )
    message( FATAL_ERROR "Tried to initialize MinGW dll install options while not using a MinGW compiler. This should only be called when using MinGW." )
  endif()

  set( _MINGW_DLL_NAME
    "LIBSTDCXX"
    "SEH" 
    "LIBWINPTHREAD"
    "LIBATOMIC"
  )

  set( _CORRESPONDING_FILES
    "libstdc++-6.dll"
    "libgcc_s_seh-1.dll"
    "libwinpthread-1.dll"
    "libatomic-1.dll"
  )

  cmake_path( GET CMAKE_C_COMPILER PARENT_PATH MINGW_DLL_DIR )

  foreach( dll_name matching_file IN ZIP_LISTS _MINGW_DLL_NAME _CORRESPONDING_FILES )
    set( dll_file_var GCMAKE_FILE_MINGW_${dll_name}_DLL )
    find_file( ${dll_file_var}
      NAMES "${matching_file}"
      PATHS "${MINGW_DLL_DIR}"
      NO_DEFAULT_PATH
      NO_PACKAGE_ROOT_PATH
      NO_CMAKE_PATH
      NO_CMAKE_ENVIRONMENT_PATH
      NO_SYSTEM_ENVIRONMENT_PATH
      NO_CMAKE_SYSTEM_PATH
      NO_CMAKE_FIND_ROOT_PATH
    )

    if( NOT ${dll_file_var} )
      message( FATAL_ERROR "Unable to find MinGW's ${matching_file}." )
    endif()

    # These currently default to ON so that the install will work out of the box
    # on other users' machines. Eventually I'd like to be able to determine which
    # of these are actually needed by the installed executables and DLLS. However,
    # that would be more effort than it's worth at the moment.
    option( GCMAKE_MINGW_INSTALL_${dll_name} "When ON, ${matching_file} is copied to the build directory and installed with the project." ON )

    if( GCMAKE_MINGW_INSTALL_${dll_name} )
      add_to_needed_bin_files_list( "${${dll_file_var}}" )
    endif()
  endforeach()
endmacro()

# ================================================================================
# Custom Find Modules: Any custom Find<LibName>.cmake files which need to be
# installed with the project.
# ================================================================================
macro( initialize_custom_find_modules_list )
  set( MY_CUSTOM_FIND_MODULES )
endmacro()

macro( add_to_custom_find_modules_list
  dep_name
)
  list( APPEND MY_CUSTOM_FIND_MODULES "${TOPLEVEL_PROJECT_DIR}/cmake/modules/Find${dep_name}.cmake" )
endmacro()

macro( raise_custom_find_modules_list )
  set( LATEST_SUBPROJECT_CUSTOM_FIND_MODULES_LIST ${MY_CUSTOM_FIND_MODULES} PARENT_SCOPE )
endmacro()

# ================================================================================
# Documentable files: Any header or source files used by your project outputs
# which should be directly used by a documentation tool. This includes all normal
# .c/.cpp and .h/.hpp files, as well as .cpp files generated by cppfront.
# ================================================================================

macro( gcmake_init_documentable_files_list )
  set( DOCUMENTABLE_FILES )
endmacro()

macro( gcmake_add_to_documentable_files_list files_list_var )
  list( APPEND DOCUMENTABLE_FILES ${${files_list_var}} )
endmacro()

macro( gcmake_raise_documentable_files_list )
  set( LATEST_DOCUMENTABLE_FILES ${DOCUMENTABLE_FILES} PARENT_SCOPE )
endmacro()

macro( _propagate_subproject_var
  var_from_subproject
  matching_current_scope_var
)
  set( combined_list ${${matching_current_scope_var}})
  list( APPEND combined_list ${${var_from_subproject}} )
  set( ${matching_current_scope_var} ${combined_list} PARENT_SCOPE )
endmacro()

function( gcmake_configure_subproject
  subproject_path
)
  add_subdirectory( "${subproject_path}" )

  _propagate_subproject_var( LATEST_SUBPROJECT_TARGET_LIST                          MY_INSTALLABLE_TARGETS )
  _propagate_subproject_var( LATEST_SUBPROJECT_NEEDED_BIN_FILES_LIST                MY_NEEDED_BIN_FILES )
  _propagate_subproject_var( LATEST_SUBPROJECT_CUSTOM_FIND_MODULES_LIST             MY_CUSTOM_FIND_MODULES )
  _propagate_subproject_var( LATEST_ADDITIONAL_DEPENDENCY_INSTALL_LIST              MY_ADDITIONAL_DEPENDENCY_INSTALL_TARGETS )
  _propagate_subproject_var( LATEST_RELATIVE_DEP_PATHS                              MY_ADDITIONAL_RELATIVE_DEP_PATHS )
  _propagate_subproject_var( LATEST_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE_LIST   MY_GENERATED_EXPORT_HEADERS_BUILD_INTERFACE )
  _propagate_subproject_var( LATEST_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE_LIST MY_GENERATED_EXPORT_HEADERS_INSTALL_INTERFACE )
  _propagate_subproject_var( LATEST_MINIMAL_INSTALLS_LIST                           MY_MINIMAL_INSTALLS )
  _propagate_subproject_var( LATEST_NEEDED_DEB_PACKAGE_LIST                         MY_NEEDED_DEB_PACKAGES )
  _propagate_subproject_var( LATEST_DOCUMENTABLE_FILES                              DOCUMENTABLE_FILES)
endfunction()
