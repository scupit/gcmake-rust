macro( initialize_pgo_defaults )
  if( NOT PGO_DEFAULTS_INITIALIZED )
    set( VALID_PGO_STEP_VALUES NONE PROFILE_GENERATION USE_PROFILES )
    set( GCMAKE_PGO_STEP "NONE" CACHE STRING "The current step in the profile-guided optimization process. To turn off (not use) PGO, set this to NONE" )
    set_property( CACHE GCMAKE_PGO_STEP PROPERTY STRINGS ${VALID_PGO_STEP_VALUES} )

    list( FIND VALID_PGO_STEP_VALUES "${GCMAKE_PGO_STEP}" IS_PGO_STEP_VALID )

    if( IS_PGO_STEP_VALID LESS 0 )
      message( FATAL_ERROR "Invalid PGO step '${GCMAKE_PGO_STEP}' given. Must be one of: ${VALID_PGO_STEP_VALUES}")
    else()
      # if( USING_MINGW )
      #   message( WARNING "Warning: MinGW PGO is currently ")
      # endif()
    endif()

    if( USING_GCC )
      if( GCMAKE_PGO_STEP STREQUAL "PROFILE_GENERATION" )
        link_libraries(gcov)
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
    elseif( USING_CLANG AND NOT USING_MINGW )
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
        set( MERGED_PROFILES_FILE "${PROFILES_DIR}/default.profdata")

        add_custom_target( merge_clang_profile_data ALL
          DEPENDS "${PROFILES_DIR}"
        )
        
        add_custom_command(
          TARGET merge_clang_profile_data
          PRE_BUILD
          COMMAND
            llvm-profdata merge "-output=${MERGED_PROFILES_FILE}" "${PROFILES_DIR}/*.profraw"
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
  )

  if( msvc_pgort140_dll )
    add_custom_target( copy_pgort_dll ALL )
    add_custom_command(
      TARGET copy_pgort_dll
      PRE_BUILD
      COMMAND
        ${CMAKE_COMMAND} -E copy ${msvc_pgort140_dll} "${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${the_pgo_file_name}"
    )
  else()
    message( FATAL_ERROR "Unable to find MSVC's pgort140.dll, which is required for running PGO instrumented binaries." )
  endif()
endfunction()
