# Dependency Configuration Procedures

Step-by-step workflows for authoring, updating, and verifying configurations in the
[gcmake-dependency-configs](../gcmake-dependency-configs/) registry. Concepts live in the
[system reference](predefined_dependency_system_reference.md); recurring problems in the
[problem-class catalog](dependency_problem_classes.md); requirements in the
[standards](dependency_config_standards.md). This document tells you what to *do*, and carries the
[copyable templates](#appendix-templates).

## Prerequisites and workspace setup

1. A working `gcmake-rust` build (`cargo build`), and the registry checked out at
   `~/.gcmake/gcmake-dependency-configs` (`gcmake-rust dep-config update`). For registry
   development, work in a normal git clone of the registry and point your testing at it (or
   symlink/copy it over the `~/.gcmake` copy while testing — just remember which copy you're
   editing).
2. Clone dependency source repositories into `gcmake-rust/reference-repos/` — that directory is
   gitignored for exactly this purpose. Prefer partial clones for big repos:
   `git clone --filter=blob:none <url>`.
3. Useful commands: `gcmake-rust predep-info -d <name>` (inspect a config as the tool sees it),
   `gcmake-rust new root-project <name>` (scratch consumer projects),
   `gcmake-rust` in a project dir (regenerate CMakeLists).

> **Verification vehicle:** use hand-built **scratch projects** for end-to-end checks. As of
> August 2026 the `gcmake-test-project` samples predate current validation rules and mostly fail
> to load with gcmake-rust HEAD, so they are not a usable harness until migrated.

## Procedure A — Analyze the dependency

Produce a short **analysis worksheet** before writing any config file. It becomes the design
rationale for the config (and the source of its in-place comments). Answer the nine questions;
against each answer, note the [problem classes](dependency_problem_classes.md) that fire.

1. **What is it, physically?** Well-behaved CMake project / misbehaving CMake project / raw source
   drop with no build system / system facility / nothing consumable (→ wrapper repo).
   *Method:* read the root `CMakeLists.txt` end to end (or confirm none exists); note every
   `option()`, `install()`, `find_package()`, `set(... CACHE ...)`, `add_library`/ALIAS, and
   output-directory setting.
2. **Where should it live?** In-build subdirectory vs. on-system module.
   *Method:* [standards §2](dependency_config_standards.md#2-choosing-the-dependency-type). For
   borderline cases, time a clean build.
3. **How is it acquired and versioned?** Tags, archives, URL schemes, history size.
   *Classes:* PC-version-impedance, PC-broken-upstream.
4. **What targets does it expose, and which do we surface?** Real names, flavors, components,
   internals to hide, platform limits.
   *Method:* grep for `add_library` + `ALIAS`, or install it once and read the generated
   `*Targets.cmake`. *Classes:* PC-target-surface, PC-cross-dep-requires.
5. **What does importing it break?**
   *Method:* add it to a scratch superbuild next to at least one other dependency; diff the CMake
   cache before/after; run a multi-config build.
   *Classes:* PC-option-hygiene, PC-global-pollution, PC-hardcoded-placement, PC-lost-usage-reqs,
   PC-bundled-tooling, PC-sibling-discovery.
6. **What does installing our project require from it?** Install toggle, export-set membership,
   header destinations, `find_dependency` needs, deb packages.
   *Classes:* PC-install-toggle, PC-identity-mismatch, PC-debian-packages.
7. **What does running require?** Windows DLLs; implicit platform libraries.
   *Classes:* PC-dll-distribution, PC-system-libs.
8. **What's the cross-compile/Emscripten story?** Port flag / builds-with-emcc / incompatible.
   *Classes:* PC-emscripten.
9. **What must a human be told?** Manual installs, branch pins, troubleshooting.
   *Classes:* PC-human-channel.

## Procedure B — Author the configuration, by shape

Question 1 of the worksheet selects the shape. All shapes end with
[Procedure D](#procedure-d--verify-a-configuration) and the
[standards checklists](dependency_config_standards.md#10-compliance-checklists).

### B1 — Well-behaved CMake subdirectory

*(argparse, cli11, cxxopts, magic_enum, nlohmann_json, fmt, re2, doctest)*

1. Copy template [T1](#t1--dep_configyaml-as_subdirectory). Fill download info (both methods when
   available), namespace, targets, `install_var`.
2. If any upstream option defaults wrong for embedded use, add a pre_load with [T4](#t4--pre_loadcmake-option-hygiene)-style
   `option()` lines. Many Shape-B1 deps need no hooks at all.

### B2 — Misbehaving CMake subdirectory

*(catch2, googletest, glm, glfw, sfml, spdlog, yaml-cpp, ftxui, pugixml, raylib, crow, freetype, kokkos)*

1. Start as B1.
2. For each damage class found in worksheet question 5, add the corresponding fix:
   - Wrong defaults → pre_load [T4] (PC-option-hygiene; FORCE policy per
     [standards 5.2.2](dependency_config_standards.md#52-pre_loadcmake)).
   - Cache/global pollution → post_load `unset(... CACHE)` / restore, naming the upstream file
     responsible (PC-global-pollution).
   - Hardcoded output dirs → post_load property re-pin over `if( TARGET ...)`-guarded target list
     (PC-hardcoded-placement; googletest's post_load is the model).
   - Lost usage requirements → post_load re-derive from target facts (PC-lost-usage-reqs;
     yaml-cpp is the model).
   - Bundled CMake helpers needed later → post_load `CMAKE_MODULE_PATH` append
     (PC-bundled-tooling; catch2 is the model).
   - Internal `find_package` of a sibling dep → pre_load fake-out + `external_requires`
     (PC-sibling-discovery; raylib is the model — read the upstream import helper first and name
     it in your comment).

### B3 — Non-CMake source drop (custom populate)

*(stb, imgui, sqlite3)*

1. In `dep_config.yaml`: `requires_custom_fetchcontent_populate: true`; targets named for what
   you intend to *create*.
2. Decide target granularity deliberately: per-header INTERFACE targets (stb), one compiled
   target (sqlite3), or a structured matrix (imgui). Record the reasoning.
3. Write `custom_populate.cmake` from [T6](#t6--custom_populatecmake-skeleton). Non-negotiables
   ([standards §5.4](dependency_config_standards.md#54-custom_populatecmake)):
   `<dep>_RELATIVE_DEP_PATH = "dep/<name>"`, anchor on `${<dep>_SOURCE_DIR}`, wrap every file
   list in generators, `FILE_SET HEADERS` + `BASE_DIRS`, `SYSTEM` include dirs with paired
   BUILD/INSTALL interfaces, namespace aliases, `NOT TARGET` guards, platform system libs.
4. Upstream renames headers across versions? Use `find_file( ... REQUIRED NO_CACHE NAMES <old> <new> )`.

### B4 — Manually-installed CMake package (`ConfigFile` module)

*(sdl2, glew, lws)*

1. Build and install the dependency once, per its own docs. Record the exact commands — they
   become the README ([standards §7](dependency_config_standards.md#7-readme-standards)).
2. Read the installed `<X>Config.cmake` / `<X>Targets.cmake` for the real package name, found
   variable, and target names. Fill [T2](#t2--dep_configyaml-cmake_module).
3. If the dep can be a shared library on Windows: post_load from
   [T5](#t5--canonical-dll-copyinstall-post_loadcmake), locating the DLL relative to the package
   config dir or a variable the config file provides (sdl2's `SDL2_BINDIR`).
4. Note extra config-file variables your post_load uses as comments in the YAML (see sdl2).

### B5 — System library via builtin find module

*(opengl, curl, threads, openmp, cuda, wxwidgets)*

1. Read the CMake module's documentation page; link it as `links.cmake_find_module`. Fill
   [T2](#t2--dep_configyaml-cmake_module) (or [T3](#t3--dep_configyaml-cmake_components_module)
   for component modules), with the module's exact name/case and real found variable.
2. Declare the module's documented hint variables in pre_load using **its** names
   ([standards 3.7](dependency_config_standards.md#3-naming-and-casing)).
3. DLL post_load if applicable (worksheet Q7).
4. If the builtin module fails against real installs, escalate: wrapper ([B6], zlib/openssl
   pattern) or `ConfigFile` (glew) — and record the rejection reason in a comment.

### B6 — Custom find module

*(zstd, brotli, asio, zlib, openssl)*

1. Enumerate every way the library lands on systems (own CMake install / apt / MSYS2 / vcpkg /
   Program Files). The finder must handle all of them.
2. **Wrapper** (builtin works but caches stale): [T8](#t8--wrapper-find-module-findxcmake).
   **From scratch** (no/broken builtin): [T7](#t7--from-scratch-find-module-findxcmake), starting
   from `Findzstd.cmake` as the canonical reference.
3. `module_type: CustomFindModule`; the file MUST be named `Find<module_name>.cmake` (the loader
   derives the filename from `module_name`).
4. pre_load declares the hints (`<X>_ROOT` + preference variable per
   [standards 3.7](dependency_config_standards.md#3-naming-and-casing)).
5. Check every rule in [standards §6](dependency_config_standards.md#6-custom-find-module-standards) —
   especially the four-context constraint and no-FATAL_ERROR etiquette.

### B7 — Wrapper repository

See [Procedure E](#procedure-e--author-a-wrapper-repository); the resulting repo is then
configured as B1.

## Procedure C — Update an existing configuration

1. **Establish the era.** In the registry: `git log --format='%h %ad %s' --date=short -- <dir>/`.
   The last substantive commit dates the author's knowledge.
2. **See what the author saw.** In `reference-repos/<dep>`, check out the upstream commit/tag
   from around that date (the commit message may name the verified version — see
   [standards 9.2](dependency_config_standards.md#9-registry-hygiene--co-evolution)).
3. **Diff forward.** Diff upstream's CMake files (`CMakeLists.txt`, `cmake/`) between that era
   and the target version. For each hunk, ask: does this obsolete a hook (upstream fixed it —
   retire the fix and its comment), break a hook (renamed option/target/file), or introduce a new
   problem class?
4. **Re-run the worksheet** ([Procedure A](#procedure-a--analyze-the-dependency)) for changed
   answers only.
5. Apply changes per the shape playbook, then [verify](#procedure-d--verify-a-configuration).
   Update README version pins while you're there
   ([standards 9.4](dependency_config_standards.md#9-registry-hygiene--co-evolution)).

## Procedure D — Verify a configuration

Run in a **scratch project** (`gcmake-rust new root-project scratch-<dep>`; add the dependency
and a link to an executable output in `cmake_data.yaml`; run `gcmake-rust` to regenerate). Scale
the matrix to what the config claims — every claimed capability gets exercised, on every platform
the change plausibly affects (at minimum: Windows for any DLL logic, one Unix).

| Check | How | Catches |
| ----- | --- | ------- |
| Loads + generates | `gcmake-rust` in the scratch project | YAML schema errors, bad target refs |
| `predep-info` sanity | `gcmake-rust predep-info -d <dep>` | metadata mistakes |
| Build-tree run | `cmake -B build [-G Ninja]`, build, **run the exe from `build/bin/<config>/`** | import errors, missing DLL copies, PC-hardcoded-placement |
| Reconfigure idempotency | run CMake configure twice; second run clean | guard violations (5.1.1) |
| Hint re-search (finders) | configure, then reconfigure with `-D<X>_ROOT=...` / preference flipped | stale-cache violations (6.3) |
| Install, FULL path | `cmake --install build --prefix _install` with the dep PUBLIC-linked | export-set errors, missing headers, **DLL presence in `_install/bin`** (the check that would have caught the casing bug) |
| Install, MINIMAL path | same with the dep PRIVATE-linked | over/under-installation |
| Consume the install | tiny plain-CMake consumer: `find_package( scratch-<dep> CONFIG )` + link + build, with `CMAKE_PREFIX_PATH=_install` | config-file `find_dependency` gaps, installed find-module problems, INSTALL_INTERFACE path errors |
| Superbuild coexistence | scratch project importing this dep **plus** one other popular dep (glfw is a good canary) | duplicate targets, cache pollution (PC-global-pollution / PC-broken-upstream) |
| Tests integration (test frameworks only) | scratch project with `test_framework` + a test project; build & `ctest` | PC-bundled-tooling |
| Emscripten (only if claimed) | configure with the emscripten toolchain; confirm port-flag deps skip their import | PC-emscripten wiring |
| Multi-config (Windows) | Visual Studio generator or `Ninja Multi-Config`, build Debug+Release | `$<CONFIG>` path assumptions |

## Procedure E — Author a wrapper repository

When no consumable upstream exists (worksheet Q1):

1. Create a standalone repo under GCMake ownership (pattern: `cppfront-cmake-wrapper`,
   `gcmake-emscripten-compat`). The wrapper is a *normal, well-behaved CMake project*: proper
   targets, namespaced aliases, install rules, options with sane defaults.
2. Put all volatile knowledge (upstream revision, build workarounds) in the **wrapper**, not the
   registry config — that's the point: the registry config stays a boring Shape-B1 entry while
   the wrapper absorbs upstream churn.
3. Expose upstream-selection knobs as cache variables and surface them through `config_options`
   (cppfront's `CPPFRONT_REVISION` / `CPPFRONT_REPOSITORY`).
4. Configure the wrapper as a plain subdirectory dep ([B1](#b1--well-behaved-cmake-subdirectory)).

## Commit / PR checklist

- [ ] [Standards checklist](dependency_config_standards.md#10-compliance-checklists) for the
      shape passes
- [ ] Verification matrix run; note in the commit message which rows ran on which platforms
- [ ] Commit message names the upstream version(s) verified against
- [ ] In-place comments updated (new decisions documented; obsolete comments removed)
- [ ] README pins/instructions refreshed if touched
- [ ] [Known issues](dependency_config_known_issues.md) updated if this fixes or discovers one
- [ ] If gcmake-rust behavior changed too: co-evolution grep done
      ([standards 9.1](dependency_config_standards.md#9-registry-hygiene--co-evolution))

---

## Appendix: Templates

Replace `mydep` (registry directory name, lowercase), `MYDEP` (uppercase form), `MyDep`
(upstream's real casing) throughout. Delete inapplicable sections **and their comments**; add
in-place comments for every judgment call you make (standards §8).

### T1 — dep_config.yaml (`as_subdirectory`)

```yaml
as_subdirectory:
  can_cross_compile: true   # VERIFY, don't assume. false + TODO if untested.
  links:
    github: https://github.com/<owner>/<repo>

  download_info:
    git_method:
      repo_url: git@github.com:<owner>/<repo>.git
    # Provide url_method too when upstream publishes release archives:
    url_method:
      url_base: https://github.com/<owner>/<repo>/archive/refs/tags/
      version_transform: "v{{MAJOR}}.{{MINOR}}.{{PATCH}}"
      extensions:
        windows: zip
        unix: tar.gz

  namespace_config:
    cmakelists_linking: "MyDep::"   # or a prefix like "sfml-", or "" for bare targets

  install_var: MYDEP_INSTALL        # or inverse_install_var / install_by_default: false

  target_configs:
    mydep:
      actual_target_name: MyDep     # omit when identical to the key
      # requires: [ other_target ]              # intra-dep prerequisite
      # external_requires:
      #   - otherdep::target or otherdep::alt   # "or" = any one satisfies
  # mutually_exclusive:
  #   - [ flavor_a, flavor_b ]
```

### T2 — dep_config.yaml (`cmake_module`)

```yaml
cmake_module:
  module_type: ConfigFile   # or BuiltinFindModule / CustomFindModule (see standards 2.3)
  module_name: MyDep        # EXACT find_package name & case; also names Find<X>.cmake
  found_var: MyDep_FOUND    # the module's REAL success variable — verify it

  links:
    gcmake_readme: "https://github.com/scupit/gcmake-dependency-configs/tree/develop/mydep"
    # cmake_find_module: "https://cmake.org/cmake/help/latest/module/FindMyDep.html"

  debian_packages:
    runtime: [ libmydep1 ]
    dev: [ libmydep-dev ]

  namespace_config:
    cmakelists_linking: "MyDep::"

  targets:
    mydep:
      actual_target_name: MyDep
```

### T3 — dep_config.yaml (`cmake_components_module`)

```yaml
cmake_components_module:
  module_type: BuiltinFindModule
  module_name: MyDep

  cmakelists_usage:
    link_format: Target          # Target: link "<link_value><component>"; Variable: link "${<link_value>}"
    link_value: "MyDep::"
    found_var: MyDep_FOUND

  links:
    gcmake_readme: "https://github.com/scupit/gcmake-dependency-configs/tree/develop/mydep"
    components_doc: "<upstream components list URL>"

  components:
    base: { }
    core:
      requires: [ base ]         # component prerequisite graph
```

### T4 — pre_load.cmake (option hygiene)

```cmake
# Non-forcing: first-set wins, users can still override (standards 5.2.2).
option( MYDEP_BUILD_TESTS "Whether to build MyDep tests. GCMake sets this to OFF by default." OFF )
option( MYDEP_BUILD_EXAMPLES "Whether to build MyDep examples. GCMake sets this to OFF by default." OFF )

# FORCE only when upstream force-sets or ignores the non-forced value — say which and why:
# set( MYDEP_BUILD_DOCS FALSE CACHE BOOL "Keep off, since MyDep is built as a dependency." FORCE )

# Finder hints (module-type deps only). Use the builtin module's own hint names when wrapping;
# <X>_ROOT / <X>_PREFER_STATIC for from-scratch finders (standards 3.7):
# set( MYDEP_ROOT "" CACHE PATH "An alternative MyDep search path." )
# option( MYDEP_PREFER_STATIC "When ON, try to use static MyDep libraries instead of shared if possible." OFF )
```

### T5 — Canonical DLL copy/install (post_load.cmake)

```cmake
if( TARGET_SYSTEM_IS_WINDOWS AND NOT ALREADY_CONFIGURED_MYDEP )
  set( MYDEP_WIN_SHOULD_COPY_DLL ON CACHE BOOL "(Windows Only) whether to automatically copy the MyDep DLL to the build and install directories, when needed." )

  if( MyDep_FOUND AND MYDEP_WIN_SHOULD_COPY_DLL )
    # Only distribute when the found library is actually shared. Derive sharedness from the
    # strongest available fact: target TYPE, a finder side-channel var, or lib filename.
    find_file( MYDEP_SHARED_LIB_FILE
      NAMES
        "mydep.dll"                       # list every known vendor naming variant
      PATHS
        # Anchor to what discovery ALREADY found — e.g. "${MyDep_DIR}/../bin" (config dir),
        # "${_mydep_import_lib_dir}/../bin" (import lib), or a module-provided bin dir.
        "${MyDep_DIR}/../bin"
      # Disable all implicit search paths so a mismatched DLL elsewhere on the
      # system can never be silently picked up:
      NO_DEFAULT_PATH
      NO_PACKAGE_ROOT_PATH
      NO_CMAKE_PATH
      NO_CMAKE_ENVIRONMENT_PATH
      NO_SYSTEM_ENVIRONMENT_PATH
      NO_CMAKE_SYSTEM_PATH
    )

    if( NOT MYDEP_SHARED_LIB_FILE )
      message( FATAL_ERROR "Unable to find MyDep's DLL while searching \"${MyDep_DIR}/../bin\". Does the DLL file exist?" )
    endif()

    if( NOT TARGET copy-mydep-shared )
      add_custom_target( copy-mydep-shared ALL
        COMMAND
          ${CMAKE_COMMAND} -E copy "${MYDEP_SHARED_LIB_FILE}" "${MY_RUNTIME_OUTPUT_DIR}"
      )
      # EXACT lowercase registry directory name — CMake vars are case-sensitive (standards 3.4).
      if( DEFINED PROJECT_mydep_INSTALL_MODE )
        add_to_needed_bin_files_list( "${MYDEP_SHARED_LIB_FILE}" )
      endif()
    endif()
  endif()

  set( ALREADY_CONFIGURED_MYDEP TRUE )
endif()
```

### T6 — custom_populate.cmake skeleton

```cmake
# REQUIRED contract (standards 5.4.1): where our hand-installed headers live in the install
# tree, relative to ${CMAKE_INSTALL_INCLUDEDIR}. Convention: dep/<name>, so hand-installed
# headers can never collide with a real project's include directory.
set( mydep_RELATIVE_DEP_PATH "dep/mydep" )
set( mydep_DEP_DIR "${mydep_SOURCE_DIR}" )   # <name>_SOURCE_DIR comes from CPM
set( mydep_INCLUDE_DIR "${mydep_DEP_DIR}" )

function( _populate_mydep )
  if( NOT TARGET mydep )
    set( mydep_sources "${mydep_DEP_DIR}/mydep.c" )
    set( mydep_headers "${mydep_DEP_DIR}/mydep.h" )

    # Validate + tolerate upstream renames with a candidate list when needed:
    # find_file( mydep_main_header REQUIRED NO_CACHE NAMES "mydep.h" "mydep2.h" PATHS "${mydep_DEP_DIR}" )

    # Wrap every file list — but the two halves are used differently (standards 5.4.9):
    # SOURCES attach the build half ONLY. Attaching the install half writes an absolute
    # dep-cache path into consumers' installed export sets, which only resolves on the
    # machine that built the install (the defect imgui shipped until 2026-08).
    # HEADERS attach both halves via FILE_SET HEADERS below, which CMake installs and
    # relocates properly (reference §7.5).
    gcmake_wrap_dep_files_in_generators( mydep_sources mydep_s_b mydep_s_i )
    gcmake_wrap_dep_files_in_generators( mydep_headers mydep_h_b mydep_h_i )

    add_library( mydep )                       # INTERFACE for header-only
    add_library( mydep::mydep ALIAS mydep )    # must match namespace_config

    target_sources( mydep PRIVATE ${mydep_s_b} )
    target_sources( mydep
      PUBLIC
        FILE_SET HEADERS
          BASE_DIRS "${mydep_INCLUDE_DIR}"
          FILES ${mydep_h_b} ${mydep_h_i}
    )

    target_include_directories( mydep
      SYSTEM
      PUBLIC
        "$<BUILD_INTERFACE:${mydep_INCLUDE_DIR}>"
        "$<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}/${mydep_RELATIVE_DEP_PATH}>"
    )

    # Platform system libraries the sources need (PC-system-libs):
    # if( TARGET_SYSTEM_IS_UNIX AND (USING_GCC OR USING_CLANG) )
    #   target_link_libraries( mydep PRIVATE m )   # or -ldl, Threads::Threads, ...
    # endif()
  endif()
endfunction()

_populate_mydep()
```

### T7 — From-scratch find module (`Find<X>.cmake`)

Skeleton of the canonical anatomy — use
[Findzstd.cmake](../gcmake-dependency-configs/zstd/Findzstd.cmake) as the full reference
implementation while writing.

```cmake
# Input Hints:
#   - MYDEP_ROOT
#   - MYDEP_PREFER_STATIC
# Targets:
#   - MyDep::mydep
# Defines:
#   - MYDEP_INCLUDE_DIR, MYDEP_LIBRARY
#   - MYDEP_WINDOWS_SHARED_IMPORT_LIB   (side-channel for the DLL post_load step)

# 1. Save global search state; restore at the end (standards 5.1.2).
set( _ORIGINAL_FIND_LIBRARY_SUFFIXES ${CMAKE_FIND_LIBRARY_SUFFIXES} )

# 2. Re-search invalidation: if MYDEP_ROOT or the static preference changed since last
#    configure, unset( <result vars> CACHE ) so find_* actually re-search (standards 6.3).
#    Pattern: compare against _MYDEP_PREVIOUSLY_SEARCHED_FOR_STATIC (CACHE INTERNAL).

# 3. Config-mode first, when upstream CAN install a package config:
#    find_package( MyDep CONFIG QUIET ) ; alias its target to MyDep::mydep if found.

# 4. Raw fallback: adjust CMAKE_FIND_LIBRARY_SUFFIXES for the static preference
#    (prepend ".lib" ".a" on Windows, ".a" elsewhere; append ".dll.a" for MinGW),
#    then find_path + find_library( NAMES ... NAMES_PER_DIR ) over the search configs.

# 5. include( FindPackageHandleStandardArgs )
#    find_package_handle_standard_args( MyDep REQUIRED_VARS MYDEP_INCLUDE_DIR MYDEP_LIBRARY )
#    NEVER message( FATAL_ERROR ) on not-found — generated code owns that (standards 6.7).

# 6. On success: create ONE stable imported/alias target (MyDep::mydep) regardless of which
#    path found it, and export DLL side-channel variables (standards 6.8).

# 7. Restore global search state.
set( CMAKE_FIND_LIBRARY_SUFFIXES ${_ORIGINAL_FIND_LIBRARY_SUFFIXES} )
```

### T8 — Wrapper find module (`Find<X>.cmake`)

```cmake
# Wrapper over CMake's builtin FindMyDep: adds re-search invalidation the builtin lacks.
# (Use the builtin's own hint names: MYDEP_ROOT / MYDEP_USE_STATIC_LIBS — standards 3.7.)

if( NOT "${_MYDEP_PREVIOUSLY_SEARCHED_FOR_STATIC}" STREQUAL "${MYDEP_USE_STATIC_LIBS}" )
  set( _MYDEP_TYPE_SEARCHING_FOR_HAS_CHANGED TRUE )
else()
  set( _MYDEP_TYPE_SEARCHING_FOR_HAS_CHANGED FALSE )
endif()
set( _MYDEP_PREVIOUSLY_SEARCHED_FOR_STATIC ${MYDEP_USE_STATIC_LIBS} CACHE INTERNAL "" FORCE )

if( MYDEP_ROOT OR _MYDEP_TYPE_SEARCHING_FOR_HAS_CHANGED )
  # unset EVERY cached result var the builtin module defines — read its source for the list:
  unset( MYDEP_INCLUDE_DIR CACHE )
  unset( MYDEP_LIBRARY CACHE )
endif()

# This file shadows the builtin by name; clearing CMAKE_MODULE_PATH prevents it from
# recursively including itself (standards 6.4).
set( _INITIAL_MODULE_PATH ${CMAKE_MODULE_PATH} )
set( CMAKE_MODULE_PATH )
include( FindMyDep )
set( CMAKE_MODULE_PATH ${_INITIAL_MODULE_PATH} )
```
