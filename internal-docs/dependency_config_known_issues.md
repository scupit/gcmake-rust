# Dependency Configuration Known Issues

Living backlog of divergence between the current
[gcmake-dependency-configs](../gcmake-dependency-configs/) registry and the
[standards](dependency_config_standards.md). Sourced from the full registry audit of
**2026-08-15/16**. Update this file whenever an item is fixed, a new defect is found, or a tool
gap is closed.

Statuses: **Confirmed** (mechanism verified against source), **Needs verification** (strong
evidence, not yet reproduced in a build), **Recorded** (upstream/tool limitation, no action until
its blocker moves).

## Confirmed defects

### 1. Install-mode gate casing mismatch — DLLs silently never installed

**Status: Needs verification (high confidence) · Severity: high · Class:**
[PC-dll-distribution](dependency_problem_classes.md#pc-dll-distribution) /
[standards 3.4](dependency_config_standards.md#3-naming-and-casing)

The generated CMakeLists sets `PROJECT_<name>_INSTALL_MODE` using the **lowercase registry
directory name** (`mark_gcmake_project_usage` is called with the graph's project identifier —
the `predefined_dependencies:` key, which must match a lowercase directory). Hooks written before
the Nov 2022 lowercase migration still test the old casings. CMake variables are case-sensitive,
so those `if( DEFINED ... )` guards can never pass: **DLLs are copied to the build tree but never
registered for installation**, and installed Windows projects ship without them.

| Config | Guard tested (wrong) | Should be |
| ------ | -------------------- | --------- |
| openssl/post_load.cmake | `PROJECT_OPENSSL_INSTALL_MODE` | `PROJECT_openssl_INSTALL_MODE` |
| lws/post_load.cmake | `PROJECT_LWS_INSTALL_MODE` | `PROJECT_lws_INSTALL_MODE` |
| sdl2/post_load.cmake | `PROJECT_SDL2_INSTALL_MODE` | `PROJECT_sdl2_INSTALL_MODE` |
| brotli/post_load.cmake | `PROJECT_Brotli_INSTALL_MODE` | `PROJECT_brotli_INSTALL_MODE` |
| curl/post_load.cmake | `PROJECT_CURL_INSTALL_MODE` | `PROJECT_curl_INSTALL_MODE` |
| glew/post_load.cmake | `PROJECT_GLEW_INSTALL_MODE` | `PROJECT_glew_INSTALL_MODE` |
| zlib/post_load.cmake | `PROJECT_ZLIB_INSTALL_MODE` | `PROJECT_zlib_INSTALL_MODE` |
| wxwidgets/post_load.cmake | `PROJECT_wxWidgets_INSTALL_MODE` | `PROJECT_wxwidgets_INSTALL_MODE` |

Correct already: zstd (`PROJECT_zstd_...`), cuda (`PROJECT_cuda_...`) — both directories were
already lowercase when written. **Fix plan:** verify once via
[Procedure D](dependency_config_procedures.md#procedure-d--verify-a-configuration)'s install
check on any one config (sdl2 is easiest), then batch-fix all eight and re-verify sdl2 + one
finder-based config (zlib).

### 2. imgui custom_populate references renamed upstream headers

**Status: Confirmed · Severity: medium · Class:** Shape C / upstream drift

`imgui/custom_populate.cmake` core header list names `imgui_rectpack.h`, `imgui_textedit.h`,
`imgui_truetype.h`; upstream's files are `imstb_rectpack.h`, `imstb_textedit.h`,
`imstb_truetype.h`. Not a bug at time of writing (2023) — upstream renamed. Additionally, line
~54 appends the opengl3 loader header to `imgui_${api}_headers` while every other reference uses
`_imgui_${api}_headers` (leading underscore), so `imgui_impl_opengl3_loader.h` is never wrapped
into the header file set. **Fix plan:** part of an imgui config refresh via
[Procedure C](dependency_config_procedures.md#procedure-c--update-an-existing-configuration)
(last touched 2023-06-28; check out imgui from that era first).

### 3. doctest test-framework include path breaks with the dep cache (gcmake-rust side)

**Status: Confirmed (2026-08 session) · Severity: medium · Owner: gcmake-rust, not the registry**

`write_test_config_section` includes
`${TOPLEVEL_PROJECT_DIR}/dep/doctest/scripts/cmake/doctest.cmake` by literal path, which doesn't
exist now that CPM caches sources in `~/.gcmake/dep-cache`. catch2 and googletest use proper
module-path/`include( <Module> )` mechanisms and work. **Fix plan:** move doctest to the
catch2-style mechanism ([PC-bundled-tooling](dependency_problem_classes.md#pc-bundled-tooling)):
a registry `post_load.cmake` appending `${doctest_SOURCE_DIR}/scripts/cmake` to
`CMAKE_MODULE_PATH` plus a writer change to `include( doctest )`.

### 4. `target_compile_features` requests `c_std_*` even when C isn't an enabled project language (gcmake-rust side)

**Status: Confirmed (2026-08 session, discovered while verifying the sfml refresh) · Severity:
medium · Owner: gcmake-rust, not the registry**

`cmakelists_writer.rs`'s per-output `target_compile_features(...)` block emits
`c_std_${PROJECT_C_LANGUAGE_MINIMUM_STANDARD}` whenever `cmake_data.yaml`'s `languages.c` section
is present (`language_config.c.is_some()`), regardless of whether `project( LANGUAGES ... )` (built
from `_language_list`, which is keyed off whether any output actually has C source files) actually
enabled C. A pure-C++ root project generated with `gcmake-rust new root-project --cpp` still gets a
default `languages.c` entry in its `cmake_data.yaml`, so `project()` only declares `CXX`, no C
compiler is ever identified, and configure fails: `target_compile_features no known features for C
compiler ""`. Reproduced with no predefined dependencies involved — this isn't sfml/registry-
specific, it just happened to surface while building the sfml verification scratch project.
**Workaround used during verification:** delete the `languages.c` section from the scratch
project's `cmake_data.yaml`. **Fix plan:** either stop defaulting `languages.c` into freshly
generated C++-only projects, or gate the `c_std_*` compile-feature line on `_language_list`
actually including `C` rather than on `language_config.c.is_some()`.

## Minor defects

| Where | Problem | Fix |
| ----- | ------- | --- |
| asio/README.md | Body is copy-pasted OpenSSL content (title says asio, links/text say OpenSSL) | Rewrite for asio |
| zlib/README.md | Says "recommended to build **CURL** from source"; install step says `build-mingw-gcc` but the build dir is `build-mingw` | Fix both |
| brotli/README.md | Configure command missing `-B` (`cmake build-mingw -G 'Ninja' ...` treats the build dir as source dir) | Add `-B` |
| glew/post_load.cmake | Module-mode branch (`GLEW_SHARED_LIBRARY`) is dead code — the config is ConfigFile-only, so `GLEW_DIR` is always set | Remove branch, or comment why it's retained |
| freetype/dep_config.yaml | `links:` present but all entries commented out (empty object) | Populate or remove |
| sqlite3/dep_config.yaml | Lists `libsqlite3-0`/`libsqlite3-dev` although the dependency is always built into the project (subdirectory + custom populate) — confirmed a mistake by the author (2026-08) | Remove the `debian_packages` section on sqlite3's next touch; [standards 4.9](dependency_config_standards.md#4-dep_configyaml-standards) scopes subdirectory-dep deb packages to system-library prerequisites. Revisit only if sqlite3 ever gains a system-installed (module-type) configuration |
| lws/post_load.cmake | Error message interpolates `${the_file_base_name}`, a variable that's never set in that hook (copied from zstd's) | Fix message; superseded if the error check is standardized into a util function (the hook's own TODO) |

## Design inconsistencies (standardization candidates, not bugs)

- **Bundled test-tooling exposure has two mechanisms** — catch2 via registry post_load +
  `CMAKE_MODULE_PATH`; doctest via writer-side literal include (currently broken, see defect 3).
  Standards now prefer the module-path mechanism.
- **DLL-copy opt-out option** (`<X>_WIN_SHOULD_COPY_DLL`) exists for sdl2, lws, glew but not
  openssl, zlib, zstd, brotli, curl, cuda, wxwidgets. Standards
  [5.3.3](dependency_config_standards.md#53-post_loadcmake) now says SHOULD — add during each
  config's next touch.
- **Finder hint naming** looks inconsistent (`_PREFER_STATIC` vs `_USE_STATIC_LIBS`) but is
  actually principled — wrappers/builtin modules use the builtin's own documented names,
  from-scratch finders use `_PREFER_STATIC`. Now codified in
  [standards 3.7](dependency_config_standards.md#3-naming-and-casing); no changes needed.
- **The lws TODO** ("Standardize this error check into a function — I do this same thing in many
  configurations") — the DLL-find error check repeats across ~8 post_loads. Candidate: a
  `gcmake_find_dep_dll(...)` helper in `gcmake-general-utils.cmake`, which would collapse most of
  template [T5](dependency_config_procedures.md#t5--canonical-dll-copyinstall-post_loadcmake)'s
  body and fix defect-1-style divergence permanently.

## Tool-feature gaps

Workarounds carried by configs, pending gcmake-rust features. When a feature lands, retire the
workaround and its catalog note.

| Gap | Carried by | Workaround in place |
| --- | ---------- | ------------------- |
| Features for predefined dependencies (`(( feature:x ))` constraints) | crow | pre_load force-enables all Crow features; asio+openssl+zlib unconditionally required (over-linking) |
| Range-/version-conditional `version_transform` | raylib | URL method effectively unusable for the 5.0 tag (no patch component); git method works; design sketch in the YAML comments |
| Per-target Debian packages | cuda | All packages listed dependency-wide |
| Propagating link flags from a dep target to dependents (`-sASYNCIFY`) | raylib | GCMake passes `-sASYNCIFY` by default; `_raylib_compat` INTERFACE-target idea noted in YAML |
| libwebsockets plugin-definition CMake macros | lws | Unsupported; noted in YAML |
| Multi-section configs: `as_subdirectory` always wins selection | sfml (potential ConfigFile mode), sdl2 (potential subdir mode) | Only one mode configured per dep |
| URL fallback when git is missing on the user's system | (writer TODO) | Both-methods configs still require the user to pick one |
| `config_file_project_name` recorded but unused by the writer | glfw | Informational only |
| `predep-info` prints raw config only, never resolved project info | (info printer TODO) | Acceptable |
| kokkos internal component targets unexposed | kokkos | Facade target only; components commented out pending need |

## Staleness

Last substantive registry commit per config (as of 2026-08-16; regenerate with
`git log -1 --format='%ad' --date=short -- <dir>/` in the registry). Anything ≥ 2 years old is
presumed stale until re-verified via
[Procedure C](dependency_config_procedures.md#procedure-c--update-an-existing-configuration) —
upstream CMake has likely moved.

| Last touched | Configs |
| ------------ | ------- |
| 2022-10 | argparse, cxxopts, magic_enum, nlohmann_json, spdlog |
| 2022-11 | re2, pugixml, asio, brotli, catch2, cli11, crow, doctest, emscripten, googletest, zstd, freetype, ftxui, yaml-cpp |
| 2023-01 | curl, glew, opengl, threads, zlib |
| 2023-06/07 | fmt, imgui, kokkos, openmp, cuda |
| 2023-10 | sqlite3, glfw |
| 2023-12 | raylib, lws, sdl2, wxwidgets |
| 2024-01 | cppfront, glm |
| 2026-08 | openssl (README only), sfml, stb |

High-priority refresh candidates, weighing staleness × upstream churn × user impact: **imgui**
(defect 2), **catch2/googletest** (test frameworks, widely used), **crow** (the pinned
master-branch situation has had years to change).
