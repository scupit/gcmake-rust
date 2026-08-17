
# ================================================================================
# Feature tracking.
#
# Each feature is a boolean variable named like <owner>_FEATURE_<feature-name>. The owner
# prefix is either a project name (like ${LOCAL_TOPLEVEL_PROJECT_NAME}) or a predefined
# dependency's prefix (like "GCMAKE_PREDEP_imgui" for imgui). The functions below create
# those variables and turn them on. Everything that reacts to a feature just reads its
# variable: the if() around FreeType's download section, imgui's custom_populate.cmake
# deciding whether to compile imgui_freetype.cpp, and so on.
# ================================================================================

macro( gcmake_set_use_default_features
  owner
  value
)
  set( ${owner}_USE_DEFAULT_FEATURES ${value} )
endmacro()

# Provides the ability to say a feature should be enabled any time before the owner's features
# are registered and/or configured. This is necessary for enabling specific features inside
# dependency projects and predefined dependencies.
macro( gcmake_mark_for_enable
  owner
  feature_name
)
  list( APPEND ${owner}_FEATURES ${feature_name} )
endmacro()

# gcmake_register_feature( my-project NAME my-feature ENABLES some another )
# gcmake_register_feature( GCMAKE_PREDEP_crow NAME ssl )
function( gcmake_register_feature owner )
  set( ONE_VALUE_KEYWORDS "NAME" "LIST_VALUE" )
  set( MULTI_VALUE_KEYWORDS "ENABLES" "DEP_ENABLES" )
  cmake_parse_arguments( PARSE_ARGV 1 "_FEATURE" "" "${ONE_VALUE_KEYWORDS}" "${MULTI_VALUE_KEYWORDS}")

  if( NOT DEFINED _FEATURE_NAME )
    message( FATAL_ERROR "NAME is a required parameter for gcmake_register_feature(...)." )
  endif()

  set( ${owner}_FEATURE_${_FEATURE_NAME} OFF PARENT_SCOPE )

  # Every registered feature name is recorded so gcmake_get_enabled_feature_values can
  # iterate the owner's whole feature set.
  list( APPEND ${owner}_ALL_FEATURES ${_FEATURE_NAME} )
  set( ${owner}_ALL_FEATURES ${${owner}_ALL_FEATURES} PARENT_SCOPE )

  # The value this feature contributes to gcmake_get_enabled_feature_values' output when
  # enabled. Defaults to the feature's own name.
  if( NOT DEFINED _FEATURE_LIST_VALUE )
    set( _FEATURE_LIST_VALUE "${_FEATURE_NAME}" )
  endif()
  set( ${owner}_FEATURE_${_FEATURE_NAME}_LIST_VALUE "${_FEATURE_LIST_VALUE}" PARENT_SCOPE )

  if( DEFINED _FEATURE_ENABLES )
    set( ${owner}_FEATURE_${_FEATURE_NAME}_ENABLES ${_FEATURE_ENABLES} PARENT_SCOPE )
  endif()

  # Transitively enable dependency features. Must be given in sets of two. First item = project name,
  # second item = feature name.
  if( DEFINED _FEATURE_DEP_ENABLES )
    list( LENGTH _FEATURE_DEP_ENABLES dep_enables_length )
    math( EXPR value "${dep_enables_length} % 2" OUTPUT_FORMAT DECIMAL )
    if( value EQUAL 1 )
      message( FATAL_ERROR "Features which enable dependency features must specify both the dependency project name and the feature name. However, the DEP_ENABLES list for feature ${_FEATURE_NAME} has an odd number of elements. The list: ${_FEATURE_DEP_ENABLES}")
    else()
      set( ${owner}_FEATURE_${_FEATURE_NAME}_DEP_ENABLES ${_FEATURE_DEP_ENABLES} PARENT_SCOPE )
    endif()
  endif()
endfunction()

macro( gcmake_enable_feature_if_marked
  owner
  feature_name
)
  if( NOT DEFINED ${owner}_FEATURE_${feature_name} )
    message( FATAL_ERROR "Tried to enable feature \"${feature_name}\" for \"${owner}\", but no feature with that name is registered." )
  endif()

  if( "${feature_name}" IN_LIST ${owner}_FEATURES )
    gcmake_enable_feature( ${owner} ${feature_name} )
  endif()
endmacro()

macro( gcmake_enable_feature
  owner
  feature_name
)
  if( NOT DEFINED ${owner}_FEATURE_${feature_name} )
    message( FATAL_ERROR "Tried to enable feature \"${feature_name}\" for \"${owner}\", but no feature with that name is registered." )
  endif()

  # Since the "feature enable graph" can contain cycles, we should only enable features which are
  # currently disabled in order to avoid infinite recursion.
  if( NOT ${owner}_FEATURE_${feature_name} )
    set( ${owner}_FEATURE_${feature_name} ON )

    foreach( feature_to_enable IN LISTS ${owner}_FEATURE_${feature_name}_ENABLES )
      gcmake_enable_feature( ${owner} ${feature_to_enable} )
    endforeach()

    list( LENGTH ${owner}_FEATURE_${feature_name}_DEP_ENABLES _dep_enables_end_index )
    if( _dep_enables_end_index GREATER 0 )
      math( EXPR _dep_enables_end_index "${_dep_enables_end_index} - 1" OUTPUT_FORMAT DECIMAL )

      foreach( _project_name_index RANGE 0 ${_dep_enables_end_index} 2 )
        list( GET ${owner}_FEATURE_${feature_name}_DEP_ENABLES ${_project_name_index} _project_name_containing_feature )
        math( EXPR _feature_name_index "${_project_name_index} + 1" OUTPUT_FORMAT DECIMAL )
        list( GET ${owner}_FEATURE_${feature_name}_DEP_ENABLES ${_feature_name_index} _dep_feature_name )

        gcmake_mark_for_enable( "${_project_name_containing_feature}" "${_dep_feature_name}" )
      endforeach()
    endif()
  endif()
endmacro()

# Collects the LIST_VALUE of every enabled feature of the owner into out_list_var.
# Used for dependencies configured through a single feature-list variable (e.g. CROW_FEATURES).
function( gcmake_get_enabled_feature_values owner out_list_var )
  set( _enabled_value_accum )

  foreach( _gcmake_feature_name IN LISTS ${owner}_ALL_FEATURES )
    if( ${owner}_FEATURE_${_gcmake_feature_name} )
      list( APPEND _enabled_value_accum "${${owner}_FEATURE_${_gcmake_feature_name}_LIST_VALUE}" )
    endif()
  endforeach()

  set( ${out_list_var} ${_enabled_value_accum} PARENT_SCOPE )
endfunction()
