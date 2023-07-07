# Empties the default per-configuration flags list created by CMake for the
# duration of the scope. We don't need the default flags CMake sets because we
# configure them in cmake_data.yaml instead.
# This should only be called after a project's dependencies have been loaded so we don't
# interfere with existing behavior in non-GCMake projects.
macro( _gcmake_clear_scope_default_compiler_flags )
  foreach( build_type IN LISTS GCMAKE_ALL_VALID_BUILD_CONFIGS_UPPER )
    set( CMAKE_CUDA_FLAGS_${build_type} "" )
    set( CMAKE_CXX_FLAGS_${build_type} "" )
    set( CMAKE_C_FLAGS_${build_type} "" )
  endforeach()
endmacro()

function( exe_add_lib_relative_install_rpath
  exe_target
)
  if( NOT TARGET_SYSTEM_IS_WINDOWS )
    set( POSSIBLE_LIB_DIRS "${CMAKE_INSTALL_LIBDIR}" "${DEPENDENCY_INSTALL_LIBDIR}" )
    foreach( LIB_DIR IN LISTS POSSIBLE_LIB_DIRS )
      set_property(
        TARGET ${exe_target}
        APPEND PROPERTY
          INSTALL_RPATH "\${ORIGIN}/../${LIB_DIR}"
      )
    endforeach()
  endif()
endfunction()

function( shared_lib_add_relative_install_rpath
  shared_lib_target
)
  if( NOT TARGET_SYSTEM_IS_WINDOWS )
    string( REGEX REPLACE
      "${CMAKE_INSTALL_LIBDIR}/?"
      ""
      RELATIVE_DEP_INSTALL_DIR
      "${DEPENDENCY_INSTALL_LIBDIR}"
    )
    set( INITIAL_RPATHS "\${ORIGIN}/${RELATIVE_DEP_INSTALL_DIR}" "\${ORIGIN}" )

    foreach( NEEDED_RPATH IN LISTS INITIAL_RPATHS )
      set_property(
        TARGET ${shared_lib_target}
        APPEND PROPERTY
          INSTALL_RPATH "${NEEDED_RPATH}"
      )
    endforeach()
  endif()
endfunction()

# TODO: Refactor these two into one delegator function
function( gcmake_get_without_given_prefix
  all_files_var
  prefix_str
  out_var
)
  list( TRANSFORM ${all_files_var}
    REPLACE "${prefix_str}/?" ""
    OUTPUT_VARIABLE with_removed_prefix
  )

  list( TRANSFORM with_removed_prefix
    REPLACE "^\\./" ""
    OUTPUT_VARIABLE with_removed_prefix
  )

  set( ${out_var} ${with_removed_prefix} PARENT_SCOPE )
endfunction()

function( gcmake_get_without_toplevel_dir_prefix
  all_files_var
  out_var
)
  gcmake_get_without_given_prefix( ${all_files_var} "${TOPLEVEL_PROJECT_DIR}" with_removed_prefix )
  set( ${out_var} ${with_removed_prefix} PARENT_SCOPE )
endfunction()

function( gcmake_get_without_current_source_dir_prefix
  all_files_var
  out_var
)
  gcmake_get_without_given_prefix( ${all_files_var} "${CMAKE_CURRENT_SOURCE_DIR}" with_removed_prefix )
  set( ${out_var} ${with_removed_prefix} PARENT_SCOPE )
endfunction()

