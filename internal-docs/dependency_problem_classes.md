# Dependency Problem Classes

Every file in the [gcmake-dependency-configs](../gcmake-dependency-configs/) registry exists to
solve a specific, recurring problem. This catalog names those problems, gives the signal that tells
you a dependency has one, and points at the standard solution. It was derived from a full read of
all 40 configurations in the registry (August 2026).

**How to use this catalog:**

- **Analyzing a new dependency** ([Procedure A](dependency_config_procedures.md#procedure-a--analyze-the-dependency)):
  walk the detection signals; the classes that fire become your configuration's work list.
- **Reviewing a config change**: check the hooks present against the classes claimed. A hook that
  doesn't correspond to a class is suspect; a firing class with no corresponding config content is
  a gap.

Class IDs are stable kebab-case names (`PC-*`), grouped by lifecycle phase. Each entry links to the
[standards](dependency_config_standards.md) rules and [procedure templates](dependency_config_procedures.md#appendix-templates)
that implement its solution.

## Overview

| ID | Name | Phase | One-line signal |
| -- | ---- | ----- | --------------- |
| [PC-version-impedance](#pc-version-impedance) | Version & download impedance | Acquisition | Release URLs/tags don't follow a clean `vX.Y.Z` scheme |
| [PC-broken-upstream](#pc-broken-upstream) | Broken or unsuitable upstream | Acquisition | Upstream's build breaks when embedded, or no consumable project exists |
| [PC-identity-mismatch](#pc-identity-mismatch) | Identity mismatches | Acquisition | Installed package/include-dir name ≠ project name |
| [PC-target-surface](#pc-target-surface) | Target-surface modeling | Modeling | Real target names/flavors/components need a stable YAML surface |
| [PC-cross-dep-requires](#pc-cross-dep-requires) | Cross-dependency requirements | Modeling | The dependency needs targets from *other* registry dependencies |
| [PC-option-hygiene](#pc-option-hygiene) | Option hygiene | Import | Upstream `option()` defaults are wrong for embedded use |
| [PC-global-pollution](#pc-global-pollution) | Global-state pollution | Import | Adding the subdirectory changes state *outside* the dependency |
| [PC-hardcoded-placement](#pc-hardcoded-placement) | Hardcoded artifact placement | Import | Upstream pins its own output directories |
| [PC-lost-usage-reqs](#pc-lost-usage-reqs) | Lost usage requirements | Import | A define/flag consumers need doesn't propagate correctly |
| [PC-bundled-tooling](#pc-bundled-tooling) | Bundled tooling exposure | Import | The dep ships CMake helpers the generated code must `include()` |
| [PC-sibling-discovery](#pc-sibling-discovery) | Sibling-discovery bridging | Import | The dep `find_package`s something we're building, not installing |
| [PC-finder-strategy](#pc-finder-strategy) | Finder selection & authoring | Discovery | A system-installed dep needs to be located reliably |
| [PC-dll-distribution](#pc-dll-distribution) | Windows DLL distribution | Runtime | A shared library must be next to the exe on Windows |
| [PC-system-libs](#pc-system-libs) | Implicit system-library linkage | Runtime | Platform-specific linker errors (`m`, `dl`, `ws2_32`, pthread) |
| [PC-install-toggle](#pc-install-toggle) | Install-toggle wiring | Install | The dep has (or lacks) its own install on/off switch |
| [PC-emscripten](#pc-emscripten) | Emscripten & cross-compile posture | Cross-compile | The dep is an Emscripten port, works via emcc, or is incompatible |
| [PC-debian-packages](#pc-debian-packages) | Debian package metadata | Metadata | apt provides runtime/dev packages for the dep |
| [PC-human-channel](#pc-human-channel) | Manual-step documentation | Metadata | Something automation can't do must be told to a human |

---

## Group I — Acquisition & identity

### PC-version-impedance

**Phase:** Acquisition · **Solved in:** `dep_config.yaml` `download_info`

- **Detection signal:** Look at the project's releases page. Tags without a patch component
  (imgui `v1.88`, raylib `5.0`), zero-padded or concatenated versions (kokkos `4.1.00`, sqlite3
  `3410000`), URLs that change over time (sqlite3's per-year paths), unusual tag formats (freetype
  `VER-2-12-1`), or a huge git history that makes archives preferable (nlohmann_json).
- **Root cause:** GCMake wants users to write one uniform three-part `file_version`/`git_tag`; the
  ecosystem does whatever it wants.
- **Standard solution:** `version_transform` templates with `:N` zero-padding,
  `url_base_by_version` for time-varying URLs (largest key ≤ requested version wins), per-platform
  archive `extensions`, offering *both* git and URL methods when practical. When the scheme can't
  be expressed (freetype's `VER-` tags), fall back to git mode and document the tag format in the
  README ([PC-human-channel](#pc-human-channel)).
- **Exemplars:** [sqlite3](../gcmake-dependency-configs/sqlite3/dep_config.yaml) (year URLs +
  digit concatenation), [kokkos](../gcmake-dependency-configs/kokkos/dep_config.yaml),
  [imgui](../gcmake-dependency-configs/imgui/dep_config.yaml) (`v{{MAJOR}}.{{MINOR}}`),
  [nlohmann_json](../gcmake-dependency-configs/nlohmann_json/dep_config.yaml),
  [freetype](../gcmake-dependency-configs/freetype/README.md).
- **Status:** Partially blocked on tooling — raylib's 5.0 tag (no patch) needs range-based
  transforms (see [known issues](dependency_config_known_issues.md#tool-feature-gaps)).

### PC-broken-upstream

**Phase:** Acquisition · **Solved in:** `download_info.repo_url`, README, or a wrapper repository

- **Detection signal:** Empirical — the dependency fails when embedded in a superbuild, its
  latest release lacks something the config needs, or there is no CMake project to consume at all.
  Superbuild-only collisions (duplicate custom targets) are a subclass you only find by building
  multiple deps together.
- **Root cause:** Upstream doesn't test the embedded/subdirectory use case, or the "project" is
  really just a compiler/tool/source-drop.
- **Standard solution ladder** (least to most ownership):
  1. **Pin a branch/version and document why** — crow `master` (release needs Boost), yaml-cpp
     `master` (CMake bug #22909 hides `YAML_CPP_INSTALL` via `cmake_dependent_option` on the
     release branch), sfml `2.6.x` (respects `CMAKE_INSTALL_LIBDIR`), glm `>= 1.0.0`.
  2. **Fork and patch** — glm's unconditional `uninstall` target collided with GLFW's guarded one;
     `repo_url` points at the fork until upstream merges.
  3. **Author a wrapper repository** — cppfront-cmake-wrapper (build abstraction over a compiler
     repo, decoupling registry from upstream churn), gcmake-emscripten-compat (INTERFACE-target
     shim where no upstream project exists).
- **Exemplars:** [glm](../gcmake-dependency-configs/glm/dep_config.yaml),
  [cppfront](../gcmake-dependency-configs/cppfront/README.md),
  [crow](../gcmake-dependency-configs/crow/README.md),
  [yaml-cpp](../gcmake-dependency-configs/yaml-cpp/README.md),
  [emscripten](../gcmake-dependency-configs/emscripten/dep_config.yaml).
- **Status:** Permanent.

### PC-identity-mismatch

**Phase:** Acquisition · **Solved in:** `installed_include_dir_name`, `config_file_project_name`, `module_name`, `found_var`

- **Detection signal:** Install the dependency once and inspect the install tree: does the package
  config dir, include dir, or `find_package` name differ from the project/registry name?
- **Root cause:** Projects freely diverge between repo name, package name, target names, and
  include layout.
- **Standard solution:** Record each mismatch in the dedicated YAML field: nlohmann_json installs
  headers to `include/nlohmann` (`installed_include_dir_name`); glfw's installed config is `glfw3`
  (`config_file_project_name`); module deps record the exact `find_package` casing (`wxWidgets`,
  `CUDAToolkit`) and the module's real found-variable (`libwebsockets_FOUND`, `OPENGL_FOUND`).
- **Exemplars:** [nlohmann_json](../gcmake-dependency-configs/nlohmann_json/dep_config.yaml),
  [glfw](../gcmake-dependency-configs/glfw/dep_config.yaml),
  [lws](../gcmake-dependency-configs/lws/dep_config.yaml),
  [cuda](../gcmake-dependency-configs/cuda/dep_config.yaml).
- **Status:** Permanent.

## Group II — Target-surface modeling

### PC-target-surface

**Phase:** Modeling · **Solved in:** `target_configs`/`targets`/`components`, `namespace_config`, `mutually_exclusive`

- **Detection signal:** Enumerate what the dependency actually exports (`add_library`/ALIAS calls,
  or the installed `*Targets.cmake`): flavor targets (shared/static/with-main/without-main),
  component graphs, internal-only targets, un-namespaced names, prefix-style "namespaces".
- **Root cause:** GCMake needs one stable, lowercase, namespaced linking surface
  (`depname::target`) over an ecosystem of ad-hoc naming.
- **Standard solution:** `actual_target_name` maps YAML names to real names;
  `namespace_config.cmakelists_linking` handles true namespaces (`spdlog::`), prefixes (`sfml-`),
  and empty namespaces (glfw, freetype, raylib, lws); intra-dep `requires` encodes prerequisite
  graphs (wxWidgets components; sdl2 `main` requires `sdl2 or static`); `mutually_exclusive`
  encodes can't-link-both flavor sets; `((windows))` / `(( cuda ))` prefixes platform-gate
  individual targets; `or` alternatives express "any one satisfies" (no preference ordering).
  Deliberate *non-exposure* of internals is part of the model (kokkos exposes only the `kokkos`
  facade; internal component targets are recorded but commented out).
- **Judgment calls & variants:** Component libs with variable-style linking use
  `cmakelists_usage.link_format: Variable` plus `include(${wxWidgets_USE_FILE})` in post_load;
  glm exposes a `_private_header_only` pseudo-target solely so the compiled `glm` target's export
  set is valid (its dependency must be installed/exported too); pugixml exposes three flavor
  targets and uses pre_load to make them all actually exist
  (`PUGIXML_BUILD_SHARED_AND_STATIC_LIBS ON`); stb chooses one INTERFACE target *per header* for
  granularity.
- **Exemplars:** [wxwidgets](../gcmake-dependency-configs/wxwidgets/dep_config.yaml),
  [sdl2](../gcmake-dependency-configs/sdl2/dep_config.yaml),
  [pugixml](../gcmake-dependency-configs/pugixml/dep_config.yaml),
  [glm](../gcmake-dependency-configs/glm/dep_config.yaml),
  [kokkos](../gcmake-dependency-configs/kokkos/dep_config.yaml),
  [imgui](../gcmake-dependency-configs/imgui/dep_config.yaml) (the full system×API matrix).
- **Status:** Permanent.

### PC-cross-dep-requires

**Phase:** Modeling · **Solved in:** `external_requires`

- **Detection signal:** The dependency's docs or CMake `find_package`/`target_link_libraries`
  calls reference libraries that are themselves registry entries (fmt, openssl, zlib, glfw,
  opengl, threads, freetype, sdl2).
- **Root cause:** Dependencies have dependencies, and the registry must express those edges so the
  graph can order imports, validate presence, and auto-link.
- **Standard solution:** `external_requires` lists of single-namespace `dep::target` specs, with
  `or` for acceptable flavors (`sdl2::sdl2 or sdl2::static`). The referenced dep must exist in the
  registry (validated at load); the graph reports a clear error if the user's project doesn't
  import it. Often paired with a pre_load option that makes upstream *use* the external copy
  (`SPDLOG_FMT_EXTERNAL ON` — safe precisely because `external_requires` guarantees fmt is
  imported first).
- **Exemplars:** [spdlog](../gcmake-dependency-configs/spdlog/dep_config.yaml),
  [lws](../gcmake-dependency-configs/lws/dep_config.yaml) (openssl + threads),
  [glew](../gcmake-dependency-configs/glew/dep_config.yaml) (opengl),
  [raylib](../gcmake-dependency-configs/raylib/dep_config.yaml) (glfw + opengl),
  [imgui](../gcmake-dependency-configs/imgui/dep_config.yaml) (per-backend),
  [crow](../gcmake-dependency-configs/crow/dep_config.yaml).
- **Status:** Permanent — but crow shows the *feature-conditional* variant is blocked on
  predefined-dependency features (see [known issues](dependency_config_known_issues.md#tool-feature-gaps)):
  until then, crow force-enables all features and unconditionally requires asio+openssl+zlib.

## Group III — Import-time damage control (subdirectory deps)

### PC-option-hygiene

**Phase:** Import (pre_load) · **Solved in:** `pre_load.cmake`

- **Detection signal:** Read every `option()` / `cmake_dependent_option()` / cache `set()` in the
  upstream CMakeLists. Which defaults are wrong when the project is a silent build-tree guest?
  Typical offenders: tests, examples, docs, tools, formatting, packaging extras defaulting ON
  (often via a "master project" check that misfires or doesn't exist).
- **Root cause:** Upstream defaults target *their* developers, not embedders.
- **Standard solution:** Preempt in pre_load with non-forcing `option()`/`set(CACHE)` — pre_load
  runs before the subdirectory, and CMake's first-definition-wins semantics make this a default
  the user can still override. Categories seen: **disable dev noise** (catch2 ×3, ftxui ×2,
  yaml-cpp ×4, re2, glm tests, raylib examples), **enable integration** (`SPDLOG_FMT_EXTERNAL`,
  `PUGIXML_BUILD_SHARED_AND_STATIC_LIBS`, crow's feature list), **propagate project context**
  (glm's C++-standard options driven by `PROJECT_CXX_LANGUAGE_*_STANDARD` — available because
  language config is written before dependencies), **platform-conditional defaults**
  (`THREADS_PREFER_PTHREAD_FLAG` ON except on Windows), and **declare finder hints** for module
  deps (`ZSTD_ROOT`, `OPENSSL_USE_STATIC_LIBS`).
- **`FORCE` policy:** only when upstream force-sets or ignores non-forced values (glfw's
  docs/examples/tests, raylib's `PLATFORM Web` under Emscripten) — and never on generically-named
  variables without a comment explaining the risk (raylib's `BUILD_EXAMPLES` is deliberately *not*
  forced, with the reason written in place).
- **Exemplars:** [catch2](../gcmake-dependency-configs/catch2/pre_load.cmake),
  [glfw](../gcmake-dependency-configs/glfw/pre_load.cmake),
  [glm](../gcmake-dependency-configs/glm/pre_load.cmake),
  [raylib](../gcmake-dependency-configs/raylib/pre_load.cmake),
  [threads](../gcmake-dependency-configs/threads/pre_load.cmake).
- **Status:** Permanent.

### PC-global-pollution

**Phase:** Import (post_load) · **Solved in:** `post_load.cmake`

- **Detection signal:** Diff the CMake cache / global state before and after adding the
  subdirectory. Anything outside the dep's own namespace that changed — CPack variables, doc
  strings of shared cache variables, `CMAKE_*` globals — is pollution.
- **Root cause:** Upstream assumes it's the top-level project and configures global machinery.
  Note the trap: **our own install-var wiring can trigger it** — verified: setting
  `SPDLOG_INSTALL ON` makes spdlog `include(cmake/spdlogCPack.cmake)`, which sets
  `CPACK_GENERATOR` and `CPACK_PACKAGE_RELOCATABLE` as *CACHE* variables, permanently corrupting
  the consuming project's packaging.
- **Standard solution:** Undo it in post_load: `unset( <var> CACHE )` for injected cache entries
  (spdlog), or re-`set(... CACHE ... FORCE)` to restore clobbered state (sfml restoring the
  `CMAKE_BUILD_TYPE`/`BUILD_SHARED_LIBS` doc strings from
  `LOCAL_CMAKE_BUILD_TYPE_DOC_STRING`/`LOCAL_BUILD_SHARED_LIBS_DOC_STRING`).
- **Exemplars:** [spdlog](../gcmake-dependency-configs/spdlog/post_load.cmake),
  [sfml](../gcmake-dependency-configs/sfml/post_load.cmake).
- **Status:** Permanent.

### PC-hardcoded-placement

**Phase:** Import (post_load) · **Solved in:** `post_load.cmake`

- **Detection signal:** After a build, the dep's artifacts (especially DLLs) are not in
  `MY_RUNTIME_OUTPUT_DIR`/`MY_LIBRARY_OUTPUT_DIR`; on Windows, test/exe runs fail to find the
  dep's DLL. Grep upstream for `*_OUTPUT_DIRECTORY`.
- **Root cause:** Upstream pins output dirs. Verified: googletest's `internal_utils.cmake` sets
  all five `*_OUTPUT_DIRECTORY` properties to `${CMAKE_BINARY_DIR}/bin|lib` — but GCMake's world
  is `bin/$<CONFIG>`, so `gtest.dll` lands where test executables can't load it.
- **Standard solution:** post_load collects whichever of the dep's targets exist (guarded `TARGET`
  checks — target availability varies with options) and re-pins `RUNTIME_OUTPUT_DIRECTORY` /
  `PDB_OUTPUT_DIRECTORY` → `MY_RUNTIME_OUTPUT_DIR` and `LIBRARY_OUTPUT_DIRECTORY` /
  `ARCHIVE_OUTPUT_DIRECTORY` / `COMPILE_PDB_OUTPUT_DIRECTORY` → `MY_LIBRARY_OUTPUT_DIR`
  (`COMPILE_PDB` covers MSVC static-library PDBs, which belong with archives).
- **Exemplars:** [googletest](../gcmake-dependency-configs/googletest/post_load.cmake).
- **Status:** Permanent.

### PC-lost-usage-reqs

**Phase:** Import (post_load) · **Solved in:** `post_load.cmake` (or custom_populate for Shape C)

- **Detection signal:** Consumers miscompile or mislink against the embedded dep — wrong
  dllimport/dllexport macros, missing ABI defines — even though upstream "sets" them.
- **Root cause:** Upstream propagates a usage requirement keyed off a *variable* rather than a
  target fact, and the variable desyncs in a superbuild. Verified: yaml-cpp's PUBLIC
  `YAML_CPP_STATIC_DEFINE` is conditioned on its `YAML_BUILD_SHARED_LIBS` cache variable, not the
  target's actual type.
- **Standard solution:** Re-derive from ground truth on the imported target and re-attach:
  `gcmake_unaliased_target_name` → `get_target_property( ... TYPE )` → `target_compile_definitions(
  <target> INTERFACE ... )`.
- **Exemplars:** [yaml-cpp](../gcmake-dependency-configs/yaml-cpp/post_load.cmake).
- **Status:** Permanent.

### PC-bundled-tooling

**Phase:** Import (post_load) · **Solved in:** `post_load.cmake`

- **Detection signal:** GCMake-generated code (or documented usage) needs to `include()` a CMake
  script the dependency ships — most commonly test-discovery helpers.
- **Root cause:** As an *installed* package the dep's scripts land on the package module path
  automatically; as a *subdirectory* they don't.
- **Standard solution:** post_load appends the script directory to `CMAKE_MODULE_PATH`
  (catch2: `${catch2_SOURCE_DIR}/extras`, enabling the generated `include( Catch )` +
  `catch_discover_tests`). Contrast: doctest is instead handled by the **writer** including
  `dep/<name>/scripts/cmake/doctest.cmake` by literal path — two mechanisms for one class, and the
  writer-side path is currently broken with the dep cache (see
  [known issues](dependency_config_known_issues.md)). Prefer the module-path mechanism.
- **Exemplars:** [catch2](../gcmake-dependency-configs/catch2/post_load.cmake).
- **Status:** Permanent.

### PC-sibling-discovery

**Phase:** Import (pre_load) · **Solved in:** `pre_load.cmake` + `external_requires`

- **Detection signal:** The dep's CMake calls `find_package` for a library the user's project is
  *building as another subdirectory dependency* — which will fail (nothing is installed) or find a
  wrong system copy.
- **Root cause:** Upstream can't know its dependency is already present as targets in the same
  configure.
- **Standard solution:** Convince the dep's discovery logic to stand down while ensuring the real
  targets exist: raylib's pre_load sets `USE_EXTERNAL_GLFW OFF` (skip the find_package) **and**
  `glfw3_FOUND TRUE` (its `GlfwImport.cmake` then links the `glfw` target directly), while
  `external_requires: glfw::glfw` guarantees GCMake imported real GLFW earlier in Phase A.
  Read the dep's find/import helper before writing this — the exact variables to fake are
  dependency-specific, and the config comment must name the upstream file it's tricking.
- **Exemplars:** [raylib](../gcmake-dependency-configs/raylib/pre_load.cmake).
- **Status:** Permanent (currently one instance, but the pattern generalizes to any
  `find_package`-happy embedded dep).

## Group IV — System-installed discovery

### PC-finder-strategy

**Phase:** Discovery · **Solved in:** `module_type` choice + optional `Find<X>.cmake` + `pre_load.cmake`

- **Detection signal:** The dependency lives on the system (Shape D/E/F). How is it found, and
  does that mechanism actually work across install flavors (own CMake install, apt, MSYS2, vcpkg,
  Program Files)?
- **Root cause:** Discovery quality varies wildly: some deps export perfect package configs; some
  builtin CMake modules work; some builtin modules are broken or cache-stale; some deps have no
  finder at all.
- **Standard solution — an escalation ladder:**
  1. **Dep's own package config** (`module_type: ConfigFile`): sdl2, lws, glew (glew specifically
     because the builtin `FindGLEW` failed against apt installs — the rejected alternative is
     recorded in a comment).
  2. **Builtin find module as-is** (`BuiltinFindModule`): opengl, curl, threads, openmp,
     CUDAToolkit, wxWidgets. Declare its documented hint variables in pre_load using the
     *builtin's own* names (`THREADS_PREFER_PTHREAD_FLAG`, `GLEW_USE_STATIC_LIBS`).
  3. **Wrapper find module** (`CustomFindModule` that delegates): when the builtin works but
     caches results such that changed hints (`<X>_ROOT`, static preference) never trigger a
     re-search. Anatomy: detect hint change via a `_<X>_PREVIOUSLY_SEARCHED_FOR_*` INTERNAL cache
     var → `unset( <result vars> CACHE )` → clear `CMAKE_MODULE_PATH`, `include( Find<X> )`,
     restore (the clearing prevents the wrapper recursively including itself, since it shadows the
     builtin by name). Exemplars: zlib, openssl.
  4. **From-scratch find module**: no builtin, and upstream's config may or may not exist
     depending on install flavor. Anatomy (zstd is the canonical form, brotli its derivative):
     header comment documenting hints/targets/defines → try `find_package( <x> CONFIG )` first
     when a config *can* exist (zstd, asio incl. vcpkg detection) → fall back to
     `find_path`/`find_library` + `find_package_handle_standard_args` → alias whatever was found
     to **one stable namespaced target** → static preference via edited (and restored!)
     `CMAKE_FIND_LIBRARY_PREFIXES/SUFFIXES`, incl. MinGW `.dll.a` import libs → export
     side-channel variables the DLL step needs (`ZSTD_WINDOWS_SHARED_IMPORT_LIB`,
     `BROTLI_WINDOWS_SHARED_IMPORT_LIBRARIES`).
- **Exemplars:** [zstd](../gcmake-dependency-configs/zstd/Findzstd.cmake),
  [brotli](../gcmake-dependency-configs/brotli/FindBrotli.cmake),
  [asio](../gcmake-dependency-configs/asio/Findasio.cmake),
  [zlib](../gcmake-dependency-configs/zlib/FindZLIB.cmake),
  [openssl](../gcmake-dependency-configs/openssl/FindOpenSSL.cmake),
  [glew](../gcmake-dependency-configs/glew/dep_config.yaml).
- **Status:** Permanent. See
  [standards §6](dependency_config_standards.md#6-custom-find-module-standards) for the normative
  anatomy and the four contexts a find module must survive
  ([reference §9](predefined_dependency_system_reference.md#9-custom-find-module-lifecycle)).

## Group V — Runtime distribution

### PC-dll-distribution

**Phase:** Runtime (post_load) · **Solved in:** `post_load.cmake`

- **Detection signal:** The dependency can be (or always is) a shared library, and the target
  platform is Windows — where there is no rpath, so the DLL must sit next to the executable both
  in the build tree and in installs.
- **Root cause:** Import ≠ runtime distribution. `find_package` gives link information only.
- **Standard solution** (the canonical recipe — template
  [T5](dependency_config_procedures.md#t5--canonical-dll-copyinstall-post_loadcmake)):
  1. Gate on `TARGET_SYSTEM_IS_WINDOWS` and an `ALREADY_CONFIGURED_<DEP>` idempotency flag.
  2. Determine whether the found library is actually shared (filename/extension matching "dll",
     target `TYPE` check, or a finder side-channel variable).
  3. `find_file` the DLL **relative to what discovery already found**, with all implicit search
     paths disabled (`NO_DEFAULT_PATH`, `NO_CMAKE_ENVIRONMENT_PATH`, `NO_SYSTEM_ENVIRONMENT_PATH`,
     `NO_CMAKE_SYSTEM_PATH`, `NO_PACKAGE_ROOT_PATH`, ...) so a mismatched DLL elsewhere on the
     system can never be silently picked up.
  4. Hard-fail with a diagnostic naming the exact path searched if not found.
  5. One `add_custom_target( <name> ALL COMMAND ${CMAKE_COMMAND} -E copy ... "${MY_RUNTIME_OUTPUT_DIR}" )`
     per dependency, guarded by `NOT TARGET`.
  6. Register for installation **only** `if( DEFINED PROJECT_<dep_dir_name>_INSTALL_MODE )` via
     `add_to_needed_bin_files_list` (exact lowercase directory name — see
     [known issues](dependency_config_known_issues.md#confirmed-defects) for the casing bug).
  7. SHOULD offer an opt-out cache option (`SDL2_WIN_SHOULD_COPY_DLL`).
- **Variants by DLL-location source:** import-lib adjacency `<libdir>/../bin` (zstd, brotli);
  found-libraries list filtered for shared entries (openssl skips names containing "static"; zlib
  searches a seven-name candidate list because zlib DLL naming varies by vendor); package-config
  dir arithmetic (lws `${libwebsockets_DIR}/../bin`, curl `${CURL_DIR}/../../../bin`, glew's regex
  `lib.*$ → bin`); toolkit bin dir + regex narrowing where **ambiguity is a hard error** (cuda);
  and the extreme case — wxWidgets infers the vendor suffix statistically from the DLL directory
  because wx DLL names embed version+vendor.
- **Not in scope:** MinGW runtime DLLs — handled centrally by
  `initialize_mingw_dll_install_options`, never per-config.
- **Exemplars:** [sdl2](../gcmake-dependency-configs/sdl2/post_load.cmake) (simplest full form),
  [openssl](../gcmake-dependency-configs/openssl/post_load.cmake),
  [zlib](../gcmake-dependency-configs/zlib/post_load.cmake),
  [cuda](../gcmake-dependency-configs/cuda/post_load.cmake),
  [wxwidgets](../gcmake-dependency-configs/wxwidgets/post_load.cmake).
- **Status:** Permanent.

### PC-system-libs

**Phase:** Runtime · **Solved in:** `custom_populate.cmake` or the find module

- **Detection signal:** Linker errors on specific platforms only: `undefined reference to 'sin'`
  (`m`), `'dlopen'`/`'dlsym'` (`dl`), winsock symbols on MinGW (`ws2_32`, `wsock32`), pthread
  symbols.
- **Root cause:** Hand-built targets (Shape C/F) don't inherit upstream's knowledge of implicit
  platform libraries.
- **Standard solution:** Conditional `target_link_libraries` on the adapter target, gated on
  `TARGET_SYSTEM_IS_*` / `USING_*`: stb links `m` on Linux GCC/Clang; sqlite3 links
  `Threads::Threads` + `-ldl` on Unix; imgui links `-ldl` for sdl/opengl backends on Linux; asio's
  finder links `ws2_32`/`wsock32` under MinGW and defines `ASIO_STANDALONE`.
- **Exemplars:** [sqlite3](../gcmake-dependency-configs/sqlite3/custom_populate.cmake),
  [stb](../gcmake-dependency-configs/stb/custom_populate.cmake),
  [asio](../gcmake-dependency-configs/asio/Findasio.cmake).
- **Status:** Permanent.

## Group VI — Install & export

### PC-install-toggle

**Phase:** Install · **Solved in:** `install_var` / `inverse_install_var` / `install_by_default`

- **Detection signal:** Grep upstream for its install toggle: `option(*_INSTALL ...)`,
  `*_ENABLE_INSTALL`, `*_NO_INSTALL`, `SKIP_INSTALL_ALL`, or install rules gated on a
  master-project check.
- **Root cause:** Installing the consuming project requires the dependency's own install rules to
  be on (export-set validity, transitive headers) or off (test frameworks, minimal installs) at
  the right times — decided by GCMake's FULL/MINIMAL machinery, not by upstream defaults.
- **Standard solution:** Declare the toggle in YAML and let the writer wire it
  ([reference §6.3](predefined_dependency_system_reference.md#63-install-var-wiring-for-subdirectory-dependencies)).
  `install_var` for positive toggles (spdlog, fmt, glfw, ftxui, cxxopts, magic_enum, yaml-cpp,
  glm, nlohmann_json `JSON_Install`); `inverse_install_var` for negative ones (doctest
  `DOCTEST_NO_INSTALL`, freetype `SKIP_INSTALL_ALL`); `install_by_default: false` for never-shipped
  deps (googletest, doctest — googletest belt-and-suspenders it with a pre_load
  `option( INSTALL_GTEST OFF )` too).
- **Exemplars:** [nlohmann_json](../gcmake-dependency-configs/nlohmann_json/dep_config.yaml),
  [doctest](../gcmake-dependency-configs/doctest/dep_config.yaml),
  [googletest](../gcmake-dependency-configs/googletest/dep_config.yaml).
- **Status:** Permanent.

## Group VII — Cross-compilation

### PC-emscripten

**Phase:** Cross-compile · **Solved in:** `emscripten_config`, `can_cross_compile`, hooks

- **Detection signal:** Check [Emscripten's ports list](https://emscripten.org/docs/compiling/Building-Projects.html)
  for a `-sUSE_*` flag; otherwise, try building the dep with emcc; otherwise, determine why it
  can't work (needs to *run* at build time, needs the filesystem, etc.).
- **Standard solution — three postures**
  (semantics in [reference §10](predefined_dependency_system_reference.md#10-emscripten-and-cross-compilation-semantics)):
  1. **Internal port:** `link_flag` (`-sUSE_SDL=2`, `-sUSE_ZLIB=1`, `-sUSE_FREETYPE=1`,
     `-sUSE_SQLITE3=1`, `-sUSE_PTHREADS=1`) — the normal import is skipped under Emscripten;
     `is_flag_link_time_only: true` when the flag is link-only (glfw's `-sUSE_GLFW=3`);
     `is_internally_supported: true` alone for opengl.
  2. **Builds fine with emcc:** `can_cross_compile: true`, nothing else.
  3. **Incompatible:** guard + explain + escape hatch in a hook (cppfront pre_load fatals on
     `EMBED_CPPFRONT` + Emscripten, pointing at the system-install alternative). Extra pattern:
     raylib's pre_load force-sets `PLATFORM "Web"` under Emscripten. The `emscripten` config
     itself is a shim of INTERFACE filesystem targets, deliberately *not* platform-gated because
     they're no-ops elsewhere (documented in its YAML).
- **Exemplars:** [sdl2](../gcmake-dependency-configs/sdl2/dep_config.yaml),
  [glfw](../gcmake-dependency-configs/glfw/dep_config.yaml),
  [cppfront](../gcmake-dependency-configs/cppfront/pre_load.cmake),
  [raylib](../gcmake-dependency-configs/raylib/pre_load.cmake),
  [emscripten](../gcmake-dependency-configs/emscripten/dep_config.yaml).
- **Status:** Mostly permanent; raylib's `-sASYNCIFY` propagation gap is a tool-feature issue
  (see [known issues](dependency_config_known_issues.md#tool-feature-gaps)).

## Group VIII — Metadata & the human channel

### PC-debian-packages

**Phase:** Metadata · **Solved in:** `debian_packages`

- **Detection signal:** `apt search <dep>` — does Debian package it, and what are the runtime vs
  dev package names?
- **Standard solution:** `debian_packages.runtime` / `.dev` lists. The generated code registers
  runtime packages whenever the dep is used by an installed output, dev packages only under FULL
  mode. Primarily for module-type deps (system libraries). Subdirectory deps that build their own
  copy generally don't need packages for *themselves* (deliberately removed in Oct 2022) — but
  SHOULD list packages for **system libraries their targets link at runtime** (sfml's
  openal/vorbis/X11/freetype lists). sqlite3's self-referential listing is a confirmed
  mistake slated for removal (see [known issues](dependency_config_known_issues.md#minor-defects)).
- **Exemplars:** [sdl2](../gcmake-dependency-configs/sdl2/dep_config.yaml),
  [wxwidgets](../gcmake-dependency-configs/wxwidgets/dep_config.yaml),
  [sfml](../gcmake-dependency-configs/sfml/dep_config.yaml).
- **Status:** Permanent; cuda records the open per-*target* packages problem.

### PC-human-channel

**Phase:** Metadata · **Solved in:** `README.md`

- **Detection signal:** Anything a machine can't do for the user: manual build+install of system
  deps, required branches/versions, hint variables for troublesome platforms, known failure modes.
- **Standard solution:** A per-config README with verified commands **including the exact
  toolchain they were tested with** ("I use Ninja with the MSYS2 MinGW GCC 12.2.0 distribution"),
  recommended configure flags with a rationale table (lws), hint-variable troubleshooting
  (`OPENSSL_ROOT_DIR`, `wxWidgets_LIB_DIR`), branch/version requirements with reasons (glm,
  crow, yaml-cpp, sfml), and known-failure notes (lws's stale-build-dir `find_package` trap).
  Registry-level pointers for must-read READMEs go in the
  [registry README](../gcmake-dependency-configs/README.md).
- **Exemplars:** [lws](../gcmake-dependency-configs/lws/README.md) (the most complete),
  [openssl](../gcmake-dependency-configs/openssl/README.md),
  [glew](../gcmake-dependency-configs/glew/README.md) (release-snapshot subtlety),
  [cppfront](../gcmake-dependency-configs/cppfront/README.md).
- **Status:** Permanent.

---

## Appendix: Reverse index (configuration → classes)

Baseline modeling (`PC-target-surface`) applies to every config and is listed only where it's
non-trivial. This table doubles as a coverage audit — a config whose row disagrees with its files
deserves a look.

| Config | Classes instantiated |
| ------ | -------------------- |
| argparse | *(baseline only)* |
| asio | PC-finder-strategy (from-scratch, config-first + vcpkg), PC-system-libs, PC-debian-packages, PC-human-channel |
| brotli | PC-finder-strategy (from-scratch), PC-dll-distribution, PC-debian-packages, PC-human-channel |
| catch2 | PC-option-hygiene, PC-bundled-tooling, PC-target-surface (mutual exclusion) |
| cli11 | PC-target-surface (`actual_target_name`) |
| cppfront | PC-broken-upstream (wrapper repo), PC-emscripten (incompatible-embed guard), `config_options`, PC-human-channel |
| crow | PC-cross-dep-requires, PC-option-hygiene (feature workaround), PC-broken-upstream (master pin), PC-human-channel |
| cuda | PC-dll-distribution (glob+regex variant), PC-target-surface (`(( cuda ))` gates), PC-debian-packages |
| curl | PC-dll-distribution (config-dir arithmetic), PC-debian-packages, PC-human-channel |
| cxxopts | PC-install-toggle |
| doctest | PC-install-toggle (inverse, default off), PC-target-surface (mutual exclusion) |
| emscripten | PC-broken-upstream (authored shim repo), PC-emscripten |
| fmt | PC-version-impedance (URL method), PC-install-toggle |
| freetype | PC-install-toggle (inverse), PC-emscripten (port flag), PC-version-impedance (tag format → README), PC-human-channel |
| ftxui | PC-option-hygiene, PC-install-toggle |
| glew | PC-finder-strategy (builtin rejected → ConfigFile), PC-dll-distribution, PC-cross-dep-requires, PC-target-surface (exclusions), PC-human-channel |
| glfw | PC-option-hygiene (FORCE'd), PC-identity-mismatch (`glfw3`), PC-emscripten (link-only flag), PC-install-toggle |
| glm | PC-broken-upstream (fork), PC-option-hygiene (C++ std propagation), PC-target-surface (export-set fix), PC-install-toggle, PC-human-channel |
| googletest | PC-hardcoded-placement, PC-install-toggle (default off), PC-target-surface (mutual exclusion) |
| imgui | Shape C; PC-target-surface (system×API matrix, platform gates), PC-cross-dep-requires, PC-system-libs, PC-version-impedance |
| kokkos | PC-version-impedance (`{{PATCH:2}}`), PC-target-surface (facade-only exposure) |
| lws | PC-cross-dep-requires, PC-dll-distribution, PC-target-surface (empty namespace), PC-debian-packages, PC-human-channel |
| magic_enum | PC-install-toggle |
| nlohmann_json | PC-version-impedance, PC-identity-mismatch, PC-install-toggle |
| opengl | PC-emscripten (internally supported) |
| openmp | PC-emscripten (pthreads flag) |
| openssl | PC-finder-strategy (wrapper), PC-dll-distribution, PC-target-surface (components, Target mode), PC-debian-packages, PC-human-channel |
| pugixml | PC-target-surface (three flavors + exclusion), PC-option-hygiene |
| raylib | PC-sibling-discovery, PC-cross-dep-requires, PC-option-hygiene, PC-emscripten (PLATFORM + ASYNCIFY gap), PC-version-impedance |
| re2 | PC-option-hygiene |
| sdl2 | PC-dll-distribution, PC-target-surface (exclusions, `main` requires), PC-emscripten (port flag), PC-debian-packages, PC-human-channel |
| sfml | PC-global-pollution, PC-broken-upstream (2.6.x pin), PC-target-surface (prefix namespace, `((windows)) main`), PC-debian-packages, PC-human-channel |
| spdlog | PC-cross-dep-requires, PC-option-hygiene, PC-global-pollution, PC-install-toggle |
| sqlite3 | Shape C; PC-version-impedance (year URLs), PC-system-libs, PC-emscripten (port flag), PC-debian-packages |
| stb | Shape C; PC-system-libs, PC-target-surface (per-header targets) |
| threads | PC-option-hygiene (platform-conditional hint), PC-emscripten (pthreads flag) |
| wxwidgets | PC-dll-distribution (vendor-suffix inference), PC-target-surface (components, Variable mode + USE_FILE), PC-debian-packages, PC-human-channel |
| yaml-cpp | PC-lost-usage-reqs, PC-option-hygiene, PC-broken-upstream (master pin), PC-install-toggle |
| zlib | PC-finder-strategy (wrapper), PC-dll-distribution (name-candidate list), PC-emscripten (port flag), PC-debian-packages |
| zstd | PC-finder-strategy (from-scratch, dual-mode), PC-dll-distribution, PC-debian-packages, PC-human-channel |
