# Should only be called from the root project, and only from the toplevel project being built.
# if( "${CMAKE_SOURCE_DIR}" STREQUAL "${CMAKE_CURRENT_SOURCE_DIR}" )

# This file should be included after installation utils and variable-utils.

function( gcmake_configure_cpack )
  include( CPackComponent )
  include( ProcessorCount )
  ProcessorCount( num_cpu_cores )

  # 0 when cannot determine
  if( NOT num_cpu_cores )
    set( num_cpu_cores 1 )
  endif()

  set( CPACK_NUM_PACKAGER_THREADS ${num_cpu_cores} CACHE STRING "Number of threads to use for CPack jobs" )

  set( CPACK_THREADS ${CPACK_NUM_PACKAGER_THREADS} )
  set( CPACK_ARCHIVE_THREADS ${CPACK_NUM_PACKAGER_THREADS} )

  set( requiredOneValueArgs VENDOR INSTALLER_TITLE INSTALLER_DESCRIPTION INSTALLER_EXE_PREFIX PROJECT_COMPONENT )
  set( optionalOneValueArgs SHORTCUT_MAP )
  cmake_parse_arguments( PARSE_ARGV 0 INSTALLER_CONFIG "" "${requiredOneValueArgs};${optionalOneValueArgs}" "" )

  foreach( required_arg IN LISTS requiredOneValueArgs )
    if( NOT INSTALLER_CONFIG_${required_arg} )
      message( FATAL_ERROR "${required_arg} is required by gcmake_configure_cpack(...), but wasn't passed.")
    endif()
  endforeach()

  if( USING_EMSCRIPTEN )
    # We need file stripping to be off when using Emscripten, otherwise WIX installer generation
    # will fail due to llvm-strip failing to strip the .html files.
    set( CPACK_STRIP_FILES_DEFAULT_VALUE OFF )
  else()
    set( CPACK_STRIP_FILES_DEFAULT_VALUE ON )
  endif()

  option( CPACK_STRIP_FILES "Whether to strip symbols from installed binaries" ${CPACK_STRIP_FILES_DEFAULT_VALUE} )

  if( INSTALLER_CONFIG_SHORTCUT_MAP )
    set( CPACK_PACKAGE_EXECUTABLES ${INSTALLER_CONFIG_SHORTCUT_MAP} )
    set( CPACK_CREATE_DESKTOP_LINKS )
    set( temp_should_add TRUE )

    foreach( item IN LISTS CPACK_PACKAGE_EXECUTABLES )
      if( temp_should_add )
        # The shortcuts are specified in a flat list formatted as target1;shortcut-name1;target2;shortcut-name2.
        # this extracts and adds only the target names to CPACK_CREATE_DESKTOP_LINKS.
        list( APPEND CPACK_CREATE_DESKTOP_LINKS ${item} )
        set( temp_should_add FALSE )
      else()
        set( temp_should_add TRUE )
      endif()
    endforeach()
  endif()

  get_installer_compatible_license( license_file )

  if( license_file )
    message( STATUS "Found valid license file" )
    set( CPACK_RESOURCE_FILE_LICENSE "${license_file}" )
  else()
    message( STATUS "No license file found for ${LOCAL_TOPLEVEL_PROJECT_NAME}" )
  endif()

  # https://gitlab.kitware.com/cmake/cmake/-/issues/20177
  # https://cmake.org/cmake/help/latest/module/CPackComponent.html
  get_cmake_property( CPACK_COMPONENTS_ALL COMPONENTS )
  # message( "components: ${CPACK_COMPONENTS_ALL}" )

  # For some reason, the Kokkos library installs its targets both with and without
  # specifying the 'Kokkos' COMPONENT. This causes the WIX generator to fail with
  # a "duplicate GUID for file..." error. Everything other than the Kokkos targets
  # are installed without specifying a COMPONENT as well. As a result, we can tell
  # CPack to just omit the 'Kokkos' component so that only a single copy of each Kokkos
  # library is installed. This hack fixes the WIX issue, although it should probably be moved
  # into the Kokkos pre_load or post_load script somehow. For now, this is fine.
  # TODO: Move this into a pre_load or post_load script if it causes issues.
  if( "Kokkos" IN_LIST CPACK_COMPONENTS_ALL )
    list( REMOVE_ITEM CPACK_COMPONENTS_ALL "Kokkos" )
  endif()

  set( DEP_COMPONENT_LIST ${CPACK_COMPONENTS_ALL} )
  list( REMOVE_ITEM DEP_COMPONENT_LIST ${INSTALLER_CONFIG_PROJECT_COMPONENT} )

  cpack_add_component( ${INSTALLER_CONFIG_PROJECT_COMPONENT}
    DISPLAY_NAME "Libraries and executables"
    DESCRIPTION "All programs build by ${INSTALLER_CONFIG_INSTALLER_TITLE}"
    DEPENDS ${DEP_COMPONENT_LIST}
  )

  foreach( dep_component_name IN LISTS DEP_COMPONENT_LIST )
    cpack_add_component( ${dep_component_name}
      DEPENDS ${INSTALLER_CONFIG_PROJECT_COMPONENT}
      HIDDEN
    )
  endforeach()

  set( CPACK_GENERATOR )
  set( CPACK_SOURCE_GENERATOR )

  locate_7zip_exe( GCMAKE_exe_7zip )

  if( GCMAKE_exe_7zip )
    set( ENABLE_7ZIP_DEFAULT ON )
  else()
    set( ENABLE_7ZIP_DEFAULT OFF )
  endif()

  option( CPACK_7Z_ENABLED "Enable .7z package generator" ${ENABLE_7ZIP_DEFAULT} )

  # The 7Z CPACK_GENERATOR will be enabled for windows only. However, it's fine to create
  # a .7z source package from any system.
  if( CPACK_7Z_ENABLED )
    list( APPEND CPACK_SOURCE_GENERATOR "7Z" )
  endif()

  # Currently don't support Apple because I have no way to test it.
  if( CURRENT_SYSTEM_IS_WINDOWS )
    locate_wix_candle_exe( GCMAKE_wix_candle_exe )
    if( GCMAKE_wix_candle_exe )
      set( WIX_ENABLED_BY_DEFAULT ON )
    else()
      set( WIX_ENABLED_BY_DEFAULT OFF )
    endif()

    option( CPACK_WIX_ENABLED "Generate installer using WiX" ${WIX_ENABLED_BY_DEFAULT} )

    locate_nsis_makensis_exe( GCMAKE_nsis_makensis_exe )
    if( GCMAKE_nsis_makensis_exe )
      set( NSIS_ENABLED_BY_DEFAULT ON )
    else()
      set( NSIS_ENABLED_BY_DEFAULT OFF )
    endif()

    option( CPACK_NSIS_ENABLED "Generate installer using NSIS" ${NSIS_ENABLED_BY_DEFAULT} )
    option( CPACK_ZIP_ENABLED "Enable .zip package generator" ON )

    if( CPACK_7Z_ENABLED )
      list( APPEND CPACK_GENERATOR "7Z" )
    endif()

    if( CPACK_ZIP_ENABLED )
      list( APPEND CPACK_GENERATOR "ZIP" )
      list( APPEND CPACK_SOURCE_GENERATOR "ZIP" )
    endif()

    # TODO: Icons and banners in the installers themselves
    if( CPACK_WIX_ENABLED )
      list( APPEND CPACK_GENERATOR "WIX" )

      set( CPACK_WIX_ROOT_FEATURE_TITLE "${INSTALLER_CONFIG_INSTALLER_TITLE}" )
      set( CPACK_WIX_ROOT_FEATURE_DESCRIPTION "${INSTALLER_CONFIG_INSTALLER_DESCRIPTION}" )

      configure_custom_wix_template(
        "${INSTALLER_CONFIG_INSTALLER_TITLE}"
        wix_custom_template_file
      )

      set( CPACK_WIX_TEMPLATE "${wix_custom_template_file}" )
    endif()

    if( CPACK_NSIS_ENABLED )
      list( APPEND CPACK_GENERATOR "NSIS64" )
      set( CPACK_NSIS_ENABLE_UNINSTALL_BEFORE_INSTALL ON )
      set( CPACK_NSIS_MODIFY_PATH ON )
      set( CPACK_NSIS_DISPLAY_NAME "${INSTALLER_CONFIG_INSTALLER_TITLE}" )
      set( CPACK_NSIS_PACKAGE_NAME "${INSTALLER_CONFIG_INSTALLER_TITLE}" )
      set( CPACK_NSIS_WELCOME_TITLE "Welcome to ${INSTALLER_CONFIG_INSTALLER_TITLE} Setup" )
      set( CPACK_NSIS_UNINSTALL_NAME "Uninstall ${INSTALLER_CONFIG_INSTALLER_TITLE}" )
    endif()
  elseif( CURRENT_SYSTEM_IS_LINUX )
    locate_dpkg_exe( GCMAKE_dpkg_exe )
    if( GCMAKE_dpkg_exe )
      set( DEB_ENABLED_BY_DEFAULT ON )
    else()
      set( DEB_ENABLED_BY_DEFAULT OFF )
    endif()

    option( CPACK_DEB_ENABLED "Generate DEB installer" ${DEB_ENABLED_BY_DEFAULT} )
    option( CPACK_RPM_ENABLED "Generate RPM installer" OFF )
    option( CPACK_TGZ_ENABLED "Enable tar.gz package generator" ON )
    option( CPACK_TXZ_ENABLED "Enable tar.xz package generator" ON )
    option( CPACK_FreeBSD_ENABLED "Generate FreeBSD installer" OFF )

    if( CPACK_TGZ_ENABLED )
      list( APPEND CPACK_GENERATOR "TGZ" )
      list( APPEND CPACK_SOURCE_GENERATOR "TGZ" )
    endif()

    if( CPACK_TXZ_ENABLED )
      list( APPEND CPACK_GENERATOR "TXZ" )
      list( APPEND CPACK_SOURCE_GENERATOR "TXZ" )
    endif()

    if( CPACK_DEB_ENABLED )
      list( APPEND CPACK_GENERATOR "DEB" )
      set( CPACK_DEBIAN_PACKAGE_MAINTAINER "${INSTALLER_CONFIG_VENDOR}" )
      string( REPLACE ";" "," USABLE_DEB_DEPENDENCY_LIST "${MY_NEEDED_DEB_PACKAGES}")
      set( CPACK_DEBIAN_PACKAGE_DEPENDS ${USABLE_DEB_DEPENDENCY_LIST} )
    endif()

    if( CPACK_RPM_ENABLED )
      list( APPEND CPACK_GENERATOR "RPM" )
    endif()

    if( CPACK_FreeBSD_ENABLED )
      list( APPEND CPACK_GENERATOR "FreeBSD" )
    endif()
  endif()

  set( CPACK_PACKAGE_VENDOR "${INSTALLER_CONFIG_VENDOR}" )
  set( CPACK_SOURCE_IGNORE_FILES "/\\\\.git/" "/\\\\.cache/" "/\\\\.vscode/" "/build/" "/dep/" "/__pycache__/" "/\\\\.mypy_cache/" )
  set( CPACK_PACKAGE_DESCRIPTION ${INSTALLER_CONFIG_INSTALLER_DESCRIPTION} )
  set( CPACK_PACKAGE_NAME ${INSTALLER_CONFIG_INSTALLER_EXE_PREFIX} )
  set( CPACK_PACKAGE_DIRECTORY packaged )

  set( AVAILABLE_CHECKSUM_ALGORITHMS SHA256 SHA512 )

  set( CPACK_USING_CHECKSUM_ALGORITHM SHA256 CACHE STRING "Algorithm used to generate package checksums" )
  set_property( CACHE CPACK_USING_CHECKSUM_ALGORITHM PROPERTY STRINGS ${AVAILABLE_CHECKSUM_ALGORITHMS} )

  set( CPACK_PACKAGE_CHECKSUM ${CPACK_USING_CHECKSUM_ALGORITHM} )

  include( CPack )
