# Should only be called from the root project, and only from the toplevel project being built.
# if( "${CMAKE_SOURCE_DIR}" STREQUAL "${CMAKE_CURRENT_SOURCE_DIR}" )

function( gcmake_configure_cpack )
  include( ProcessorCount )
  ProcessorCount( num_cpu_cores )

  if( num_cpu_cores EQUAL 0 )
    set( num_cpu_cores 1 )
  endif()

  set( CPACK_NUM_PACKAGER_THREADS ${num_cpu_cores} CACHE STRING "Number of threads to use for CPack jobs" )

  set( CPACK_THREADS ${CPACK_NUM_PACKAGER_THREADS} )
  set( CPACK_ARCHIVE_THREADS ${CPACK_NUM_PACKAGER_THREADS} )

  set( requiredOneValueArgs VENDOR INSTALLER_TITLE INSTALLER_DESCRIPTION INSTALLER_EXE_PREFIX PROJECT_COMPONENT )
  cmake_parse_arguments( PARSE_ARGV 0 INSTALLER_CONFIG "" "${requiredOneValueArgs}" "" )

  foreach( required_arg IN LISTS requiredOneValueArgs )
    if( NOT INSTALLER_CONFIG_${required_arg} )
      message( FATAL_ERROR "${required_arg} is required by gcmake_configure_cpack(...), but wasn't passed.")
    endif()
  endforeach()

  get_cmake_property( LIST_OF_COMPONENTS COMPONENTS )
  # message( "components: ${LIST_OF_COMPONENTS}" )

  # https://gitlab.kitware.com/cmake/cmake/-/issues/20177
  set( CPACK_COMPONENTS_ALL ${LIST_OF_COMPONENTS} )

  set( DEP_COMPONENT_LIST ${LIST_OF_COMPONENTS} )
  list( REMOVE_ITEM DEP_COMPONENT_LIST ${INSTALLER_CONFIG_PROJECT_COMPONENT} )

  cpack_add_component( ${INSTALLER_CONFIG_PROJECT_COMPONENT}
    DISPLAY_NAME "Libraries and executables"
    DESCRIPTION "All programs build by ${INSTALLER_CONFIG_INSTALLER_TITLE}"
    DEPENDS ${DEP_COMPONENT_LIST}
  )

  foreach( dep_component_name IN LISTS DEP_COMPONENT_LIST )
    cpack_add_component( ${dep_component_name}
      DEPENDS ${INSTALLER_CONFIG_PROJECT_COMPONENT}
      HIDDEN
    )
  endforeach()

  # Currently don't support Apple because I have no way to test it.
  if( CURRENT_SYSTEM_IS_WINDOWS )
    option( CPACK_WIX_ENABLED "Generate installer using WiX" ON )
    option( CPACK_NSIS_ENABLED "Generate installer using NSIS" OFF )

    set( CPACK_GENERATOR "7Z" "ZIP" )
    set( CPACK_SOURCE_GENERATOR "ZIP" "7Z" )

    # TODO: Installer icons
    if( CPACK_WIX_ENABLED )
      list( APPEND CPACK_GENERATOR "WIX" )
      set( CPACK_WIX_ROOT_FEATURE_TITLE "${INSTALLER_CONFIG_INSTALLER_TITLE}" )
      set( CPACK_WIX_ROOT_FEATURE_DESCRIPTION "${INSTALLER_CONFIG_INSTALLER_DESCRIPTION}" )
    endif()

    if( CPACK_NSIS_ENABLED )
      list( APPEND CPACK_GENERATOR "NSIS64" )
      set( CPACK_NSIS_ENABLE_UNINSTALL_BEFORE_INSTALL ON )
      set( CPACK_NSIS_MODIFY_PATH ON )
      set( CPACK_NSIS_DISPLAY_NAME "${INSTALLER_CONFIG_INSTALLER_TITLE}" )
      set( CPACK_NSIS_PACKAGE_NAME "${INSTALLER_CONFIG_INSTALLER_TITLE}" )
      set( CPACK_NSIS_WELCOME_TITLE "Welcome to ${INSTALLER_CONFIG_INSTALLER_TITLE} Setup" )
      set( CPACK_NSIS_UNINSTALL_NAME "Uninstall ${INSTALLER_CONFIG_INSTALLER_TITLE}" )
    endif()
  elseif( CURRENT_SYSTEM_IS_LINUX )
    option( CPACK_DEB_ENABLED "Generate DEB installer" OFF )
    option( CPACK_RPM_ENABLED "Generate RPM installer" OFF )
    option( CPACK_FreeBSD_ENABLED "Generate FreeBSD installer" OFF )

    set( CPACK_GENERATOR "TGZ" "TXZ" )

    if( CPACK_DEB_ENABLED )
      list( APPEND CPACK_GENERATOR "DEB" )
    endif()

    if( CPACK_RPM_ENABLED )
      list( APPEND CPACK_GENERATOR "RPM" )
    endif()

    if( CPACK_FreeBSD_ENABLED )
      list( APPEND CPACK_GENERATOR "FreeBSD" )
    endif()

    set( CPACK_SOURCE_GENERATOR "TGZ" "TXZ" "ZIP" "7Z" )
  endif()

  set( CPACK_PACKAGE_VENDOR "${INSTALLER_CONFIG_VENDOR}" )
  set( CPACK_SOURCE_IGNORE_FILES "/\\\\.git/" "/\\\\.cache/" "/\\\\.vscode/" "/build/" "/dep/" "/__pycache__/" "/\\\\.mypy_cache/" )
  set( CPACK_PACKAGE_DESCRIPTION ${INSTALLER_CONFIG_INSTALLER_DESCRIPTION} )
  set( CPACK_PACKAGE_NAME ${INSTALLER_CONFIG_INSTALLER_EXE_PREFIX} )
  set( CPACK_PACKAGE_DIRECTORY packaged )

  set( AVAILABLE_CHECKSUM_ALGORITHMS SHA256 SHA512 )

  set( CPACK_USING_CHECKSUM_ALGORITHM SHA256 CACHE STRING "Algorithm used to generate package checksums" )
  set_property( CACHE CPACK_USING_CHECKSUM_ALGORITHM PROPERTY STRINGS ${AVAILABLE_CHECKSUM_ALGORITHMS} )

  set( CPACK_PACKAGE_CHECKSUM ${CPACK_USING_CHECKSUM_ALGORITHM} )

  include( CPack )
endfunction()
