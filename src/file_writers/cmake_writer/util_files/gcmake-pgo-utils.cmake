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
      add_compile_options( 
        # MSVC generation step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},PROFILE_GENERATION>:/GL;/LTCG;/GENPROFILE>"
        # MSVC usage step
        "$<$<STREQUAL:${GCMAKE_PGO_STEP},USE_PROFILES>:/GL;/LTCG/USEPROFILE>"
      )
    endif()

    set( PGO_DEFAULTS_INITIALIZED TRUE )
    message( STATUS "PGO Step: ${GCMAKE_PGO_STEP}")
  endif()
endmacro()
