function( copy_resource_dir_if_exists
  resources_dir
  build_time_resource_dir_location
)
  if( EXISTS ${resources_dir} )
    set_property(
      DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
      APPEND
      PROPERTY ADDITIONAL_CLEAN_FILES
        "${build_time_resource_dir_location}"
    )

    add_custom_command(
      TARGET ${PRE_BUILD_TARGET_NAME}
      POST_BUILD
      COMMAND ${CMAKE_COMMAND}
        -E copy_directory ${resources_dir} ${build_time_resource_dir_location}
      COMMENT "Copying ${PROJECT_NAME} resources"
      VERBATIM
    )
  endif()
endfunction()
