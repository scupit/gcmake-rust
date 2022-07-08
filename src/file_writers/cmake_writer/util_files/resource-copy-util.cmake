function( copy_resource_dir_if_exists
  resources_dir
  build_time_resource_dir_location
  pre_build_step_target
)
  if( EXISTS ${resources_dir} )
    add_custom_command(
      TARGET ${PROJECT_NAME}-pre-build-step
      PRE_BUILD
      COMMAND ${CMAKE_COMMAND}
        -E copy_directory ${resources_dir} ${build_time_resource_dir_location}
      COMMENT "Copying ${PROJECT_NAME} resources"
      VERBATIM
    )
  endif()
endfunction()
