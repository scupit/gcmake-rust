if( NOT GCMAKE_PRE_BUILD_UTIL_HAS_BEEN_INCLUDED )
  set( GCMAKE_GLOBAL_PRE_BUILD_TARGET run-pre-build )
  add_custom_target( ${GCMAKE_GLOBAL_PRE_BUILD_TARGET}
    ALL
    COMMENT "Running all pre-build scripts in the entire GCMake project tree, including GCMake dependencies"
  )

  if( USING_EMSCRIPTEN )
    find_program( GCMAKE_NODEJS_EXECUTABLE
      NAMES "node" "nodejs"
    )

    if( NOT GCMAKE_NODEJS_EXECUTABLE )
      message( WARNING "GCMake Warning: Unable to find a NodeJS executable on your system. Any executable pre-build scripts built by your project will not be run when using Emscripten." )
    endif()
  endif()
  
  set( GCMAKE_PRE_BUILD_UTIL_HAS_BEEN_INCLUDED TRUE )
endif()

function( initialize_prebuild_step
  pre_build_name
)
  set( PRE_BUILD_TARGET_NAME ${pre_build_name}_PRE_BUILD_STEP )
  set( PRE_BUILD_TARGET_NAME ${PRE_BUILD_TARGET_NAME} PARENT_SCOPE )
  add_custom_target( ${PRE_BUILD_TARGET_NAME}
    # ALL
    COMMENT "Beginning pre-build processing"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
  )
  add_dependencies( ${GCMAKE_GLOBAL_PRE_BUILD_TARGET} ${PRE_BUILD_TARGET_NAME} )
endfunction()

function( use_executable_prebuild_script
  pre_build_executable_target
  generated_files_var
)
  if( USING_EMSCRIPTEN AND EMSCRIPTEN_MODE STREQUAL "WITH_HTML" AND GCMAKE_NODEJS_EXECUTABLE )
    # The WITH_HTML "executable" output file is a .html file. A runnable .js file is also guaranteed to
    # be produced, so we need to make sure to run that instead of attempting to run the .html file with
    # Node.
    set( runnable_command ${GCMAKE_NODEJS_EXECUTABLE} "${MY_RUNTIME_OUTPUT_DIR}/${pre_build_executable_target}.js" )
  else()
    set( runnable_command ${pre_build_executable_target} )
  endif()

  # https://cmake.org/cmake/help/latest/command/add_custom_command.html
  # According to CMake's add_custom_command(...) docs, a COMMAND which
  # is a target will not be run when CMAKE_CROSSCOMPILING, unless a
  # CMAKE_CROSSCOMPILING_EMULATOR is specified. 
  # Therefore we don't have to guard this with a conditional, since CMake
  # already does that for us.
  add_dependencies( ${PRE_BUILD_TARGET_NAME} ${pre_build_executable_target} )
  add_custom_command(
    TARGET ${PRE_BUILD_TARGET_NAME}
    PRE_BUILD
    COMMAND ${runnable_command}
    COMMENT "Running ${PROJECT_NAME} pre-build executable script"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    BYPRODUCTS ${${generated_files_var}}
    COMMAND_EXPAND_LISTS
  )
endfunction()

function( use_python_prebuild_script
  python_prebuild_file
  generated_files_var
)
  include( FindPython3 )
  find_package( Python3 COMPONENTS Interpreter )

  if( ${Python3_FOUND} AND ${Python3_Interpreter_FOUND} )
    add_custom_command(
      TARGET ${PRE_BUILD_TARGET_NAME}
      PRE_BUILD
      COMMAND Python3::Interpreter ${python_prebuild_file}
      COMMENT "Running ${PROJECT_NAME} pre-build python script"
      WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
      BYPRODUCTS ${${generated_files_var}}
      COMMAND_EXPAND_LISTS
    )
  else()
    if( NOT ${Python3_Interpreter_FOUND} )
      message( FATAL_ERROR "A Python 3 interpreter is needed to run the pre-build script for project ${PROJECT_NAME}, however a valid interpreter was not found." )
    else()
      message( FATAL_ERROR "Unable to find a valid Python 3 configuration when configuring project ${PROJECT_NAME}" )
    endif()
  endif()
endfunction()

function( add_depends_on_pre_build
  some_target
)
  add_dependencies( ${some_target} ${PRE_BUILD_TARGET_NAME} )
endfunction()
