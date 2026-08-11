function( gcmake_write_dep_hash_file
  file_path
  hash_string
)
  cmake_path( GET file_path PARENT_PATH file_dir )
  cmake_path( ABSOLUTE_PATH file_dir NORMALIZE )

  # NOTE: if( EXISTS ... ) does not dereference a bare variable name, so these must be written
  # as "${file_dir}" / "${file_path}".
  if( NOT EXISTS "${file_dir}" )
    file( MAKE_DIRECTORY "${file_dir}" )
  endif()

  # Rewrite the hash file whenever its contents don't match what gcmake-rust expects. A stale
  # hash file would make gcmake-rust either fail to locate this dependency's source tree or
  # bind to the wrong one, so "write only if missing" is not sufficient here.
  set( _should_write TRUE )

  if( EXISTS "${file_path}" )
    file( READ "${file_path}" _existing_hash )

    if( "${_existing_hash}" STREQUAL "${hash_string}" )
      set( _should_write FALSE )
    endif()
  endif()

  if( _should_write )
    file( WRITE "${file_path}" "${hash_string}" )
  endif()
endfunction()

function( ensure_gcmake_config_dirs_exist )
  if( "${CMAKE_SOURCE_DIR}" STREQUAL "${CMAKE_CURRENT_SOURCE_DIR}" )
    if( NOT IS_DIRECTORY "${GCMAKE_CONFIG_DIR}" )
      execute_process( COMMAND ${CMAKE_COMMAND} -E make_directory "${GCMAKE_CONFIG_DIR}" )
    endif()
    if( NOT IS_DIRECTORY "${GCMAKE_DEP_CACHE_DIR}" )
      execute_process( COMMAND ${CMAKE_COMMAND} -E make_directory "${GCMAKE_DEP_CACHE_DIR}" )
    endif()
  endif()
endfunction()

macro( initialize_uncached_dep_list )
  set( UNCACHED_DEP_LIST "" )
endmacro()

macro( initialize_actual_dep_list )
  set( ACTUAL_DEP_LIST "" )
endmacro()

macro( append_to_uncached_dep_list
  dep_name
)
  list( APPEND UNCACHED_DEP_LIST ${dep_name} )
endmacro()

macro( append_to_actual_dep_list
  dep_name
)
  list( APPEND ACTUAL_DEP_LIST ${dep_name} )
endmacro()
