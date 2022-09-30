macro( initialize_pgo_defaults )
  if( NOT PGO_DEFAULTS_INITIALIZED AND (NOT CMAKE_CROSSCOMPILING OR (USING_EMSCRIPTEN AND GCMAKE_NODEJS_EXECUTABLE)) )
    set( VALID_PGO_STEP_VALUES NONE PROFILE_GENERATION USE_PROFILES )
    set( GCMAKE_PGO_STEP "NONE" CACHE STRING "The current step in the profile-guided optimization process. To turn off (not use) PGO, set this to NONE" )
    set_property( CACHE GCMAKE_PGO_STEP PROPERTY STRINGS ${VALID_PGO_STEP_VALUES} )

    list( FIND VALID_PGO_STEP_VALUES "${GCMAKE_PGO_STEP}" IS_PGO_STEP_VALID )

    if( IS_PGO_STEP_VALID LESS 0 )
      message( FATAL_ERROR "Invalid PGO step '${GCMAKE_PGO_STEP}' given. Must be one of: ${VALID_PGO_STEP_VALUES}")
    endif()

    if( USING_GCC )
      if( GCMAKE_PGO_STEP STREQUAL "PROFILE_GENERATION" )
        link_libraries(gcov)
      endif()

      if( CURRENT_SYSTEM_IS_WINDOWS )
        message( NOTICE "When using GCC PGO on Windows, make sure GCC's bin/ directory is exposed by the system PATH. Compilation and running will not work correctly otherwise." )
      endif()

      add_compile_options(
        # GCC generation step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},PROFILE_GENERATION>:-fprofile-generate;-fprofile-update=prefer-atomic>"
        # GCC usage step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},USE_PROFILES>:-fprofile-use;-fprofile-correction>"
      )
      add_link_options(
        # GCC generation step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},PROFILE_GENERATION>:-fprofile-generate;-fprofile-update=prefer-atomic>"
        # GCC usage step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},USE_PROFILES>:-fprofile-use;-fprofile-correction>"
      )
    elseif( USING_CLANG )
      if( CURRENT_SYSTEM_IS_WINDOWS )
        # try_copy_llvm_libunwind_dll()
        # try_copy_llvm_libcpp_dll()
        message( NOTICE "When using Clang PGO on Windows, make sure Clang/LLVM's bin/ directory is exposed by the system PATH. Compilation and running will not work correctly otherwise." )
      endif()

      set( PROFILES_DIR "${CMAKE_BINARY_DIR}/instrumentation_profiles" )

      add_compile_options(
        # Clang generation step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},PROFILE_GENERATION>:-fprofile-generate=${PROFILES_DIR};-fprofile-update=atomic>"
        # Clang usage step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},USE_PROFILES>:-fprofile-use=${PROFILES_DIR}>"
      )
      add_link_options(
        # Clang generation step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},PROFILE_GENERATION>:-fprofile-generate=${PROFILES_DIR};-fprofile-update=atomic>"
        # Clang usage step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},USE_PROFILES>:-fprofile-use=${PROFILES_DIR}>"
      )

      # https://clang.llvm.org/docs/UsersManual.html#profiling-with-instrumentation
      if( GCMAKE_PGO_STEP STREQUAL "USE_PROFILES" )
        locate_llvm_profdata_exe( llvm_profdata_exe )
        message( "llvm-profdata found: ${llvm_profdata_exe}")
        set( MERGED_PROFILES_FILE "${PROFILES_DIR}/default.profdata")

        add_custom_target( merge_clang_profile_data ALL
          DEPENDS "${PROFILES_DIR}"
        )
        
        add_custom_command(
          TARGET merge_clang_profile_data
          PRE_BUILD
          COMMAND
            ${llvm_profdata_exe} merge "-output=${MERGED_PROFILES_FILE}" "${PROFILES_DIR}/*.profraw"
        )
      endif()
    elseif( USING_MSVC)
      if( GCMAKE_PGO_STEP STREQUAL "PROFILE_GENERATION" )
        copy_msvc_pgort140_dll()
      endif()

      add_compile_options( 
        # MSVC generation step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},PROFILE_GENERATION>:/GL>"
        # MSVC usage step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},USE_PROFILES>:/GL>"
      )

      add_link_options(
        # MSVC generation step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},PROFILE_GENERATION>:/LTCG;/GENPROFILE>"
        # MSVC usage step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},USE_PROFILES>:/LTCG;/USEPROFILE>"
      )
    endif()

    set( PGO_DEFAULTS_INITIALIZED TRUE )
    message( STATUS "PGO Step: ${GCMAKE_PGO_STEP}")
  endif()
endmacro()