endfunction()

function( find_license_and_readme_files
  license_file_out
  readme_file_out
)
  set( license_names )
  set( readme_names )

  foreach( readme_prefix IN ITEMS "readme" "README" "Readme" )
    foreach( readme_extension IN ITEMS ".md" ".txt")
      list( APPEND readme_names "${readme_prefix}${readme_extension}")
    endforeach()
  endforeach()

  foreach( license_prefix IN ITEMS "LICENSE" "license" "License" )
    foreach( license_extension IN ITEMS "" ".md" ".txt")
      list( APPEND license_names "${license_prefix}${license_extension}")
    endforeach()
  endforeach()

  find_file( GCMAKE_license_file
    NAMES ${license_names}
    PATHS "${TOPLEVEL_PROJECT_DIR}"
    NO_CMAKE_ENVIRONMENT_PATH
    NO_CMAKE_FIND_ROOT_PATH
    NO_CMAKE_PATH
    NO_CMAKE_SYSTEM_PATH
    NO_DEFAULT_PATH
    NO_SYSTEM_ENVIRONMENT_PATH
    NO_PACKAGE_ROOT_PATH
  )

  find_file( GCMAKE_readme_file
    NAMES ${readme_names}
    PATHS "${TOPLEVEL_PROJECT_DIR}"
    NO_CMAKE_ENVIRONMENT_PATH
    NO_CMAKE_FIND_ROOT_PATH
    NO_CMAKE_PATH
    NO_CMAKE_SYSTEM_PATH
    NO_DEFAULT_PATH
    NO_SYSTEM_ENVIRONMENT_PATH
    NO_PACKAGE_ROOT_PATH
  )

  set( ${license_file_out} "${GCMAKE_license_file}" PARENT_SCOPE )
  set( ${readme_file_out} "${GCMAKE_readme_file}" PARENT_SCOPE )
