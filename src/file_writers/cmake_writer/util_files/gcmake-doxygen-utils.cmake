# Assumes Doxygen and its targets were already found.

function( _gcmake_get_source_dirs
  subproject_dir_list_var
  out_var
)
  set( dirs_list )

  foreach( subproject_dir IN ${subproject_dir_list_var} )
    list( APPEND dirs_list "${subproject_dir}/src" )
    list( APPEND dirs_list "${subproject_dir}/include" )
  endforeach()

  set( out_var ${dirs_list} PARENT_SCOPE )
endfunction()

function( gcmake_use_doxygen
  all_documentable_files_var
)

find_package( Doxygen )
if( NOT Doxygen_FOUND )
  message( FATAL_ERROR "Unable to locate a doxygen executable. A Doxygen executable is required to document project '${PROJECT_NAME}'.\nSee https://www.doxygen.nl/manual/install.html for instructions on how to install Doxygen." )
endif()

set( DOXYGEN_INPUT_FILES ${${all_documentable_files_var}} )
string( REPLACE ";" "\n  " DEBUG_DOXY_INPUT_FILES "${DOXYGEN_INPUT_FILES}")
message( "Input files:\n${DEBUG_DOXY_INPUT_FILES}")

# DOXYGEN_INPUTS is the string value used to set the INPUTS option in Doxyfile.in.
list( TRANSFORM DOXYGEN_INPUT_FILES APPEND "\"" OUTPUT_VARIABLE DOXYGEN_INPUTS )
list( TRANSFORM DOXYGEN_INPUTS PREPEND "\"" )
string( REGEX REPLACE "\";\"" "\" \"" DOXYGEN_INPUTS "${DOXYGEN_INPUTS}" )

set( DOXYGEN_OUTPUT_DIR "${CMAKE_BINARY_DIR}/docs/${PROJECT_NAME}")
set( DOXYFILE_IN "${CMAKE_CURRENT_SOURCE_DIR}/docs/Doxyfile.in" )
set( DOXYFILE_OUT "${CMAKE_CURRENT_SOURCE_DIR}/docs/Doxyfile" )

if( ${PROJECT_NAME}_BUILD_DOCS )
  configure_file( ${DOXYFILE_IN} ${DOXYFILE_OUT} @ONLY )
  add_custom_target( ${PROJECT_NAME}-docs ALL
    COMMAND Doxygen::doxygen "${DOXYFILE_OUT}"
    WORKING_DIRECTORY "${TOPLEVEL_PROJECT_DIR}"
    DEPENDS ${DOXYFILE_OUT} ${DOXYGEN_INPUT_FILES}
    # Main output file
    BYPRODUCTS "${DOXYGEN_OUTPUT_DIR}/html/index.html"
    COMMAND_EXPAND_LISTS
    VERBATIM
    COMMENT "Generating documentation for project \"${PROJECT_NAME}\""
  )
endif()

endfunction()