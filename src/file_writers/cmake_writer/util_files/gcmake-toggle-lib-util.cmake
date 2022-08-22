function( make_toggle_lib
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