function( _gcmake_wrap_files_in_generators_helper
  build_files_list_var
  prefix_removal_mode
  out_var_build
  out_var_install
)
  set( ${out_var_build} )
  set( ${out_var_install} )

  foreach( file_for_build IN LISTS ${build_files_list_var} )
    list( APPEND ${out_var_build} "$<BUILD_INTERFACE:${file_for_build}>")
  endforeach()

  if( prefix_removal_mode STREQUAL "SOURCE" )
    gcmake_get_without_current_source_dir_prefix( ${build_files_list_var} files_for_install )
  elseif( prefix_removal_mode STREQUAL "TOPLEVEL" )
    gcmake_get_without_toplevel_dir_prefix( ${build_files_list_var} files_for_install )
  else()
    message( FATAL_ERROR "Invalid prefix_removal_mode \"${prefix_removal_mode}\"given to _gcmake_wrap_files_in_generators_helper. Must be either \"SOURCE\" or \"TOPLEVEL\" ")
  endif()

  foreach( file_for_install IN LISTS files_for_install )
    list( APPEND ${out_var_install} "$<INSTALL_INTERFACE:${file_for_install}>")
  endforeach()

  set( ${out_var_build} ${${out_var_build}} PARENT_SCOPE )
  set( ${out_var_install} ${${out_var_install}} PARENT_SCOPE )
endfunction()

macro( gcmake_wrap_files_in_generators
  build_files_list_var
  out_var_build
  out_var_install
)
  _gcmake_wrap_files_in_generators_helper( ${build_files_list_var} "SOURCE" ${out_var_build} ${out_var_install} )
endmacro()

macro( gcmake_wrap_dep_files_in_generators
  build_files_list_var
  out_var_build
  out_var_install
)
  _gcmake_wrap_files_in_generators_helper( ${build_files_list_var} "TOPLEVEL" ${out_var_build} ${out_var_install} )
endmacro()

function( gcmake_apply_exe_files
  exe_target
  receiver_target
  entry_file
  source_list_var
  header_list_var
)
  set( receiver_interface_lib ${receiver_target} )

  gcmake_wrap_files_in_generators( entry_file entry_file_build entry_file_install )
  target_sources( ${exe_target} PRIVATE "${entry_file_build}" )

  gcmake_wrap_files_in_generators( ${source_list_var} sources_build sources_install )
  target_sources( ${receiver_interface_lib} INTERFACE ${sources_build} )

  list( LENGTH ${header_list_var} num_headers )

  if( num_headers GREATER 0 )
    gcmake_wrap_files_in_generators( ${header_list_var} headers_build headers_install )
    target_sources( ${receiver_interface_lib} INTERFACE
      FILE_SET HEADERS
        FILES
          ${headers_build}
    )
  endif()
endfunction()

function( get_entry_file_alias_dir
  out_var
)
  set( ${out_var} "${CMAKE_BINARY_DIR}/aliased_entry_files/include" PARENT_SCOPE )
endfunction()