function( copy_msvc_pgort140_dll )
  set( COMPILER_PATHS_CHECKING "${CMAKE_CXX_COMPILER}" "${CMAKE_C_COMPILER}" )
  set( LOCATIONS_CHECKING "." "./onecore" )

  set( possible_pgort_dirs )

  foreach( absolute_compiler_path IN LISTS COMPILER_PATHS_CHECKING )
    cmake_path( REMOVE_FILENAME absolute_compiler_path OUTPUT_VARIABLE compiler_dir )

    foreach( relative_location_checking IN LISTS LOCATIONS_CHECKING )
      cmake_path(
        APPEND compiler_dir "${relative_location_checking}"
        OUTPUT_VARIABLE maybe_pgort_dir
      )

      list( APPEND possible_pgort_dirs "${maybe_pgort_dir}" )
    endforeach()
  endforeach()

  set( the_pgo_file_name "pgort140.dll" )

  find_file( msvc_pgort140_dll
    NAMES "${the_pgo_file_name}"
    PATHS ${possible_pgort_dirs}
    DOC "MSVC's Profile Guided Optimization RunTime"
    NO_CMAKE_ENVIRONMENT_PATH
    NO_SYSTEM_ENVIRONMENT_PATH
    NO_CMAKE_SYSTEM_PATH
    NO_DEFAULT_PATH
  )

  if( msvc_pgort140_dll )
    add_custom_target( _pgo_copy_pgort140_dll ALL )
    add_custom_command(
      TARGET _pgo_copy_pgort140_dll
      PRE_BUILD
      COMMAND
        ${CMAKE_COMMAND} -E copy ${msvc_pgort140_dll} "${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${the_pgo_file_name}"
    )
  else()
    message( FATAL_ERROR "Unable to find MSVC's pgort140.dll, which is required for running PGO instrumented binaries." )
  endif()
endfunction()

function( _pgo_get_compiler_dirs
  out_var
)
  set( compiler_dirs )
  foreach( compiler_file_path IN ITEMS "${CMAKE_C_COMPILER}" "${CMAKE_CXX_COMPILER}" )
    cmake_path( REMOVE_FILENAME compiler_file_path OUTPUT_VARIABLE compiler_dir_path )
    list( APPEND compiler_dirs "${compiler_dir_path}")
  endforeach()

  set( ${out_var} ${compiler_dirs} PARENT_SCOPE )
endfunction()

function( locate_llvm_profdata_exe
  out_var
)
  _pgo_get_compiler_dirs( compiler_dirs )

  find_program( llvm_profdata_exe
    NAMES "llvm-profdata" "llvm-profdata.exe"
    PATHS ${compiler_dirs}
    DOC "The executable which combines multiple .profraw files into a single .profdata which is used during the PGO PROFILE_USE step"
    NO_CMAKE_ENVIRONMENT_PATH
    NO_SYSTEM_ENVIRONMENT_PATH
    NO_CMAKE_SYSTEM_PATH
    NO_DEFAULT_PATH
  )

  if( llvm_profdata_exe )
    set( ${out_var} ${llvm_profdata_exe} PARENT_SCOPE )
  else()
    message( FATAL_ERROR "Unable to find llvm-profdata executable, which is required for PGO. More info at https://clang.llvm.org/docs/UsersManual.html#profiling-with-instrumentation")
  endif()
endfunction()

# NOTE: These aren't needed if MinGW Clang's libunwind.dll and libc++.dll are exposed by the system PATH.
# The same could probably be done for MinGW GCC as well, however I've decided not to do this for now.
# I'm leaving functions in because they might be useful later if a packaging step is created which
# puts standard library DLLs into the program distribution. However, I don't know enough to know whether
# or not that is a bad idea. More information is needed.
# For now, there will just be a "please ensure the bin/ directory is exposed in the system PATH" on Windows.

# function( try_copy_llvm_libunwind_dll )
#   _pgo_get_compiler_dirs( compiler_dirs )
#   set( libunwind_file_name "libunwind.dll" )

#   find_file( llvm_libunwind_dll
#     NAMES ${libunwind_file_name}
#     DOC "When using MinGW Clang on Windows, LLVM's libunwind.dll is required for PGO profile runs to work."
#     PATHS ${compiler_dirs}
#     DOC "When using MinGW Clang on Windows, LLVM's libunwind.dll is required for PGO profile runs to work."
#     NO_CMAKE_ENVIRONMENT_PATH
#     NO_SYSTEM_ENVIRONMENT_PATH
#     NO_CMAKE_SYSTEM_PATH
#     NO_DEFAULT_PATH
#   )

#   if( llvm_libunwind_dll )
#     add_custom_target( _pgo_copy_libunwind_dll ALL )
#     add_custom_command(
#       TARGET _pgo_copy_libunwind_dll
#       PRE_BUILD
#       COMMAND
#         ${CMAKE_COMMAND} -E copy ${llvm_libunwind_dll} "${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${libunwind_file_name}"
#     )
#   endif()
# endfunction()

# function( try_copy_llvm_libcpp_dll )
#   _pgo_get_compiler_dirs( compiler_dirs )
#   set( libcpp_file_name "libc++.dll" )

#   find_file( llvm_libcpp_dll
#     NAMES ${libcpp_file_name}
#     PATHS ${compiler_dirs}
#     DOC "When using MinGW Clang on Windows, LLVM's libc++.dll is required for PGO profile runs to work."
#     NO_CMAKE_ENVIRONMENT_PATH
#     NO_SYSTEM_ENVIRONMENT_PATH
#     NO_CMAKE_SYSTEM_PATH
#     NO_DEFAULT_PATH
#   )

#   if( llvm_libcpp_dll )
#     add_custom_target( _pgo_copy_llvm_libcpp_dll ALL )
#     add_custom_command(
#       TARGET _pgo_copy_llvm_libcpp_dll
#       PRE_BUILD
#       COMMAND
#         ${CMAKE_COMMAND} -E copy ${llvm_libcpp_dll} "${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${libcpp_file_name}"
#     )
#   endif()
# endfunction()
