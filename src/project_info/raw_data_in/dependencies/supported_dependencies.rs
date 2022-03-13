pub const DEPENDENCIES_YAML_STRING: &'static str = r#"
---
# TODO: In build.rs (build time before program compilation),
# turn this into a string in src/data_types/supported_dependencies.rs.
# Serde will read it at runtime and use this configuration to help write and
# validate these predefined dependencies.

as_subdirectory:
  SFML:
    git_repo:
      repo_url: git@github.com:SFML/SFML.git
    namespace_config:
      used_in_cmake_yaml: SFML
      cmakelists_linking: sfml-
    target_names:
      - system
      - window
      - network
      - graphics
      - audio
      # sfml-main is only available when building for windows
      - main

  nlohmann_json:
    git_repo:
      repo_url: git@github.com:ArthurSonzogni/nlohmann_json_cmake_fetchcontent.git
    namespace_config:
      used_in_cmake_yaml: nlohmann_json
      cmakelists_linking: "nlohmann_json::"
    target_names:
      - nlohmann_json

  fmt:
    git_repo:
      repo_url: git@github.com:fmtlib/fmt.git
    namespace_config:
      used_in_cmake_yaml: fmt
      cmakelists_linking: "fmt::"
    target_names:
      - fmt

  # JUCE:
  #   find_package_config:
  #     targets_retain_namespace: true
  #   git_repo:
  #     repo_url: git@github.com:juce-framework/JUCE.git
  #     latest_stable_release_tag: "6.1.2"
  #   namespace_prefix: juce
  #   target_names:
  #     - juce_analytics
  #     - juce_audio_basics
  #     - juce_audio_devices
  #     - juce_audio_formats
  #     - juce_audio_plugin_client
  #     - juce_audio_processors
  #     - juce_audio_utils
  #     - juce_box2d
  #     - juce_core
  #     - juce_cryptography
  #     - juce_data_structures
  #     - juce_dsp
  #     - juce_events
  #     - juce_graphics
  #     - juce_gui_basics
  #     - juce_gui_extra
  #     - juce_opengl
  #     - juce_osc
  #     - juce_product_unlocking
  #     - juce_video

# prewritten_find_modules:
  # SDL:
  #   git_repo:
  #     repo_url: git@github.com:libsdl-org/SDL.git
# non_subdirectory_cmake_projects:
"#;