function( gcmake_apply_lib_files
  lib_target
  lib_type_spec
  entry_file
  source_list_var
  header_list_var
)
  set( _valid_lib_type_specs "COMPILED_LIB" "HEADER_ONLY_LIB" )
  if( NOT lib_type_spec IN_LIST _valid_lib_type_specs )
    message( FATAL_ERROR "Invalid lib type spec '${lib_type_spec}' given to gcmake_apply_lib_files(...)" )
  endif()

  if( lib_type_spec STREQUAL "COMPILED_LIB" )
    list( LENGTH ${source_list_var} num_non_entry_sources )
    if( num_non_entry_sources GREATER 0 )
      gcmake_wrap_files_in_generators( ${source_list_var} source_list_build source_list_install )
      target_sources( ${lib_target}
        PRIVATE
          ${source_list_build}
          # TODO: I don't think this one is needed since source files are never part of an installation.
          # However, CMake might require it. I'll have to test.
          # ${source_list_install}
      )
    endif()
  endif()

  cmake_path( GET entry_file FILENAME entry_file_name )

  # Want to make sure entry files can be included with "TOPLEVEL_INCLUDE_PREFIX/entry_file_name.extension"
  # Both when building and after installation in order to eliminate possible include issues.
  get_entry_file_alias_dir( entry_file_alias_dir )
  set( full_entry_file_alias_dir "${entry_file_alias_dir}/${TOPLEVEL_INCLUDE_PREFIX}")
  set( aliased_entry_file_path "${full_entry_file_alias_dir}/${entry_file_name}" )

  # I can't make this a PRE_BUILD command for the target because the target might be a
  # header-only library, and INTERFACE libraries can't have any associated build event
  # commands. It's annoying, but makes sense since they aren't actually ever built.
  add_custom_target( _${lib_target}_alias_file ALL
    COMMAND ${CMAKE_COMMAND} -E make_directory "${full_entry_file_alias_dir}"
    COMMAND ${CMAKE_COMMAND} -E copy_if_different "${entry_file}" "${full_entry_file_alias_dir}"
    DEPENDS "${entry_file}"
    VERBATIM
  )

  add_dependencies( ${lib_target} _${lib_target}_alias_file )

  if( "${lib_type_spec}" STREQUAL "HEADER_ONLY_LIB" )
    set( header_inheritance_mode INTERFACE )
  else()
    set( header_inheritance_mode PUBLIC )
  endif()

  # We don't actually add the aliased entry file to the build because it would mess up our installation
  # structure. The aliased file is only there to allow a uniform inclusion path for library entry
  # files when both building and after installing a library.
  set( all_headers "${entry_file}" ${${header_list_var}} )
  gcmake_wrap_files_in_generators( all_headers all_headers_build all_headers_install )

  target_sources( ${lib_target} ${header_inheritance_mode}
    FILE_SET HEADERS
      FILES
        ${all_headers_install}
        # The "build interface" headers don't need to be specified at all for the build
        # to work because they will be found inside the library's "include directories".
        # However, the headers won't be installed as part of the file set if they aren't specified
        # here as part of the build interface. I'm not sure why that is.
        ${all_headers_build}
  )
endfunction()

function( gcmake_apply_include_dirs
  target
  target_type
  project_include_dir
  # Private headers are stored in src/
  project_src_dir
)
  if( "${target_type}" STREQUAL "COMPILED_LIB" OR "${target_type}" STREQUAL "HEADER_ONLY_LIB" )
    get_entry_file_alias_dir( entry_file_alias_dir )
    set( BUILD_INTERFACE_INCLUDE_DIRS "${entry_file_alias_dir}" "${project_include_dir}")
  elseif( "${target_type}" STREQUAL "EXE_RECEIVER" OR "${target_type}")
    set( BUILD_INTERFACE_INCLUDE_DIRS "${project_include_dir}")
  else()
    message( FATAL_ERROR "Invalid target_type '${target_type}' given to function 'gcmake_apply_include_dirs'" )
  endif()

  if( "${target_type}" STREQUAL "HEADER_ONLY_LIB" )
    set( include_dir_inheritance_mode INTERFACE )
  elseif( "${target_type}" STREQUAL "EXE_RECEIVER" )
    set( include_dir_inheritance_mode INTERFACE )
  else()
    set( include_dir_inheritance_mode PUBLIC )
  endif()

  foreach( include_dir_build_only IN LISTS BUILD_INTERFACE_INCLUDE_DIRS )
    target_include_directories( ${target}
      ${include_dir_inheritance_mode}
        "$<BUILD_INTERFACE:${include_dir_build_only}>"
    )
  endforeach()

  target_include_directories( ${target}
    ${include_dir_inheritance_mode}
      "$<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>"
      "$<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}/${TOPLEVEL_INCLUDE_PREFIX}/include>"
      # Some libraries (like SFML 2.6.x) hardcode the include dir installation path to 'include/'.
      # This is fixed in SFML's master branch, but most people are going to want a stable branch.
      # This allows targets to access include files for libraries which hardcode their installation dir.
      "$<INSTALL_INTERFACE:include>"
  )

  if( NOT "${target_type}" STREQUAL "HEADER_ONLY_LIB" )
    if( "${target_type}" STREQUAL "EXE_RECEIVER" )
      set( private_include_dir_inheritance_mode INTERFACE )
    else()
      set( private_include_dir_inheritance_mode PRIVATE )
    endif()

    target_include_directories( ${target}
      ${private_include_dir_inheritance_mode}
        "$<BUILD_INTERFACE:${project_src_dir}>"
    )
  endif()
