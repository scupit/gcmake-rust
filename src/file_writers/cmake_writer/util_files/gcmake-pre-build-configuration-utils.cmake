if( NOT GCMAKE_PRE_BUILD_UTIL_HAS_BEEN_INCLUDED )
  set( GCMAKE_GLOBAL_PRE_BUILD_TARGET run-pre-build )
  add_custom_target( ${GCMAKE_GLOBAL_PRE_BUILD_TARGET}
    ALL
    COMMENT "Running all pre-build scripts in the entire GCMake project tree, including GCMake dependencies"
  )
  
  set( GCMAKE_PRE_BUILD_UTIL_HAS_BEEN_INCLUDED TRUE )
endif()

function( initialize_prebuild_step
  pre_build_name
)
  set( PRE_BUILD_TARGET_NAME ${pre_build_name}_PRE_BUILD_STEP )
  set( PRE_BUILD_TARGET_NAME ${PRE_BUILD_TARGET_NAME} PARENT_SCOPE )
  add_custom_target( ${PRE_BUILD_TARGET_NAME}
    ALL
    COMMENT "Beginning pre-build processing"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
  )
  add_dependencies( run-pre-build ${PRE_BUILD_TARGET_NAME} )
endfunction()

function( use_executable_prebuild_script
  pre_build_executable_target
)
  if( NOT CMAKE_CROSSCOMPILING )
    add_custom_command(
      TARGET ${PRE_BUILD_TARGET_NAME}
      PRE_BUILD
      COMMAND ${pre_build_executable_target}
      COMMENT "Running ${PROJECT_NAME} pre-build executable script"
      WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    )
  endif()
endfunction()

function( use_python_prebuild_script
  python_prebuild_file
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
