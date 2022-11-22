function( gcmake_transform_cppfront_files
  cpp2_file_list_var
)
  set( generated_files_initial_prefix "${${PROJECT_BASE_NAME}_ENTRY_ROOT}")
  set( generated_files_dir "${${PROJECT_BASE_NAME}_GENERATED_SOURCE_ROOT}" )

  foreach( file_transforming_in IN LISTS ${cpp2_file_list_var} )
    string( REGEX REPLACE "\\.cpp2" ".cpp" file_transforming_out "${file_transforming_in}")
    string( REPLACE "${generated_files_initial_prefix}" "${generated_files_dir}" file_transforming_out "${file_transforming_out}" )
    cmake_path( GET file_transforming_out PARENT_PATH out_dir )

    add_custom_command(
      COMMAND ${CMAKE_COMMAND} -E make_directory "${out_dir}"
      # Assume we have access to cppfront::compiler target. GCMake ensures this is the case.
      COMMAND cppfront::compiler "${file_transforming_in}" -o "${file_transforming_out}"

      # Gives CMake knowledge of the cpp files to be generated so that targets
      # can use them even if they haven't been created yet.
      OUTPUT "${file_transforming_out}"

      # This command depends on the contents of the .cpp2 files, and
      # should be re-run every time a .cpp2 file is changed.
      MAIN_DEPENDENCY "${file_transforming_in}"
      VERBATIM
    )
  endforeach()
endfunction()