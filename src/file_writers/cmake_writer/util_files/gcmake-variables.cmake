# ==================================================
# Compiler usage variables
# ==================================================
if( "${CMAKE_C_COMPILER_ID}" MATCHES "GNU" OR "${CMAKE_CXX_COMPILER_ID}" MATCHES "GNU" )
  set( USING_GCC TRUE )
else()
  set( USING_GCC FALSE )
endif()

if( "${CMAKE_C_COMPILER_ID}" MATCHES "Clang" OR "${CMAKE_CXX_COMPILER_ID}" MATCHES "Clang" )
  set( USING_CLANG TRUE )
else()
  set( USING_CLANG FALSE )
endif()

set( USING_MSVC ${MSVC} )
set( USING_MINGW ${MINGW} )

# ==================================================
# Host (current) system variables
# ==================================================
if( CMAKE_HOST_UNIX AND NOT CMAKE_HOST_APPLE )
  set( CURRENT_SYSTEM_IS_LINUX TRUE )
else()
  set( CURRENT_SYSTEM_IS_LINUX FALSE )
endif()

set( CURRENT_SYSTEM_IS_WINDOWS ${CMAKE_HOST_WIN32} )
set( CURRENT_SYSTEM_IS_MACOS ${CMAKE_HOST_APPLE} )

set( CURRENT_SYSTEM_IS_UNIX ${CMAKE_HOST_UNIX} )

# ==================================================
# Target system variables
# ==================================================
if( UNIX AND NOT APPLE )
  set( TARGET_SYSTEM_IS_LINUX TRUE )
else()
  set( TARGET_SYSTEM_IS_LINUX FALSE )
endif()

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
