# It's hard to find good reading material on using .rc files with CMake, but this answer has some good info.
# https://stackoverflow.com/questions/68517552/how-to-add-icon-to-a-qt-application-on-windows-using-a-rc-file-on-a-cmake-proje

if( NOT GCMAKE_WINDOWS_RC_UTIL_CONFIG_HAS_RUN )
  if( TARGET_SYSTEM_IS_WINDOWS )
    # Windows configuration files (.rc) are built, then linked as part of an executable program.
    # An example use case is setting the icon of an exe file.
    enable_language( RC )
  endif()
  set( GCMAKE_WINDOWS_RC_UTIL_CONFIG_HAS_RUN TRUE )
endif()

function( generate_rc_file_for_windows_exe
  target_name
)
  gcmake_unaliased_target_name( ${target_name} TARGET_BASE_NAME )
  if( TARGET_SYSTEM_IS_WINDOWS AND NOT ${TARGET_BASE_NAME}_RC_ALREADY_GENERATED )
    set( optionalOneValueArgs ICON_PATH )
    cmake_parse_arguments( PARSE_ARGV 1 RC_CONFIG "" "${optionalOneValueArgs}" "" )

    set( RC_FILE_CONTENT )
    string( MAKE_C_IDENTIFIER "${TARGET_BASE_NAME}" useable_target_name )

    if( RC_CONFIG_ICON_PATH )
      # For a target named my-test, this would generate the line:
      # my_testIcon ICON "C:\Path\to_icon.ico"
      # I think my_testIcon is an identifier which can be used in windows GUI apps, but I'm not sure yet.
      string( APPEND RC_FILE_CONTENT "${useable_target_name}Icon ICON \"${RC_CONFIG_ICON_PATH}\"\n" )
    endif()

    set( RC_FILE_PATH "${CMAKE_BINARY_DIR}/generated_windows_rc_files/${TARGET_BASE_NAME}.rc" )

    file( WRITE
      "${RC_FILE_PATH}"
      "${RC_FILE_CONTENT}"
    )

    # This file path doesn't need a 'windows-only' generator expression because this
    # function body is only run when the target system is Windows anyways.
    target_sources( ${TARGET_BASE_NAME} PRIVATE "${RC_FILE_PATH}" )

    set( ${TARGET_BASE_NAME}_RC_ALREADY_GENERATED TRUE )
  endif()
endfunction()