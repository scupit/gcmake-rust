# This is a CMake "Find Module" for the Sphinx documentation tool.
# https://www.sphinx-doc.org/en/master/
# 
# Targets:
#   - Sphinx::executable

include( FindPackageHandleStandardArgs )

if( TARGET Sphinx::executable )
  find_package_handle_standard_args( Sphinx )
else()
  if( CMAKE_CROSSCOMPILING )
    # The sphinx executable will always run on the host system, since it just generates documentation
    # files. Cross-compilation shouldn't affect the search.
    set( CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER )
    set( CMAKE_FIND_ROOT_PATH_MODE_PACKAGE NEVER )
  endif()

  find_program( SPHINX_EXE
    NAMES
      "sphinx-build"
      "sphinx-build.exe"
    NAMES_PER_DIR
    DOC "Path to Sphinx executable (sphinx-build)"
    PATH_SUFFIXES "bin"
  )

  find_package_handle_standard_args( Sphinx
    SPHINX_EXE
  )

  add_executable( _sphinx_executable IMPORTED )
  add_executable( Sphinx::executable ALIAS _sphinx_executable )
  set_target_properties( _sphinx_executable
    PROPERTIES
      IMPORTED_LOCATION "${SPHINX_EXE}"
  )
endif()
