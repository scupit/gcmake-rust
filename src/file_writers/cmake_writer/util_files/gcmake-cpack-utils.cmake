# Should only be called from the root project, and only from the toplevel project being built.
# if( "${CMAKE_SOURCE_DIR}" STREQUAL "${CMAKE_CURRENT_SOURCE_DIR}" )
function( gcmake_configure_cpack
  vendor_name
)
  include( ProcessorCount )
  ProcessorCount( num_cpu_cores )

  if( num_cpu_cores EQUAL 0 )
    set( num_cpu_cores 1 )
  endif()

  set( CMAKE_NUM_PACKAGER_THREADS ${num_cpu_cores} CACHE STRING "Number of threads to use for CPack jobs" )

  set( CPACK_THREADS ${CMAKE_NUM_PACKAGER_THREADS} )
  set( CPACK_ARCHIVE_THREADS ${CMAKE_NUM_PACKAGER_THREADS} )

  # Currently don't support Apple because I have no way to test it.
  if( CURRENT_SYSTEM_IS_WINDOWS )
    option( CPACK_WIX_ENABLED "Generate installer using WiX" ON )
    option( CPACK_NSIS_ENABLED "Generate installer using NSIS" OFF )

    set( CPACK_GENERATOR "7Z" "ZIP" )
    set( CPACK_SOURCE_GENERATOR "ZIP" "7Z" )

    if( CPACK_WIX_ENABLED )
      list( APPEND CPACK_GENERATOR "WIX" )
    endif()

    if( CPACK_NSIS_ENABLED )
      list( APPEND CPACK_GENERATOR "NSIS" )
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

  set( CPACK_PACKAGE_VENDOR "${vendor_name}" )
  set( CPACK_SOURCE_IGNORE_FILES "/\\\\.git/" "/\\\\.vscode/" "/build/" "/dep/" "/__pycache__/" "/\\\\.mypy_cache/" )

  include( CPack )
endfunction()
