if( NOT ALREADY_CONFIGURED_EMSCRIPTEN_GCMAKE_UTIL )
  if( USING_EMSCRIPTEN )
    set( _using_ccache FALSE )
    set( _ccache_path )

    if( CMAKE_C_COMPILER_LAUNCHER )
      cmake_path( GET CMAKE_C_COMPILER_LAUNCHER FILENAME _c_launcher_name )
      if( _c_launcher_name MATCHES "ccache" )
        # set( _ccache_path "${CMAKE_C_COMPILER_LAUNCHER}" )
        # unset( CMAKE_C_COMPILER_LAUNCHER CACHE )
        set( _using_ccache TRUE )
      endif()
    endif()

    if( CMAKE_CXX_COMPILER_LAUNCHER )
      cmake_path( GET CMAKE_CXX_COMPILER_LAUNCHER FILENAME _cxx_launcher_name )
      if( _cxx_launcher_name MATCHES "ccache" )
        # set( _ccache_path "${CMAKE_CXX_COMPILER_LAUNCHER}" )
        # unset( CMAKE_CXX_COMPILER_LAUNCHER CACHE )
        set( _using_ccache TRUE )
      endif()
    endif()

    if( _using_ccache AND NOT GCMAKE_FORCE_USE_EMSCRIPTEN_CCACHE )
      # set( ENV{_EMCC_CCACHE} "${_ccache_path}" )
      message( FATAL_ERROR "CCache cannot be used when compiling a GCMake project with Emscripten. Theoretically it should work, however I haven't found a way to make it work when using the Emscripten CMake toolchain file. If you find a working Emscripten + GCMake + CCache setup, please file an issue at https://github.com/scupit/gcmake-rust. To force using CCache with Emscripten, set GCMAKE_FORCE_USE_EMSCRIPTEN_CCACHE cache variable to ON." )
    endif()
  endif()

  set( ALREADY_CONFIGURED_EMSCRIPTEN_GCMAKE_UTIL TRUE )
endif()