endfunction()

function( initialize_build_tests_var )
  set( option_description "Whether to build tests for the ${PROJECT_NAME} project tree." )

  if( "${CMAKE_SOURCE_DIR}" STREQUAL "${CMAKE_CURRENT_SOURCE_DIR}" AND (NOT CMAKE_CROSSCOMPILING OR (USING_EMSCRIPTEN AND GCMAKE_NODEJS_EXECUTABLE)) )
    option( ${LOCAL_TOPLEVEL_PROJECT_NAME}_BUILD_TESTS "${option_description}" ON )
  else()
    option( ${LOCAL_TOPLEVEL_PROJECT_NAME}_BUILD_TESTS "${option_description}" OFF )
  endif()
endfunction()

function( gcmake_initialize_build_docs_var )
  set( option_description "Whether to build documentation for the \"${PROJECT_NAME}\" project tree." )
  # It would be nice to build docs by default. However, since building the docs requires external tools
  # (Doxygen, maybe Sphinx, Breathe, Exhale, etc.) I think it's better to let the consumer of a project
  # manually specify that the docs should be built. This allows a quick project build to work by default
  # without requiring any additional documentation tools.
  option( ${LOCAL_TOPLEVEL_PROJECT_NAME}_BUILD_DOCS "${option_description}" OFF )
endfunction()

macro( initialize_build_config_vars )
  set( ALL_CONFIGS_LOCAL_DEFINES )

  foreach( config_name IN ITEMS "DEBUG" "RELEASE" "MINSIZEREL" "RELWITHDEBINFO" )
    set( ${config_name}_LOCAL_COMPILER_FLAGS
      ${GCMAKE_SANITIZER_FLAGS}
      ${${LOCAL_TOPLEVEL_PROJECT_NAME}_SANITIZER_FLAGS}
      ${GCMAKE_ADDITIONAL_COMPILER_FLAGS}
      ${${LOCAL_TOPLEVEL_PROJECT_NAME}_ADDITIONAL_COMPILER_FLAGS}
    )

    set( ${config_name}_LOCAL_CUDA_FLAGS
      ${GCMAKE_ADDITIONAL_CUDA_FLAGS}
      ${${LOCAL_TOPLEVEL_PROJECT_NAME}_ADDITIONAL_CUDA_FLAGS}
    )

    string( REPLACE ";" "," GCMAKE_ADDITIONAL_LINKER_FLAGS "${GCMAKE_ADDITIONAL_LINKER_FLAGS}" )
    set( ${config_name}_LOCAL_LINK_FLAGS
      ${GCMAKE_SANITIZER_FLAGS}
      ${${LOCAL_TOPLEVEL_PROJECT_NAME}_SANITIZER_FLAGS}
      "LINKER:${GCMAKE_ADDITIONAL_LINKER_FLAGS}"
      "LINKER:${${LOCAL_TOPLEVEL_PROJECT_NAME}_ADDITIONAL_LINKER_FLAGS}"
      ${GCMAKE_ADDITIONAL_LINK_TIME_FLAGS}
      ${${LOCAL_TOPLEVEL_PROJECT_NAME}_ADDITIONAL_LINK_TIME_FLAGS}
    )

    set( ${config_name}_LOCAL_DEFINES )
  endforeach()
endmacro()

macro( propagate_all_configs_local_defines )
  foreach( config_name IN ITEMS "DEBUG" "RELEASE" "MINSIZEREL" "RELWITHDEBINFO" )
    list( APPEND ${config_name}_LOCAL_DEFINES "${ALL_CONFIGS_LOCAL_DEFINES}" )
  endforeach()
endmacro()