endfunction()

function( get_installer_compatible_license
  license_file_out
)
  find_license_and_readme_files(
    license_file
    _readme_file
  )
  set( ${license_file_out} "${license_file}" PARENT_SCOPE )

  if( license_file )
    cmake_path( GET license_file EXTENSION LAST_ONLY license_file_extension )
    if( "${license_file_extension}" STREQUAL ".md" )
      cmake_path( GET license_file STEM LAST_ONLY license_stem )

      set( usable_license_file_name "${license_stem}.txt")
      set( license_file_dir "${CMAKE_BINARY_DIR}/license_files/${LOCAL_TOPLEVEL_PROJECT_NAME}" )
      set( license_file_generated_path "${license_file_dir}/${usable_license_file_name}" )
      
      # This copy has to be done at configure time because the existence of the file is checked by
      # cpack at configure time.
      file( MAKE_DIRECTORY "${license_file_dir}" )
      file( COPY_FILE "${license_file}" "${license_file_generated_path}" ONLY_IF_DIFFERENT )

      set( ${license_file_out} "${license_file_generated_path}" PARENT_SCOPE )
    endif()
  endif()
endfunction()

function( locate_7zip_exe
  out_var
)
  find_program( GCMAKE_exe_7zip
    NAMES "7z" "7z.exe"
    PATH_SUFFIXES "7-Zip"
  )

  set( ${out_var} "${GCMAKE_exe_7zip}" PARENT_SCOPE )
