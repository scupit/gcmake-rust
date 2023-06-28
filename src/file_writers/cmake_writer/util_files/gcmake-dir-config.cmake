function( gcmake_write_dep_hash_file_if_missing
  file_path
  hash_string
)
  cmake_path( GET file_path PARENT_PATH file_dir )
  cmake_path( ABSOLUTE_PATH file_dir NORMALIZE )
  if( NOT EXISTS file_dir )
    file( MAKE_DIRECTORY "${file_dir}" )
  endif()

  if( NOT EXISTS file_path )
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
