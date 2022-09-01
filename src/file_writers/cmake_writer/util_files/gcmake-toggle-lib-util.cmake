function( resolve_actual_toggle_lib_type
  lib_name
  var_name
  out_var
)
  set( given_value ${${var_name}} )

  if( PROJECT_NAME STREQUAL LOCAL_TOPLEVEL_PROJECT_NAME )
    set( PROJECT_SPECIFIER "'${PROJECT_NAME}'" )
  else()
    set( PROJECT_SPECIFIER "'${PROJECT_NAME}' (nested in its root project '${LOCAL_TOPLEVEL_PROJECT_NAME}')")
  endif()

  if( given_value STREQUAL "DEFAULT" )
    if( BUILD_SHARED_LIBS AND BUILD_STATIC_LIBS )
      message( FATAL_ERROR "The project ${PROJECT_SPECIFIER} contains a toggle type library called '${lib_name}' which is set to use the project default library type. However, both BUILD_STATIC_LIBS and BUILD_SHARED_LIBS are set to ON. Only one of these variables should be set to ON in order for a default library type to be properly determined.")
    elseif( BUILD_SHARED_LIBS )
      set( ${out_var} SHARED PARENT_SCOPE )
    elseif( BUILD_STATIC_LIBS )
      set( ${out_var} STATIC PARENT_SCOPE )
    else()
      message( FATAL_ERROR "The project ${PROJECT_SPECIFIER} contains a toggle type library called '${lib_name}' which is set to use the project default library type. However, neither BUILD_STATIC_LIBS nor BUILD_SHARED_LIBS are set to ON. Please set either of these variables to ON to determine the default library type.")
    endif()
  elseif( given_value STREQUAL "SHARED" )
    set( ${out_var} SHARED PARENT_SCOPE )
  elseif( given_value STREQUAL "STATIC" )
    set( ${out_var} STATIC PARENT_SCOPE )
  else()
    message( FATAL_ERROR "Cannot resolve actual library type of toggle library '${lib_name}' due to unknown lib type '${given_value}' given to '${var_name}'")
  endif()
endfunction()

function( make_toggle_lib
  lib_name
  default_lib_type
)
  if( NOT "${default_lib_type}" MATCHES "^(DEFAULT|STATIC|SHARED)$" )
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

  set_property( CACHE ${LOCAL_TOPLEVEL_PROJECT_NAME}__${lib_name}_LIB_TYPE PROPERTY STRINGS "DEFAULT" "STATIC" "SHARED" )

  resolve_actual_toggle_lib_type(
    ${lib_name}
    ${LOCAL_TOPLEVEL_PROJECT_NAME}__${lib_name}_LIB_TYPE
    actual_lib_type
  )

  if ( actual_lib_type STREQUAL STATIC )
    add_library( ${lib_name} STATIC )
  elseif( actual_lib_type STREQUAL SHARED )
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
