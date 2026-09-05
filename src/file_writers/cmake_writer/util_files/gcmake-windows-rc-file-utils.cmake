# It's hard to find good reading material on using .rc files with CMake, but this answer has some good info.
# https://stackoverflow.com/questions/68517552/how-to-add-icon-to-a-qt-application-on-windows-using-a-rc-file-on-a-cmake-proje

if( NOT GCMAKE_WINDOWS_RC_UTIL_CONFIG_HAS_RUN )
  if( TARGET_SYSTEM_IS_WINDOWS )
    # Windows configuration files (.rc) are built, then linked as part of an executable program.
    # An example use case is setting the icon of an exe file.
    enable_language( RC )
  endif()
  set( GCMAKE_WINDOWS_RC_UTIL_CONFIG_HAS_RUN TRUE )
endif()

function( generate_rc_file_for_windows_exe
  target_name
)
  gcmake_unaliased_target_name( ${target_name} TARGET_BASE_NAME )
  if( TARGET_SYSTEM_IS_WINDOWS AND NOT ${TARGET_BASE_NAME}_RC_ALREADY_GENERATED )
    set( optionalOneValueArgs ICON_PATH )
    cmake_parse_arguments( PARSE_ARGV 1 RC_CONFIG "" "${optionalOneValueArgs}" "" )

    set( RC_FILE_CONTENT )
    string( MAKE_C_IDENTIFIER "${TARGET_BASE_NAME}" useable_target_name )

    if( RC_CONFIG_ICON_PATH )
      # For a target named my-test, this would generate the line:
      # my_testIcon ICON "C:\Path\to_icon.ico"
      # I think my_testIcon is an identifier which can be used in windows GUI apps, but I'm not sure yet.
      string( APPEND RC_FILE_CONTENT "${useable_target_name}Icon ICON \"${RC_CONFIG_ICON_PATH}\"\n" )
    endif()

    # Every Windows executable gets an application manifest. GUI toolkits such as
    # wxWidgets import functions which only exist in ComCtl32 version 6, and
    # without a manifest asking for it the loader binds to the ancient 5.82
    # version shipped for compatibility. The process then fails to start at all,
    # with STATUS_ENTRYPOINT_NOT_FOUND (0xC0000139). The remaining settings are
    # harmless for console programs and useful for every executable:
    #   - supportedOS: without a compatibility section Windows runs the process in
    #     Vista compatibility mode (version lies, legacy behaviours).
    #   - dpiAwareness / dpiAware: per-monitor DPI on Windows 10 1607+ / 8.1,
    #     system DPI before that. Inert without windows.
    #   - longPathAware: file APIs accept paths longer than MAX_PATH (when the
    #     system policy allows it).
    #   - activeCodePage UTF-8: argv, narrow file APIs and stdio use UTF-8
    #     (Windows 10 1903+; ignored, harmlessly, before that).
    # Microsoft's convention for the assembly name is Organization.Division.Name.
    set( MANIFEST_FILE_PATH "${CMAKE_BINARY_DIR}/generated_windows_rc_files/${TARGET_BASE_NAME}.manifest" )
    string( MAKE_C_IDENTIFIER "${PROJECT_VENDOR}" manifest_vendor )
    string( MAKE_C_IDENTIFIER "${LOCAL_TOPLEVEL_PROJECT_NAME}" manifest_project )
    if( NOT manifest_vendor )
      set( manifest_vendor "Unknown" )
    endif()

    file( WRITE
      "${MANIFEST_FILE_PATH}"
      "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
<assembly xmlns=\"urn:schemas-microsoft-com:asm.v1\" manifestVersion=\"1.0\">
  <assemblyIdentity type=\"win32\" name=\"${manifest_vendor}.${manifest_project}.${useable_target_name}\" version=\"1.0.0.0\" processorArchitecture=\"*\"/>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type=\"win32\" name=\"Microsoft.Windows.Common-Controls\" version=\"6.0.0.0\" processorArchitecture=\"*\" publicKeyToken=\"6595b64144ccf1df\" language=\"*\"/>
    </dependentAssembly>
  </dependency>
  <compatibility xmlns=\"urn:schemas-microsoft-com:compatibility.v1\">
    <application>
      <!-- Windows 10 and Windows 11 -->
      <supportedOS Id=\"{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}\"/>
      <!-- Windows 8.1 -->
      <supportedOS Id=\"{1f676c76-80e1-4239-95bb-83d0f6d0da78}\"/>
      <!-- Windows 8 -->
      <supportedOS Id=\"{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}\"/>
      <!-- Windows 7 -->
      <supportedOS Id=\"{35138b9a-5d96-4fbd-8e2d-a2440225f93a}\"/>
    </application>
  </compatibility>
  <application xmlns=\"urn:schemas-microsoft-com:asm.v3\">
    <windowsSettings>
      <dpiAware xmlns=\"http://schemas.microsoft.com/SMI/2005/WindowsSettings\">true/pm</dpiAware>
      <dpiAwareness xmlns=\"http://schemas.microsoft.com/SMI/2016/WindowsSettings\">permonitorv2,permonitor</dpiAwareness>
      <longPathAware xmlns=\"http://schemas.microsoft.com/SMI/2016/WindowsSettings\">true</longPathAware>
      <activeCodePage xmlns=\"http://schemas.microsoft.com/SMI/2019/WindowsSettings\">UTF-8</activeCodePage>
    </windowsSettings>
  </application>
</assembly>
"
    )

    # An executable can only hold one manifest resource, and link.exe and lld-link
    # always embed one of their own (CMake passes /MANIFEST:EMBED, or the Visual
    # Studio generator's GenerateManifest). If the generated .rc also carried the
    # manifest, both would claim resource type 24 ID 1 and the link would fail with
    # "CVT1100 duplicate resource type:MANIFEST" (MSVC) or "duplicate resource: type
    # MANIFEST" (lld-link). So how the manifest gets into the exe depends on the linker:
    #   - MSVC-style linkers (MSVC itself, and Clang targeting the MSVC ABI through
    #     either driver): the .manifest file is added as a target source. CMake hands
    #     .manifest sources to the linker's manifest tool, which merges them with the
    #     linker's own manifest, so the exe ends up with a single manifest containing
    #     both our settings and the linker's UAC requestedExecutionLevel.
    #   - GNU ld (MinGW): the linker embeds nothing, and CMake ignores .manifest sources
    #     for it, so the .rc file references the manifest as resource type 24
    #     (RT_MANIFEST) with ID 1 (CREATEPROCESS_MANIFEST_RESOURCE_ID). The numbers are
    #     used directly because the generated .rc includes no header to name them.
    if( MSVC OR "${CMAKE_CXX_SIMULATE_ID}" STREQUAL "MSVC" OR "${CMAKE_C_SIMULATE_ID}" STREQUAL "MSVC" )
      target_sources( ${TARGET_BASE_NAME} PRIVATE "${MANIFEST_FILE_PATH}" )
    else()
      string( APPEND RC_FILE_CONTENT "1 24 \"${MANIFEST_FILE_PATH}\"\n" )
    endif()

    # Only write and attach the .rc file when it has something in it. For an MSVC
    # executable without an icon, the manifest is passed to the linker directly and
    # the .rc would be empty.
    if( RC_FILE_CONTENT )
      set( RC_FILE_PATH "${CMAKE_BINARY_DIR}/generated_windows_rc_files/${TARGET_BASE_NAME}.rc" )

      file( WRITE
        "${RC_FILE_PATH}"
        "${RC_FILE_CONTENT}"
      )

      # This file path doesn't need a 'windows-only' generator expression because this
      # function body is only run when the target system is Windows anyways.
      target_sources( ${TARGET_BASE_NAME} PRIVATE "${RC_FILE_PATH}" )
    endif()

    set( ${TARGET_BASE_NAME}_RC_ALREADY_GENERATED TRUE )
  endif()
endfunction()