function( err_if_cross_compiling
  override_var
)
  if( CMAKE_CROSSCOMPILING AND NOT ${override_var} )
    message( FATAL_ERROR "Project ${PROJECT_NAME} does not support trivial cross-compilation. To override this error and force cross-compilation, set ${override_var} to ON" )
  endif()
endfunction()

function( err_if_using_emscripten
  override_var
)
  if( USING_EMSCRIPTEN AND NOT ${override_var} )
    message( FATAL_ERROR "Project ${PROJECT_NAME} does not support compilation with Emscripten. To override this error and force-allow compilation with Emscripten, set ${override_var} to ON" )
  endif()
endfunction()