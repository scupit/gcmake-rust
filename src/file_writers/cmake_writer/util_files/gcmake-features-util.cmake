
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
  list( APPEND ${associated_project_name}_FEATURES ${feature_name} )
endmacro()

# gcmake_register_feature( NAME my-feature ENABLES some another noice )
function( gcmake_register_feature )
  set( ONE_VALUE_KEYWORDS "NAME" )
  set( MULTI_VALUE_KEYWORDS "ENABLES" "DEP_ENABLES" )
  cmake_parse_arguments( PARSE_ARGV 0 "_FEATURE" "" "${ONE_VALUE_KEYWORDS}" "${MULTI_VALUE_KEYWORDS}")

  if( NOT DEFINED _FEATURE_NAME )
    message( FATAL_ERROR "NAME is a required parameter for gcmake_register_feature(...)." )
  endif()

  set( ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${_FEATURE_NAME} OFF PARENT_SCOPE )

  if( DEFINED _FEATURE_ENABLES )
    set( ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${_FEATURE_NAME}_ENABLES ${_FEATURE_ENABLES} PARENT_SCOPE )
  endif()

  # Transitively enable dependency features. Must be given in sets of two. First item = project name,
  # second item = feature name.
  if( DEFINED _FEATURE_DEP_ENABLES )
    list( LENGTH _FEATURE_DEP_ENABLES dep_enables_length )
    math( EXPR value "${dep_enables_length} % 2" OUTPUT_FORMAT DECIMAL )
    if( value EQUAL 1 )
      message( FATAL_ERROR "Features which enable dependency features must specify both the dependency project name and the feature name. However, the DEP_ENABLES list for feature ${_FEATURE_NAME} has an odd number of elements. The list: ${_FEATURE_DEP_ENABLES}")
    else()
      set( ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${_FEATURE_NAME}_DEP_ENABLES ${_FEATURE_DEP_ENABLES} PARENT_SCOPE )
    endif()
  endif()
endfunction()

macro( gcmake_enable_feature_if_marked
  feature_name
)
  if( NOT DEFINED ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${feature_name} )
    message( FATAL_ERROR "Tried to enable feature \"${feature_name}\" in project \"${LOCAL_TOPLEVEL_PROJECT_NAME}\", but the project doesn't have a feature named\"${feature_name}\"")
  endif()

  if( "${feature_name}" IN_LIST ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURES )
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

    list( LENGTH ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${feature_name}_DEP_ENABLES _dep_enables_end_index )
    if( _dep_enables_end_index GREATER 0 )
      math( EXPR _dep_enables_end_index "${_dep_enables_end_index} - 1" OUTPUT_FORMAT DECIMAL )

      foreach( _project_name_index RANGE 0 ${_dep_enables_end_index} 2 )
        list( GET ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${feature_name}_DEP_ENABLES ${_project_name_index} _project_name_containing_feature )
        math( EXPR _feature_name_index "${_project_name_index} + 1" OUTPUT_FORMAT DECIMAL )
        list( GET ${LOCAL_TOPLEVEL_PROJECT_NAME}_FEATURE_${feature_name}_DEP_ENABLES ${_feature_name_index} _dep_feature_name )

        gcmake_mark_for_enable( "${_project_name_containing_feature}" "${_dep_feature_name}" )
      endforeach()
    endif()
  endif()
endmacro()