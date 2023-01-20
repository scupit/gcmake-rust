function( _gcmake_get_predefined_macro_list
  all_generated_export_headers_var
  out_var
)
  set( all_export_macros )

  # path/to/lib-name_export.h
  foreach( generated_export_header_path IN LISTS ${all_generated_export_headers_var} )
    # lib-name_export
    cmake_path( GET generated_export_header_path STEM generated_filename )

    # lib_name_export
    string( MAKE_C_IDENTIFIER "${generated_filename}" export_macro_name )

    # LIB_NAME_EXPORT: This is the same as the generated export macro
    string( TOUPPER "${export_macro_name}" export_macro_name )
    list( APPEND all_export_macros "${export_macro_name}" )
  endforeach()

  set( ${out_var} ${all_export_macros} PARENT_SCOPE )
endfunction()

function( gcmake_configure_documentation
  doc_generator
  all_generated_export_headers_var
  all_documentable_files_var
  out_var_docs_output_dir
)
  set( VALID_DOC_GENERATORS "Sphinx" "Doxygen" )

  if( NOT doc_generator IN_LIST VALID_DOC_GENERATORS )
    message( FATAL_ERROR "Invalid documentation generator \"${doc_generator}\" given. Must be one of: ${VALID_DOC_GENERATORS}" )
  endif()

  find_package( Doxygen MODULE )

  if( NOT Doxygen_FOUND )
    if( doc_generator STREQUAL "Sphinx" )
      message( FATAL_ERROR "Unable to locate a doxygen executable. A Doxygen executable is required to document project '${PROJECT_NAME}' even though '${PROJECT_NAME}' is configured to use Sphinx. (Sphinx depends on Doxygen's output).\nSee https://www.doxygen.nl/manual/install.html for instructions on how to install Doxygen." )
    else()
      message( FATAL_ERROR "Unable to locate a doxygen executable. A Doxygen executable is required to document project '${PROJECT_NAME}'.\nSee https://www.doxygen.nl/manual/install.html for instructions on how to install Doxygen." )
    endif()
  endif()

  if( doc_generator STREQUAL "Sphinx" )
    find_package( Sphinx MODULE )
    if( NOT Sphinx_FOUND )
      message( FATAL_ERROR "Unable to locate sphinx-build executable. The sphinx-build executable is required to document project '${PROJECT_NAME}'.\nSee https://www.sphinx-doc.org/en/master/usage/installation.html for installation instructions." )
    endif()
  endif()

  set( DOXYGEN_PREDEFINED_MACROS )
  _gcmake_get_predefined_macro_list(
    ${all_generated_export_headers_var}
    DOXYGEN_PREDEFINED_MACROS
  )

  list( PREPEND DOXYGEN_PREDEFINED_MACROS "__declspec(x)" "__attribute__(x)" )
  list( TRANSFORM DOXYGEN_PREDEFINED_MACROS APPEND "= ")
  list( JOIN DOXYGEN_PREDEFINED_MACROS "\\\n\t\t" DOXYGEN_PREDEFINED_MACROS )

  set( DOXYGEN_INPUT_FILES ${${all_documentable_files_var}} )
  list( APPEND DOXYGEN_INPUT_FILES ${${all_generated_export_headers_var}} )
  list( REMOVE_DUPLICATES DOXYGEN_INPUT_FILES )

  # DOXYGEN_INPUTS is the string value used to set the INPUTS option in Doxyfile.in.
  list( TRANSFORM DOXYGEN_INPUT_FILES APPEND "\"" OUTPUT_VARIABLE DOXYGEN_INPUTS )
  list( TRANSFORM DOXYGEN_INPUTS PREPEND "\"" )
  string( REGEX REPLACE "\";\"" "\" \"" DOXYGEN_INPUTS "${DOXYGEN_INPUTS}" )

  set( DOCS_CONFIG_DIR "${CMAKE_CURRENT_SOURCE_DIR}/docs" )

  set( DOXYGEN_OUTPUT_DIR "${CMAKE_BINARY_DIR}/docs/${PROJECT_NAME}/doxygen")

  if( NOT IS_DIRECTORY DOXYGEN_OUTPUT_DIR )
    file( MAKE_DIRECTORY "${DOXYGEN_OUTPUT_DIR}" )
  endif()

  set( DOXYFILE_IN "${DOCS_CONFIG_DIR}/Doxyfile.in" )
  set( DOXYFILE_OUT "${DOCS_CONFIG_DIR}/Doxyfile" )

  if( ${PROJECT_NAME}_BUILD_DOCS )
    configure_file( ${DOXYFILE_IN} ${DOXYFILE_OUT} @ONLY )
    set( ${out_var_docs_output_dir} "${DOXYGEN_OUTPUT_DIR}" PARENT_SCOPE )

    # TODO: Change these to add_custom_command so they aren't re-run every pass.
    # An idea is to create an empty custom target called ${PROJECT_NAME}-docs which depends
    # on the output of the anonymous custom commands.
    add_custom_command(
      COMMAND Doxygen::doxygen "${DOXYFILE_OUT}"
      OUTPUT
        "${DOXYGEN_OUTPUT_DIR}/html/index.html"
        "${DOXYGEN_OUTPUT_DIR}/xml/index.xml"
      WORKING_DIRECTORY "${DOCS_CONFIG_DIR}"
      DEPENDS
        "${DOXYGEN_OUTPUT_DIR}"
        "${DOXYFILE_OUT}"
        "${DOXYFILE_IN}"
        ${DOXYGEN_INPUT_FILES}
      COMMENT "Generating Doxygen documentation for project \"${PROJECT_NAME}\""
      COMMAND_EXPAND_LISTS
      VERBATIM
    )

    if( doc_generator STREQUAL "Sphinx" )
      set( SPHINX_OUTPUT_DIR "${CMAKE_BINARY_DIR}/docs/${PROJECT_NAME}/sphinx" )

      if( NOT IS_DIRECTORY SPHINX_OUTPUT_DIR )
        file( MAKE_DIRECTORY "${SPHINX_OUTPUT_DIR}" )
      endif()

      set( SPHINX_PY_CONFIG_IN "${DOCS_CONFIG_DIR}/conf.py.in" )
      set( SPHINX_PY_CONFIG_OUT "${DOCS_CONFIG_DIR}/conf.py" )
      set( SPHINX_INDEX_RST "${DOCS_CONFIG_DIR}/index.rst" )

      configure_file( "${SPHINX_PY_CONFIG_IN}" "${SPHINX_PY_CONFIG_OUT}" @ONLY )
      set( ${out_var_docs_output_dir} "${SPHINX_OUTPUT_DIR}" PARENT_SCOPE )

      add_custom_command(
        COMMAND Sphinx::executable -b html
          "-Dbreathe_projects.${PROJECT_NAME}=${DOXYGEN_OUTPUT_DIR}/xml"
          "${DOCS_CONFIG_DIR}"
          "${SPHINX_OUTPUT_DIR}"
        OUTPUT
          "${SPHINX_OUTPUT_DIR}/index.html"
        DEPENDS
          "${SPHINX_OUTPUT_DIR}"
          "${DOXYGEN_OUTPUT_DIR}/xml/index.xml"   # Just in case the target dependency fails for some reason, ensure we have the output xml file. This check might not be necessary. 
          "${SPHINX_INDEX_RST}"
          "${SPHINX_PY_CONFIG_IN}"
          "${SPHINX_PY_CONFIG_OUT}"
        COMMENT "Generating Sphinx documentation for project \"${PROJECT_NAME}\""
        WORKING_DIRECTORY "${DOCS_CONFIG_DIR}"
        VERBATIM
      )

      add_custom_target( ${PROJECT_NAME}-docs ALL
        DEPENDS
          "${SPHINX_OUTPUT_DIR}/index.html"
      )
    else()
      add_custom_target( ${PROJECT_NAME}-docs ALL
        DEPENDS
          "${DOXYGEN_OUTPUT_DIR}/html/index.html"
          "${DOXYGEN_OUTPUT_DIR}/xml/index.xml"
      )
    endif()
  endif()
endfunction()