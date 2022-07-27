function( clean_list
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
  set( receiver_interface_lib ${exe_target}_exe_data_receiver )

  clean_list( "${entry_file}" entry_source )
  get_without_source_dir_prefix( "${entry_source}" entry_source_install_interface )
  make_generators( "${entry_source}" "${entry_source_install_interface}" entry_source_gens )
  target_sources( ${exe_target} PRIVATE
    ${entry_source_gens_b}
    ${entry_source_gens_i}
  )

  # set( non_entry_sources ${entry_file};${sources} )
  set( non_entry_sources ${sources} )
  clean_list( "${non_entry_sources}" non_entry_sources )
  get_without_source_dir_prefix( "${non_entry_sources}" all_sources_install_interface )

  make_generators( "${non_entry_sources}" "${all_sources_install_interface}" source_gens )
  target_sources( ${receiver_interface_lib} INTERFACE
    ${source_gens_b}
    ${source_gens_i}
  )

  set( all_headers "${headers};${template_impls}" )
  clean_list( "${all_headers}" all_headers )

  if( NOT "${all_headers}" STREQUAL "" )
    get_without_source_dir_prefix( "${all_headers}" all_headers_install_interface )

    make_generators( "${all_headers}" "${all_headers_install_interface}" header_gens )
    target_sources( ${receiver_interface_lib} INTERFACE
      FILE_SET HEADERS
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
    clean_list( "${sources}" non_entry_sources)

    if( NOT "${non_entry_sources}" STREQUAL "" )
      get_without_source_dir_prefix( "${non_entry_sources}" all_sources_install_interface )

      make_generators( "${non_entry_sources}" "${all_sources_install_interface}" source_gens )
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

function( initialize_build_tests_var )
  set( option_description "Whether to build tests for the ${PROJECT_NAME} project tree." )

  if( "${CMAKE_SOURCE_DIR}" STREQUAL "${CMAKE_CURRENT_SOURCE_DIR}" )
    option( ${LOCAL_TOPLEVEL_PROJECT_NAME}_BUILD_TESTS "${option_description}" ON )
  else()
    option( ${LOCAL_TOPLEVEL_PROJECT_NAME}_BUILD_TESTS "${option_description}" OFF )
  endif()
endfunction()