# Configures Emscripten-specific build settings and populates a target-specific
# variable with sidecar files (.wasm, .wasm.map, .data, .js) that can be used
# by the caller for conditional installation. The sidecar files variable is
# propagated to the parent scope using RETURN PROPAGATE.
function( apply_emscripten_specifics
  preload_flags_receiver
  actual_target
  relative_resource_dir
  sidecar_files_var_name
)
  if( USING_EMSCRIPTEN )
    set( target_file_base "${MY_RUNTIME_OUTPUT_DIR}/${actual_target}" )

    file( GLOB_RECURSE all_resource_file_paths CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/${relative_resource_dir}/**" )
    list( LENGTH all_resource_file_paths all_resources_count )

    # Determine if asset files are present for conditional .data file handling
    if( all_resources_count GREATER 0 )
      set( at_least_one_asset_file_present TRUE )
    else()
      set( at_least_one_asset_file_present FALSE )
    endif()

    # Get target type for resource handling (should be INTERFACE_LIBRARY for executables
    # or a compiled library type for libraries)
    get_target_property( preload_flags_receiver_type ${preload_flags_receiver} TYPE )

    if( at_least_one_asset_file_present )

      if( preload_flags_receiver_type STREQUAL "EXECUTABLE" )
        message( FATAL_ERROR "apply_emscripten_specifics: preload_flags_receiver should not be an EXECUTABLE target. This indicates a bug in the calling code." )
      elseif( preload_flags_receiver_type STREQUAL "INTERFACE_LIBRARY" )
        set( resource_inheritance_mode INTERFACE )
      else() # Is compiled library (STATIC_LIBRARY, SHARED_LIBRARY, etc.)
        set( resource_inheritance_mode PUBLIC )
      endif()

      # Emscripten file preloading: Package files into the virtual filesystem at runtime
      # - Uses SHELL: prefix to prevent CMake's option de-duplication from breaking up the --preload-file argument
      #   (see https://cmake.org/cmake/help/latest/command/target_link_options.html#option-de-duplication)
      # - The @ symbol maps source paths to virtual filesystem paths (like Docker volume mounting)
      # - Format: source_path@virtual_path where:
      #   * source_path: Physical files to package (${CMAKE_CURRENT_SOURCE_DIR}/${relative_resource_dir})
      #   * virtual_path: Where files appear in Emscripten's virtual filesystem (/${relative_resource_dir}/)
      # - This preserves the prefixed directory structure inside the virtual filesystem to prevent asset conflicts
      # - Files are packaged into a .data file and loaded automatically when the WebAssembly module starts
      target_link_options( ${preload_flags_receiver} ${resource_inheritance_mode} "SHELL:--preload-file ${CMAKE_CURRENT_SOURCE_DIR}/${relative_resource_dir}@/${relative_resource_dir}/" )

      set( hook_file_dir "${CMAKE_BINARY_DIR}/pre-js-hooks" )
      file( MAKE_DIRECTORY "${hook_file_dir}" )

      set( hook_file_name "${hook_file_dir}/${actual_target}.js" )
      
      file( WRITE
        "${hook_file_name}"
        "
          function doLocateFile(path, prefix) {
            if (typeof process !== 'undefined' && process.argv[1]) {
              const modifiedPath = require('path').resolve(
                process.argv[1],
                '..',
                prefix,
                path
              );

              return modifiedPath;
            }
            else {
              return prefix + path;
            }
          }

          // Ensure the module exists. Redeclaration with var in JavaScript
          // is not an error.
          var Module = Module ? Module : {};
          Module['locateFile'] = doLocateFile;
        "
      )

      target_link_options( ${actual_target}
        PRIVATE
          # It's very important that the hook file content is added to the very beginning of the
          # JS output, not just before the user's content runs. The 'locateFile' module hook
          # function must be present when the script is initially setting up in order
          # to correct the .data file loading paths when run by node.
          --extern-pre-js "${hook_file_name}"
      )
    endif()

    # Get the actual target type to determine if we should generate sidecar files
    get_target_property( actual_target_type ${actual_target} TYPE )

    # Emscripten sidecar files (.wasm, .wasm.map, .data, .js) are only generated
    # for executable targets. Libraries are compiled to static libraries without
    # associated sidecar files. Any resources attached to libraries are inherited
    # by executables that link to them.
    if( actual_target_type STREQUAL "EXECUTABLE" )
      # Always include .wasm and .wasm.map files for executables
      set( additional_files_list
        "${target_file_base}.wasm"
        "${target_file_base}.wasm.map"
      )

      # Only include .data file if resources are present to avoid CPack install failures
      if( at_least_one_asset_file_present )
        list( APPEND additional_files_list "${target_file_base}.data" )
      endif()

      if( EMSCRIPTEN_MODE STREQUAL "WITH_HTML" )
        list( APPEND additional_files_list "${target_file_base}.js" )
      endif()

      set_property(
        TARGET ${actual_target}
        APPEND PROPERTY
        ADDITIONAL_CLEAN_FILES ${additional_files_list}
      )

      # Set the target-specific variable with the sidecar files list
      set( ${sidecar_files_var_name} ${additional_files_list} )
    else()
      # Set empty list for non-executable targets
      set( ${sidecar_files_var_name} "" )
    endif()
  else()
    # Set empty list when not using Emscripten
    set( ${sidecar_files_var_name} "" )
  endif()

  # Propagate the sidecar files variable to parent scope
  return( PROPAGATE ${sidecar_files_var_name} )
endfunction()

macro( configure_emscripten_mode
  default_mode
)
  if( NOT ALREADY_CONFIGURED_EMSCRIPTEN_MODE )
    # WITH_HTML
    # NO_HTML
    set( EMSCRIPTEN_MODE ${default_mode} CACHE STRING "'WITH_HTML' builds an html file and js/wasm runnable in a web browser. 'NO_HTML' omits the html file and just creates a js file runnable by NO_HTML." )

    set( valid_emscripten_modes "WITH_HTML" "NO_HTML" )
    set_property( CACHE EMSCRIPTEN_MODE PROPERTY STRINGS ${valid_emscripten_modes} )

    if( EMSCRIPTEN_MODE STREQUAL "WITH_HTML" )
      set( CMAKE_EXECUTABLE_SUFFIX ".html" )
    elseif( EMSCRIPTEN_MODE STREQUAL "NO_HTML" )
      set( CMAKE_EXECUTABLE_SUFFIX ".js" )
    else()
      message( FATAL_ERROR "Given EMCRIPTEN_MODE '${EMSCRIPTEN_MODE}' is invalid. Must be one of: ${valid_emscripten_modes}" )
    endif()

    message( "Using Emscripten mode: ${EMSCRIPTEN_MODE}" )
    set( ALREADY_CONFIGURED_EMSCRIPTEN_MODE TRUE )
  endif()
endmacro()

function( use_custom_emscripten_shell_file
  exe_target
  html_shell_file_path
)
  set_property( TARGET ${exe_target}
    APPEND PROPERTY LINK_DEPENDS "${html_shell_file_path}"
  )

  target_link_options( ${exe_target}
    PRIVATE
      "SHELL:--shell-file '${html_shell_file_path}'"
  )
endfunction()