function( initialize_ipo_defaults )
  cmake_parse_arguments( PARSE_ARGV 0 "_GCMAKE_IPO_DEFAULT" "${GCMAKE_ALL_VALID_BUILD_CONFIGS_UPPER}" "" "" )

  if( NOT IPO_DEFAULTS_INITIALIZED )
    include( CheckIPOSupported )

    check_ipo_supported(
      RESULT is_ipo_supported
      OUTPUT additional_info
    )

    if( is_ipo_supported )
      set( _ipo_enabled_for_configs )

      foreach( build_config IN LISTS GCMAKE_ALL_VALID_BUILD_CONFIGS_UPPER )
        set( _ipo_var_name "GCMAKE_ENABLE_IPO_${build_config}" )
        set( _ipo_option_message "When set to ON, enables INTERPROCEDURAL_OPTIMIZATION for the whole project tree when in ${build_config} mode (including dependencies built as part of the project)" )

        if( _GCMAKE_IPO_DEFAULT_${build_config} AND NOT USING_MINGW )
          option( ${_ipo_var_name} "${_ipo_option_message}" ON )
        else()
          option( ${_ipo_var_name} "${_ipo_option_message}" OFF )
        endif()

        set( CMAKE_INTERPROCEDURAL_OPTIMIZATION_${build_config} ${${_ipo_var_name}})
        set( CMAKE_INTERPROCEDURAL_OPTIMIZATION_${build_config} ${${_ipo_var_name}} PARENT_SCOPE )

        if( CMAKE_INTERPROCEDURAL_OPTIMIZATION_${build_config} )
          list( APPEND _ipo_enabled_for_configs "${build_config}" )
        endif()
      endforeach()

      list( LENGTH _ipo_enabled_for_configs ipo_is_enabled_for_some_configs )

      if ( ipo_is_enabled_for_some_configs )
        string( REPLACE ";" ", " _enabled_str_list "${_ipo_enabled_for_configs}" )
        message( "Interprocedural Optimization enabled by default for these configurations: ${_enabled_str_list}" )
      endif()
    else()
      message( WARNING "Skipping enabling IPO because is isn't supported. Additional info: ${additional_info}" )
    endif()

    set( IPO_DEFAULTS_INITIALIZED TRUE PARENT_SCOPE )
  endif()
endfunction()

function( initialize_lib_type_options
  DEFAULT_COMPILED_LIB_TYPE
)
  if( "${DEFAULT_COMPILED_LIB_TYPE}" STREQUAL "STATIC" )
    set( SHOULD_DEFAULT_TO_STATIC ON )
    set( SHOULD_DEFAULT_TO_SHARED OFF )
  elseif( "${DEFAULT_COMPILED_LIB_TYPE}" STREQUAL "SHARED" )
    set( SHOULD_DEFAULT_TO_STATIC OFF )
    set( SHOULD_DEFAULT_TO_SHARED ON )
  else()
    message( FATAL_ERROR "(GCMake error): DEFAULT_COMPILED_LIB_TYPE should be set to either STATIC or SHARED, but is set to invalid value '${DEFAULT_COMPILED_LIB_TYPE}'.")
  endif()

  option( BUILD_SHARED_LIBS "${LOCAL_BUILD_SHARED_LIBS_DOC_STRING}" ${SHOULD_DEFAULT_TO_SHARED} )
  option( BUILD_STATIC_LIBS "${LOCAL_BUILD_STATIC_LIBS_DOC_STRING}" ${SHOULD_DEFAULT_TO_STATIC} )
endfunction()

function( gcmake_unaliased_target_name
  target_name
  out_var
)
  get_target_property( unaliased_lib_name ${target_name} ALIASED_TARGET )

  if( NOT unaliased_lib_name )
    set( unaliased_lib_name ${target_name} )
  endif()

  set( ${out_var} ${unaliased_lib_name} PARENT_SCOPE )
endfunction()