endfunction()

function( locate_wix_candle_exe
  out_var
)
  find_program( GCMAKE_wix_candle_exe
    NAMES "candle.exe"
    PATH_SUFFIXES "wix311-binaries"
  )

  set( ${out_var} "${GCMAKE_wix_candle_exe}" PARENT_SCOPE )
endfunction()

function( locate_nsis_makensis_exe
  out_var
)
  find_program( GCMAKE_nsis_makensis_exe
    NAMES "makensis.exe"
    PATH_SUFFIXES "NSIS"
  )

  set( ${out_var} "${GCMAKE_nsis_makensis_exe}" PARENT_SCOPE )
endfunction()

function( locate_dpkg_exe
  out_var
)
  find_program( GCMAKE_dpkg_exe
    NAMES "dpkg"
  )

  set( ${out_var} "${GCMAKE_dpkg_exe}" PARENT_SCOPE )
endfunction()

# Creates a very slightly modified version of CMake's wix template:
# https://github.com/Kitware/CMake/blob/master/Utilities/Release/WiX/WIX.template.in
# where the installer title can be configured.
function( configure_custom_wix_template
  installer_title
  template_file_out
)
  set( CUSTOM_TEMPLATE_FILE_OUT "${CMAKE_BINARY_DIR}/WIX-CUSTOM.template" )

  # https://wixtoolset.org/documentation/manual/v3/xsd/wix/product.html
  file( WRITE "${CUSTOM_TEMPLATE_FILE_OUT}"
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>

    <?include \"cpack_variables.wxi\"?>

    <Wix xmlns=\"http://schemas.microsoft.com/wix/2006/wi\"
        RequiredVersion=\"3.6.3303.0\">

        <Product Id=\"$(var.CPACK_WIX_PRODUCT_GUID)\"
            Name=\"${installer_title}\"
            Language=\"1033\"
            Version=\"$(var.CPACK_PACKAGE_VERSION)\"
            Manufacturer=\"$(var.CPACK_PACKAGE_VENDOR)\"
            UpgradeCode=\"$(var.CPACK_WIX_UPGRADE_GUID)\">

            <Package
              InstallerVersion=\"301\"
              Compressed=\"yes\"
            />

            <Media Id=\"1\" Cabinet=\"media1.cab\" EmbedCab=\"yes\"/>

            <MajorUpgrade
                Schedule=\"afterInstallInitialize\"
                AllowSameVersionUpgrades=\"yes\"
                DowngradeErrorMessage=\"A later version of [ProductName] is already installed. Setup will now exit.\"/>

            <WixVariable Id=\"WixUILicenseRtf\" Value=\"$(var.CPACK_WIX_LICENSE_RTF)\"/>
            <Property Id=\"WIXUI_INSTALLDIR\" Value=\"INSTALL_ROOT\"/>

            <?ifdef CPACK_WIX_PRODUCT_ICON?>
            <Property Id=\"ARPPRODUCTICON\">ProductIcon.ico</Property>
            <Icon Id=\"ProductIcon.ico\" SourceFile=\"$(var.CPACK_WIX_PRODUCT_ICON)\"/>
            <?endif?>

            <?ifdef CPACK_WIX_UI_BANNER?>
            <WixVariable Id=\"WixUIBannerBmp\" Value=\"$(var.CPACK_WIX_UI_BANNER)\"/>
            <?endif?>

            <?ifdef CPACK_WIX_UI_DIALOG?>
            <WixVariable Id=\"WixUIDialogBmp\" Value=\"$(var.CPACK_WIX_UI_DIALOG)\"/>
            <?endif?>

            <FeatureRef Id=\"ProductFeature\"/>

            <UIRef Id=\"$(var.CPACK_WIX_UI_REF)\" />

            <?include \"properties.wxi\"?>
            <?include \"product_fragment.wxi\"?>
        </Product>
    </Wix>
    "
  )

  set( ${template_file_out} ${CUSTOM_TEMPLATE_FILE_OUT} PARENT_SCOPE )
endfunction()