function( copy_resource_dir_if_exists
  relative_resource_dir
)
  set( source_resource_dir "${CMAKE_CURRENT_SOURCE_DIR}/${relative_resource_dir}" )
  set( dest_resource_dir "${MY_RUNTIME_OUTPUT_DIR}/${relative_resource_dir}" )

  if( EXISTS ${source_resource_dir} )
    set_property(
      DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
      APPEND
      PROPERTY ADDITIONAL_CLEAN_FILES
        "${dest_resource_dir}"
    )

    add_custom_command(
      TARGET ${PRE_BUILD_TARGET_NAME}
      POST_BUILD
      COMMAND ${CMAKE_COMMAND}
        -E copy_directory ${source_resource_dir} ${dest_resource_dir}
      COMMENT "Copying ${PROJECT_NAME} resources"
      VERBATIM
    )
  endif()
endfunction()
