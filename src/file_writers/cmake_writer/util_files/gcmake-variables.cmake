# ==================================================
# Compiler usage variables
# ==================================================
if( "${CMAKE_C_COMPILER_ID}" MATCHES "GNU" OR "${CMAKE_CXX_COMPILER_ID}" MATCHES "GNU" )
  set( USING_GCC TRUE )
else()
  set( USING_GCC FALSE )
endif()

if( CMAKE_SYSTEM_NAME STREQUAL "Emscripten" )
  set( USING_EMSCRIPTEN TRUE )
else()
  set( USING_EMSCRIPTEN FALSE )
endif()

# This will be TRUE when using 'zig c++' as well, since Zig uses clang internally for that.
if( NOT USING_EMSCRIPTEN AND ("${CMAKE_C_COMPILER_ID}" MATCHES "Clang" OR "${CMAKE_CXX_COMPILER_ID}" MATCHES "Clang") )
  set( USING_CLANG TRUE )
else()
  set( USING_CLANG FALSE )
endif()

set( USING_MSVC ${MSVC} )
set( USING_MINGW ${MINGW} )

# ==================================================
# Host (current) system variables
# ==================================================
set( CURRENT_SYSTEM_IS_LINUX ${CMAKE_HOST_LINUX} )
set( CURRENT_SYSTEM_IS_WINDOWS ${CMAKE_HOST_WIN32} )
set( CURRENT_SYSTEM_IS_MACOS ${CMAKE_HOST_APPLE} )

set( CURRENT_SYSTEM_IS_UNIX ${CMAKE_HOST_UNIX} )

# ==================================================
# Target system variables
# ==================================================
set( TARGET_SYSTEM_IS_LINUX ${LINUX} )
set( TARGET_SYSTEM_IS_WINDOWS ${WIN32} )
set( TARGET_SYSTEM_IS_MACOS ${APPLE} )

set( TARGET_SYSTEM_IS_UNIX ${UNIX} )
set( TARGET_SYSTEM_IS_ANDROID ${ANDROID} )

# ==================================================
# GCMake Internal Configuration variables
# ==================================================
if( CURRENT_SYSTEM_IS_WINDOWS )
  cmake_path( CONVERT "$ENV{USERPROFILE}" TO_CMAKE_PATH_LIST USER_HOME_DIR )
else()
  set( USER_HOME_DIR "$ENV{HOME}" )
endif()

set( GCMAKE_CONFIG_DIR "${USER_HOME_DIR}/.gcmake" )
set( GCMAKE_DEP_CACHE_DIR "${GCMAKE_CONFIG_DIR}/dep-cache" )
