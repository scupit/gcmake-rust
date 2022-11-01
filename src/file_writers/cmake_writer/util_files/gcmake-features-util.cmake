
macro( gcmake_set_use_default_features
  associated_project_name
  value
)
  set( ${associated_project_name}_USE_DEFAULT_FEATURES ${value} )
endmacro()

# Provides the ability to say a feature should be enabled any time before a project's features are
# registered and/or configured. This is necessary for enabling specific features inside dependency projects.
macro( gcmake_mark_for_enable
  associated_project_name
  feature_name
)
  list( APPEND ${associated_project_name}_MARKED_FOR_ENABLE ${feature_name} )
endmacro()

# gcmake_register_feature( NAME my-feature ENABLES some another noice )
function( gcmake_register_feature )
  set( ONE_VALUE_KEYWORDS "NAME" )
  set( MULTI_VALUE_KEYWORDS "ENABLES" )
  cmake_parse_arguments( PARSE_ARGV 0 "_FEATURE" "" "${ONE_VALUE_KEYWORDS}" "${MULTI_VALUE_KEYWORDS}")

  if( NOT DEFINED _FEATURE_NAME )
    message( FATAL_ERROR "NAME is a required parameter for gcmake_register_feature(...)." )
  endif()

  set( ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${_FEATURE_NAME} OFF PARENT_SCOPE )

  if( DEFINED _FEATURE_ENABLES )
    set( ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${_FEATURE_NAME}_ENABLES ${_FEATURE_ENABLES} PARENT_SCOPE )
  endif()
endfunction()

macro( gcmake_enable_feature_if_marked
  feature_name
)
  if( NOT DEFINED ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${feature_name} )
    message( FATAL_ERROR "Tried to enable feature \"${feature_name}\" in project \"${LOCAL_TOPLEVEL_PROJECT_NAME}\", but the project doesn't have a feature named\"${feature_name}\"")
  endif()

  if( "${feature_name}" IN_LIST ${LOCAL_TOPLEVEL_PROJECT_NAME}_MARKED_FOR_ENABLE )
    gcmake_enable_feature( ${feature_name} )
  endif()
endmacro()

macro( gcmake_enable_feature
  feature_name
)
  if( NOT DEFINED ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${feature_name} )
    message( FATAL_ERROR "Tried to enable feature \"${feature_name}\" in project \"${LOCAL_TOPLEVEL_PROJECT_NAME}\", but the project doesn't have a feature named\"${feature_name}\"")
  endif()

  # Since the "feature enable graph" can contain cycles, we should only enable features which are
  # currently disabled in order to avoid infinite recursion.
  if( NOT ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${feature_name} )
    set( ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${feature_name} ON )

    foreach( feature_to_enable IN LISTS ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${feature_name}_ENABLES )
      gcmake_enable_feature( ${feature_to_enable} )
    endforeach()
  endif()
endmacro()