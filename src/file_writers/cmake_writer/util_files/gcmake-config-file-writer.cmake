function( gcmake_config_file_add_contents
  contents
)
  string( APPEND CONFIG_FILE_CONTENTS "${contents}\n" )
  set( CONFIG_FILE_CONTENTS "${CONFIG_FILE_CONTENTS}" PARENT_SCOPE )
endfunction()

macro( gcmake_begin_config_file )
  set( CONFIG_FILE_CONTENTS "@PACKAGE_INIT@\n" )
  gcmake_config_file_add_contents("set( SAVED_CMAKE_MODULE_PATH $\{CMAKE_MODULE_PATH\} )")
  gcmake_config_file_add_contents("set( CMAKE_MODULE_PATH \"$\{CMAKE_CURRENT_LIST_DIR\}/modules\" )")
  gcmake_config_file_add_contents("include( CMakeFindDependencyMacro )")
endmacro()

macro( gcmake_end_config_file )
  gcmake_config_file_add_contents("include( \"$\{CMAKE_CURRENT_LIST_DIR\}/${LOCAL_TOPLEVEL_PROJECT_NAME}Targets.cmake\" )")
  gcmake_config_file_add_contents("set( CMAKE_MODULE_PATH $\{SAVED_CMAKE_MODULE_PATH\} )")
  file( WRITE "${CMAKE_CURRENT_BINARY_DIR}/Config.cmake.in" "${CONFIG_FILE_CONTENTS}" )
endmacro()
