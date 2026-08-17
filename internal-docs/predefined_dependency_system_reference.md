# Predefined Dependency System Reference

This is the internal "how it actually works" reference for the predefined dependency compatibility
layer. The user-facing overview ([docs/predefined_dependency_doc.md](../docs/predefined_dependency_doc.md))
covers *what* the layer is; this document covers *how* and *why*, at the level of detail a
configuration author or gcmake-rust developer needs.

Companion documents: [problem classes](dependency_problem_classes.md) ·
[standards](dependency_config_standards.md) · [procedures](dependency_config_procedures.md) ·
[known issues](dependency_config_known_issues.md)

## Table of Contents

1. [The compatibility-layer philosophy](#1-the-compatibility-layer-philosophy)
2. [Lifecycle overview](#2-lifecycle-overview)
3. [Stage 1: Raw loading and validation](#3-stage-1-raw-loading-and-validation)
4. [Stage 2: Final configuration resolution](#4-stage-2-final-configuration-resolution)
5. [Stage 3: Dependency graph resolution](#5-stage-3-dependency-graph-resolution)
6. [Stage 4: Code generation — anatomy of the generated blocks](#6-stage-4-code-generation--anatomy-of-the-generated-blocks)
7. [The configure-time runtime contract](#7-the-configure-time-runtime-contract)
8. [dep_config.yaml field reference](#8-dep_configyaml-field-reference)
9. [Custom find module lifecycle](#9-custom-find-module-lifecycle)
10. [Emscripten and cross-compilation semantics](#10-emscripten-and-cross-compilation-semantics)
11. [Glossary](#11-glossary)

## 1. The compatibility-layer philosophy

GCMake-generated projects obey strong invariants that arbitrary third-party projects do not:

- Uniform include prefixes and `FILE_SET HEADERS`-based header installation.
- Every consumable target is aliased into a namespace and joins the project's export set, so the
  installed package config file works transitively.
- Per-configuration build-tree output directories (`bin/$<CONFIG>`, `lib/$<CONFIG>`) shared by the
  whole superbuild, so executables are runnable from the build tree.
- Install behavior driven by *usage*: things are installed only when an installed output actually
  needs them, at either `FULL` (headers + dev artifacts) or `MINIMAL` (runtime pieces) depth.
- Cross-compilation and Emscripten awareness.

A predefined dependency configuration is a **per-library adapter** that makes one third-party
dependency behave *as if* it followed those invariants. Each directory in the
[gcmake-dependency-configs](../gcmake-dependency-configs/) registry is one adapter: a
`dep_config.yaml` describing the dependency declaratively, plus up to four CMake hook files that
patch the gap between how the dependency behaves and how GCMake needs it to behave.

The controlling design constraint: **users never modify dependencies and never write CMake.** All
knowledge about a dependency's quirks lives in the registry, once, and every GCMake project
benefits.

## 2. Lifecycle overview

```text
 dep_config.yaml + hooks          cmake_data.yaml
 (registry, ~/.gcmake/...)        (user project)
        │                               │
        ▼                               ▼
 [1] Raw load + validation ──── [2] Final config resolution
     RawPredefinedDependencyMap     FinalPredefinedDependencyConfig
                                        │
                                        ▼
                              [3] Dependency graph resolution
                                  requires / external_requires /
                                  mutual exclusion / usage analysis
                                        │
                                        ▼
                              [4] Code generation
                                  cmakelists_writer.rs splices hooks
                                  into the generated CMakeLists.txt
                                        │
                                        ▼
                              [5] CMake configure time
                                  spliced code runs inside the GCMake
                                  CMake utility library environment
```

Key source locations:

| Concern | Location |
| ------- | -------- |
| Raw YAML schema + hook loading | [src/project_info/raw_data_in/dependencies/](../src/project_info/raw_data_in/dependencies/) |
| Final (merged) dependency types | [src/project_info/final_dependencies/](../src/project_info/final_dependencies/) |
| Graph resolution | [src/project_info/dependency_graph_mod/dependency_graph.rs](../src/project_info/dependency_graph_mod/dependency_graph.rs) |
| Code generation | [src/file_writers/cmake_writer/cmakelists_writer.rs](../src/file_writers/cmake_writer/cmakelists_writer.rs) |
| CMake utility library ("GCMake standard library") | [src/file_writers/cmake_writer/util_files/](../src/file_writers/cmake_writer/util_files/) (ordering defined in [build.rs](../build.rs)) |
| Info printing (`gcmake-rust predep-info`) | [src/program_actions/info_printers/predef_dep_info_print_funcs.rs](../src/program_actions/info_printers/predef_dep_info_print_funcs.rs) |

## 3. Stage 1: Raw loading and validation

`RawPredefinedDependencyMap` (in `raw_data_in/dependencies/mod.rs`) is a **lazy** map over the
registry checkout at `~/.gcmake/gcmake-dependency-configs`. At construction it only enumerates
directory names (every non-hidden directory is a legal dependency name). A configuration is parsed
the first time something asks for it.

Loading a configuration reads:

- `dep_config.yaml` → `SingleRawPredefinedDependencyConfigGroup`, which has three optional
  sections: `as_subdirectory`, `cmake_components_module`, `cmake_module`. All deserialization uses
  `#[serde(deny_unknown_fields)]`, so typos in the YAML fail loudly.
- `pre_load.cmake`, `post_load.cmake`, `custom_populate.cmake` — each optional, loaded verbatim.
- `Find<base>.cmake`, where `<base>` is the config's `module_name` if a module section provides
  one, otherwise the config directory name.

Validation enforced at this stage (or immediately after, in final-config construction):

- A subdirectory config with `requires_custom_fetchcontent_populate: true` **must** have a
  `custom_populate.cmake`.
- A `module_type: CustomFindModule` config **must** have a matching find module file.
- A `module_type: BuiltinFindModule` config **must not** have one (you'd be shadowing CMake's own
  module unintentionally — if you want that, declare `CustomFindModule`).

## 4. Stage 2: Final configuration resolution

`FinalPredefinedDependencyConfig::new` (in `final_dependencies/mod.rs`) merges the raw registry
config with the user's entry under `predefined_dependencies:` in `cmake_data.yaml`
(`UserGivenPredefinedDependencyConfig`: `git_tag` / `commit_hash` / `repo_url` / `file_version` /
`options`).

**Section precedence:** if a config defines multiple sections, `as_subdirectory` wins, then
`cmake_module`, then `cmake_components_module`. This is a known limitation (noted in the source):
SFML, for example, could reasonably support both subdirectory and installed-config-file modes, but
only one mode is ever selected.

**Download method resolution** (`resolve_download_method` in `final_predefined_subdir_dep.rs`):
the *user's options select the mode*. Git options (`git_tag`/`commit_hash`/`repo_url`) select git
mode; `file_version` selects URL mode; specifying both is an error, as is specifying options for a
mode the config doesn't support. URL mode parses `file_version` as a strict three-part version,
applies the config's `version_transform` template, and resolves the base URL — either the flat
`url_base` or, with `url_base_by_version`, the entry with the **largest version key ≤ the requested
version** (this is how sqlite3's per-year URLs work).

**Target map construction** (`make_final_target_config_map` in `final_target_map_common.rs`):

- Target keys may carry a leading constraint spec (`((windows)) sdl_directx9`, `(( cuda )) runtime`)
  which is parsed off and stored as the target's system-spec.
- `requires` entries split on `" or "` into one-of requirement sets.
- `external_requires` entries must each be a single non-nested `namespace::target` link specifier
  (alternatives via `" or "`); at this stage the referenced predefined dependency must **exist in
  the registry** and contain the named target (the graph checks importation later).
- `mutually_exclusive` groups are expanded pairwise into `MutuallyExclusive` requirement entries on
  every member.
- Order inside `or` alternatives carries **no preference semantics** — any one satisfies.

**Install details:** `install_var` / `inverse_install_var` / `install_by_default` become a
`SubdirDepInstallationConfig` used by the writer (§6.3).

## 5. Stage 3: Dependency graph resolution

The dependency graph wires predefined-dependency targets into the project's link graph:

- **Intra-dependency `requires`** become target-to-target links inside the dependency, so linking
  `wxwidgets::richtext` transitively pulls `html`, `xml`, `core`, `base` in the right order.
- **`external_requires`** become *one-of complex requirements* pointing at targets of **other**
  predefined dependencies (`ensure_proper_predefined_dep_links`). If the required dependency isn't
  imported by the user's project, the requirement is recorded as "not imported" — and the user gets
  an error explaining what to add. When exactly one alternative exists and its project is imported,
  it is treated as a regular dependency target.
- **Link propagation:** when an output links a predefined target, the target's own requirements are
  auto-linked to the output with the same access category (PUBLIC/PRIVATE/INTERFACE). Existing
  auto-created links upgrade to the more permissive category; a *user-specified* link with a less
  permissive category than required is an error.
- **Usage analysis:** the graph knows exactly which outputs (including test executables and
  pre-build scripts) consume each dependency target, and under which platform constraints. This
  feeds the usage conditionals in §6.1 — and produces a load-time warning when a listed dependency
  is never actually linked to anything.

## 6. Stage 4: Code generation — anatomy of the generated blocks

Order of the generated **root** CMakeLists.txt (from `write_cmakelists`):

1. Project header; `include( cmake/<util>.cmake )` for the whole utility library; accumulator-list
   initialization.
2. Toplevel tweaks — including `initialize_install_mode()` and the **usage-marking block** (§6.4),
   plus feature setup and `gcmake_begin_config_file()`.
3. **Language configuration** — deliberately written *before* dependencies so hooks can read
   `PROJECT_CXX_LANGUAGE_MINIMUM_STANDARD` / `..._EXACT_STANDARD` (glm's `pre_load.cmake` depends
   on this).
4. **Phase A — Predefined dependency imports** (`write_predefined_dependencies`, §6.1–6.3), then
   GCMake dependency imports.
5. **Phase B — Apply dependencies** (`write_apply_dependencies`, §6.5): custom population and
   post-load hooks. Then `DEPENDENCY_INSTALL_LIBDIR` is captured and `CMAKE_INSTALL_LIBDIR`
   restored (§7.6), then build configurations.
6. Root vars, tests configuration, pre-build script + outputs + subprojects, test projects,
   documentation, installation/export (`gcmake_end_config_file()` → `configure_installation`),
   CPack.

> **NOTE:** a GCMake *dependency* project's own generated CMakeLists contains its own Phase A/B
> blocks. When it is consumed via `gcmake_dependencies`, those blocks run as a subdirectory of the
> consumer's configure — so the same predefined-dependency hooks can execute **more than once per
> configure**. This is why hooks require idempotency guards (see
> [standards §5](dependency_config_standards.md#5-hook-file-standards)).

### 6.1 The usage conditional

Every dependency's Phase A and Phase B blocks are wrapped in:

```cmake
if( <usage conditional> )
  # ... the dependency's import / apply code
endif()
```

The conditional is an `OR` over every consumer of the dependency. For a regular installable output
it looks like `(DEFINED TARGET_<output>_INSTALL_MODE AND (<platform constraint>))`; for test-only
or pre-build consumers it is just the platform constraint. Net effect: **a dependency that no
enabled output needs is never even downloaded**, and platform-gated usage (`((windows))`) skips the
dependency on other platforms.

### 6.2 Phase A: the import block, in exact order

```cmake
if( <usage conditional> )
  # 1. User-provided config_options values:
  set( CPPFRONT_REVISION "..." CACHE STRING "..." )

  # 2. pre_load.cmake, spliced VERBATIM (runs before the import — this is where
  #    option() defaults are preempted and finder hints are declared)

  # 3. If a custom find module exists:
  #    - the file is copied into <project>/cmake/modules/  (Rust-side file copy)
  #    - add_to_custom_find_modules_list( <dep_name> )     (registers it for installation)

  # 4. The import itself — one of three forms:

  #    cmake_module:
  find_package( <module_name> MODULE )   # or CONFIG for module_type: ConfigFile
  if( NOT <found_var> )
    message( FATAL_ERROR "...not found... See <gcmake_readme link> ..." )
  endif()
  if( "${PROJECT_<dep_name>_INSTALL_MODE}" STREQUAL "FULL" )
    gcmake_config_file_add_contents( "find_dependency( <module_name> MODULE )" )
  endif()

  #    cmake_components_module: same, but
  find_package( <module_name> MODULE COMPONENTS <used components> )
  #    Components are listed in REVERSE build order (dependents before dependencies)
  #    because link order matters for GCC-family linkers.

  #    as_subdirectory:
  set( <dep_name>_RELATIVE_DEP_PATH "<installed_include_dir_name or dep_name>" )
  #    ^ OMITTED when requires_custom_fetchcontent_populate — the custom_populate
  #      script must set it instead (see §7.4).
  #    ... install_var wiring (§6.3) ...
  CPMAddPackage(
    NAME <dep_name>
    DOWNLOAD_ONLY <ON if custom populate, else OFF>
    GIT_REPOSITORY "..."   # or URL "${<dep_name>_DOWNLOAD_URL}" chosen per host OS
    GIT_TAG "..."
    GIT_SUBMODULES_RECURSE TRUE
    SYSTEM
  )
  append_to_actual_dep_list( <dep_name> )

  # 5. Debian package registration:
  if( DEFINED PROJECT_<dep_name>_INSTALL_MODE )
    add_to_deb_list( "<runtime package>" )
    if( "${PROJECT_<dep_name>_INSTALL_MODE}" STREQUAL "FULL" )
      add_to_deb_list( "<dev package>" )
    endif()
  endif()
endif()
```

### 6.3 Install-var wiring for subdirectory dependencies

If the config declares `install_var` (or `inverse_install_var`), the writer emits logic that drives
the dependency's own install toggle from GCMake's install-mode machinery:

- `install_by_default: false` → a plain `option( <var> OFF )` (inverted appropriately for
  `inverse_install_var`).
- Otherwise → `<var>_DEFAULT_VALUE` is computed from `PROJECT_<dep_name>_INSTALL_MODE`
  (`FULL` → install ON, `MINIMAL` → install OFF), then exposed as
  `option( <var> ${<var>_DEFAULT_VALUE} )` so users can still override it.
- A dependency that is never used gets its install var forced to the "don't install" value.

Rationale for installing subdirectory dependencies by default at all (from the long comment in
`write_predefined_subdirectory_dependency`): shared-library dependents and PUBLIC/INTERFACE-linked
headers make dependency installation *transitively necessary*, and CMake gives us no reliable
universal way to detect either case across the ecosystem's many static/shared selection schemes.
So the default errs toward "installs work out of the box, possibly with extra files."

### 6.4 Install-mode marking (who sets `PROJECT_*_INSTALL_MODE`?)

At the top of the root CMakeLists (before any dependency code), the writer walks all outputs in
reverse build order and emits:

- `mark_gcmake_target_usage( <output> FULL )` — gated on `GCMAKE_INSTALL_MODE`
  (NORMAL/EXE_ONLY/LIB_ONLY) and, for libraries, on `IN_GCMAKE_CONTEXT` (libraries default to
  installable when a non-GCMake project consumes us).
- For each dependency of an installable output:
  `mark_gcmake_target_usage( <dep_target> FULL|MINIMAL )` for GCMake targets, and
  `mark_gcmake_project_usage( <predep_name> FULL|MINIMAL )` for predefined dependencies —
  **FULL when PUBLIC/INTERFACE-linked, MINIMAL when PRIVATE-linked**.

These macros (in `gcmake-installation-utils.cmake`) set `TARGET_<name>_INSTALL_MODE` and
`PROJECT_<name>_INSTALL_MODE`, upgrading MINIMAL→FULL but never downgrading. The `<name>` for a
predefined dependency is **exactly the registry directory name** (lowercase). Therefore:

> `DEFINED PROJECT_<dep_dir_name>_INSTALL_MODE` ⇒ "this dependency is used by something that gets
> installed" — the standard guard before registering DLLs or deb packages for installation.
> **CMake variables are case-sensitive; the name must match the directory name exactly.** (A batch
> of pre-Nov-2022 hooks violate this — see
> [known issues](dependency_config_known_issues.md).)

### 6.5 Phase B: the apply block

Emitted per dependency, after *all* imports (Phase A) have run, before any project targets exist:

```cmake
if( <usage conditional> )
  # Only for requires_custom_fetchcontent_populate deps:
  if( NOT <dep_name>_ADDED )   # <name>_ADDED comes from CPM
    message( FATAL_ERROR "...custom population ran before download..." )
  endif()
  if( NOT <dep_name>_GCMAKE_CONFIGURED )
    # custom_populate.cmake spliced verbatim (indented) here
    set( <dep_name>_GCMAKE_CONFIGURED TRUE )
  endif()

  # For ALL predefined deps that have one:
  # post_load.cmake spliced VERBATIM here
endif()
```

The `<dep>_GCMAKE_CONFIGURED` flag is writer-provided idempotency for custom population;
post_load scripts must provide their own (the `ALREADY_CONFIGURED_<DEP>` convention).

## 7. The configure-time runtime contract

Hook scripts are written against the environment the GCMake CMake utility library establishes.
This section is the API surface hooks may rely on. The library is embedded in the gcmake-rust
binary from [util_files/](../src/file_writers/cmake_writer/util_files/) (concatenation order in
[build.rs](../build.rs)) and written to `<project>/cmake/*.cmake` at generation time.

### 7.1 Directory variables

| Variable | Value | Meaning |
| -------- | ----- | ------- |
| `MY_RUNTIME_OUTPUT_DIR` | `${CMAKE_BINARY_DIR}/${CMAKE_INSTALL_BINDIR}/$<CONFIG>` | Standardized build-tree bin dir for the whole superbuild. Copy DLLs here so build-tree executables run. |
| `MY_LIBRARY_OUTPUT_DIR` | `${CMAKE_BINARY_DIR}/${CMAKE_INSTALL_LIBDIR}/$<CONFIG>` | Standardized build-tree lib dir (also compile-PDB destination on MSVC). |
| `TOPLEVEL_PROJECT_DIR` | root project source dir | Also anchors `cmake/modules/` on `CMAKE_MODULE_PATH`. |
| `GCMAKE_CONFIG_DIR`, `GCMAKE_DEP_CACHE_DIR` | `~/.gcmake`, `~/.gcmake/dep-cache` | CPM's `CPM_SOURCE_CACHE` defaults to the dep cache. |

### 7.2 Platform and compiler variables (`gcmake-variables.cmake`)

`USING_GCC` / `USING_CLANG` / `USING_MSVC` / `USING_MINGW` / `USING_CUDA` / `USING_EMSCRIPTEN`;
`CURRENT_SYSTEM_IS_{WINDOWS,LINUX,MACOS,UNIX}` (host) vs
`TARGET_SYSTEM_IS_{WINDOWS,LINUX,MACOS,UNIX,ANDROID}` (target). **Host vs target matters**: DLL
copying keys off the *target* system; things that run at configure/build time key off the *host*.

### 7.3 Accumulator lists (`gcmake-installation-utils.cmake`)

Initialized per project scope, raised through subprojects, consumed by `configure_installation`:

| Add with | Consumed as |
| -------- | ----------- |
| `add_to_needed_bin_files_list( <abs path> )` | `install( FILES ... DESTINATION ${CMAKE_INSTALL_BINDIR} )` — external DLLs shipped with the project. |
| `add_to_additional_dependency_install_list( <target> <rel path> )` | Dependency target joins the project export set; `<rel path>` (see §7.4) becomes its header/`INCLUDES` destination under `${CMAKE_INSTALL_INCLUDEDIR}/`. Used for PUBLIC/INTERFACE-linked subdirectory-dep targets. |
| `add_to_minimal_installs( <target> <rel path> )` | Same mechanism, for PRIVATE-linked targets (CMake can't truly install "runtime only", so this is currently equivalent — the distinction is kept for intent and future improvement). |
| `add_to_deb_list( <package> )` | CPack Debian dependency metadata. |
| `add_to_custom_find_modules_list( <dep> )` | Installs `cmake/modules/Find<dep>.cmake` with the project (§9). |

The writer emits the `add_to_additional_dependency_install_list` / `add_to_minimal_installs` calls
itself for subdirectory-dependency targets linked to installable outputs; hooks normally only ever
call `add_to_needed_bin_files_list`.

### 7.4 `<dep>_RELATIVE_DEP_PATH`

The include-directory path, relative to `${CMAKE_INSTALL_INCLUDEDIR}`, where the dependency's
headers live in the install tree.

- For ordinary subdirectory deps, the **writer** sets it to `installed_include_dir_name` (if
  given) or the dependency name — matching wherever the dependency's own install rules put its
  headers (e.g. `nlohmann`).
- For `requires_custom_fetchcontent_populate` deps, the **custom_populate script must set it**,
  because the script controls the install layout. Convention: `dep/<name>` — hand-installed
  headers are namespaced under `include/dep/` so they can never collide with a real project's
  include directory. Every `$<INSTALL_INTERFACE:...>` include dir the script declares must agree
  with this value.

### 7.5 File-list generator wrapping (`gcmake-general-utils.cmake`)

`gcmake_wrap_dep_files_in_generators( <list_var> <out_build> <out_install> )` converts a list of
absolute paths into two lists: `$<BUILD_INTERFACE:abs path>` entries and
`$<INSTALL_INTERFACE:path stripped of the toplevel prefix>` entries. Required because absolute
build-tree/dep-cache paths are illegal inside installed export sets: every `target_sources` /
`FILE_SET` on a custom-populated target needs both halves or either the build or `install(EXPORT)`
breaks. (`gcmake_wrap_files_in_generators` is the sibling that strips the *current source dir*
prefix — used for the project's own files, not dependencies.)

Also useful: `gcmake_unaliased_target_name( <maybe-alias> <out_var> )` — resolves an ALIAS to its
real target so properties can be read/written.

### 7.6 Install and export plumbing

- `gcmake_config_file_add_contents( "<line>" )` appends to the project's generated
  `Config.cmake.in`. The writer uses it to add `find_dependency(...)` lines for FULL-mode module
  dependencies, so consumers of the *installed* project re-find them.
- The generated config file temporarily rewrites `CMAKE_MODULE_PATH` to the installed `modules/`
  directory while running its `find_dependency` calls — which is why custom find modules are
  installed alongside the project, and why they must be self-contained and re-runnable (§9).
- `DEPENDENCY_INSTALL_LIBDIR`: during dependency configuration on non-Windows systems,
  `CMAKE_INSTALL_LIBDIR` is temporarily pointed at a `lib/dependencies/<project>`-style
  subdirectory so installed dependency libraries can't clobber system-installed ones; the value is
  captured into `DEPENDENCY_INSTALL_LIBDIR` and `CMAKE_INSTALL_LIBDIR` is restored before the
  project's own targets are configured. (This is why SFML must be ≥ 2.6.x — older branches ignore
  `CMAKE_INSTALL_LIBDIR`.)
- `CMAKE_INSTALL_INCLUDEDIR` is also remapped at the toplevel — permanently, for the whole
  configure — to `include/<project-name>`, so installed headers never collide with system
  headers. Every header destination expressed relative to it, including `RELATIVE_DEP_PATH`
  (§7.4), therefore lands under `include/<project>/...` on disk (e.g.
  `include/<project>/dep/imgui/` for custom-populated headers).
- MinGW runtime DLLs (`libstdc++-6.dll`, etc.) are handled **centrally** in
  `gcmake-installation-utils.cmake` (`initialize_mingw_dll_install_options`) — never per-config.

### 7.7 CPM facts hooks can rely on

After `CPMAddPackage( NAME <dep> ... )`:

- `<dep>_SOURCE_DIR` — the checkout in the dep cache (custom_populate scripts anchor on this).
- `<dep>_ADDED` — TRUE when the package was added this configure (the writer asserts it before
  splicing custom population).
- With `DOWNLOAD_ONLY ON`, sources are fetched but **no** `add_subdirectory` happens — the
  custom_populate script is the build system.

## 8. dep_config.yaml field reference

Schema types live in
[src/project_info/raw_data_in/dependencies/internal_dep_config/](../src/project_info/raw_data_in/dependencies/internal_dep_config/).
Unknown fields are rejected. Where a field exists because of one specific real-world mismatch, the
motivating example is listed — that's usually the fastest way to understand the field.

### 8.1 `as_subdirectory` (`RawSubdirectoryDependency`)

| Field | Required | Purpose / motivating example |
| ----- | -------- | ---------------------------- |
| `can_cross_compile` | yes | Whether the dep trivially cross-compiles. Feeds `predep-info` and Emscripten support decisions. |
| `namespace_config.cmakelists_linking` | yes | Prefix prepended to `actual_target_name` for linking. A namespace (`spdlog::`), a plain prefix (`sfml-` → `sfml-graphics`), or `""` for un-namespaced targets (glfw, freetype, raylib). |
| `download_info.git_method.repo_url` | one method required | Default clone URL. May point at a fork (glm) or wrapper repo (cppfront). |
| `download_info.url_method` | one method required | `url_base` **or** `url_base_by_version` (sqlite3's per-year URLs; largest key ≤ requested version wins), `version_transform` template (`{{MAJOR}}.{{MINOR}}.{{PATCH}}` with optional `:N` zero-padding — kokkos `{{PATCH:2}}`, sqlite3 `{{MINOR:3}}{{PATCH:3}}`), per-platform `extensions`. Prefer URL mode for huge-history repos (nlohmann_json). |
| `target_configs` | yes | Map of exposed target name → `{ requires, external_requires, actual_target_name }`. Keys may carry `((constraint))` prefixes. |
| `mutually_exclusive` | no | Groups of targets that can't be linked to one output together (catch2 `with_main`/`without_main`; pugixml `pugixml`/`shared`/`static`). |
| `install_var` / `inverse_install_var` | no | The dependency's own install toggle (`SPDLOG_INSTALL`; doctest's inverse `DOCTEST_NO_INSTALL`; freetype's inverse `SKIP_INSTALL_ALL`). Drives §6.3. |
| `install_by_default` | no (default true) | `false` for test-frameworks and other never-shipped deps (googletest, doctest). |
| `installed_include_dir_name` | no | When the installed include dir ≠ dep name (nlohmann_json installs headers to `include/nlohmann`). Feeds `<dep>_RELATIVE_DEP_PATH`. |
| `config_file_project_name` | no | When the installed package-config name ≠ dep name (glfw → `glfw3`). Currently recorded but unused by the writer. |
| `requires_custom_fetchcontent_populate` | no (default false) | The dep has no usable CMakeLists — download only, `custom_populate.cmake` builds the targets (stb, imgui, sqlite3). |
| `emscripten_config` | no | See §10. |
| `debian_packages.runtime` / `.dev` | no | apt package names for CPack DEB dependency metadata. |
| `config_options` | no | User-settable passthrough options: `{ name: { cmake_var, cache_description } }` (cppfront's `CPPFRONT_REVISION`/`CPPFRONT_REPOSITORY`). Values arrive via the user's `options:` map. |
| `features` | no | Features the dependency declares: `{ name: { default, enables?, list_value? } }` (imgui's `freetype`; crow's `ssl`/`compression`). `enables` may only name features of the SAME dep. Feature names gate `external_requires` entries via `(( feature:<name> ))` prefixes (validated against the dep's own declared set), and each feature is exposed to hooks as a `GCMAKE_PREDEP_<dep>_FEATURE_<name>` boolean, registered and resolved in the consumer root's generated file BEFORE any Phase A block (all shapes support this; the field also exists on the module-dep structs). Users enable them via `features:` / `use_default_features:` on the import entry, or a project feature's `enables: [dep/feature]`. Unmet feature-gated requirements become configure-time FATAL_ERROR guards instead of load errors. |
| `feature_mode.list_var` | no | Name of a CMake list variable which receives every enabled feature's `list_value` before hooks run (crow's `CROW_FEATURES`). Invalid without `features`. |
| `links.github` / `links.gcmake_readme` | no | Info links for `predep-info`. |

### 8.2 `cmake_module` (`RawModuleDep`)

| Field | Required | Purpose |
| ----- | -------- | ------- |
| `module_type` | yes | `ConfigFile` (dep's own installed package config — sdl2, glew, lws), `BuiltinFindModule` (CMake ships the finder — opengl, curl, threads, openmp, cuda), `CustomFindModule` (registry ships `Find<module_name>.cmake` — zstd, brotli, asio, zlib, openssl). Translates to `find_package( <module_name> CONFIG )` vs `find_package( <module_name> MODULE )`. |
| `module_name` | yes | The exact `find_package` name **and** the find module file base name. Case matters (`wxWidgets`, `CUDAToolkit`). |
| `found_var` | yes | Variable checked after `find_package` to produce a friendly FATAL_ERROR pointing at `links.gcmake_readme`. Match the module's real convention (`OPENGL_FOUND`, `libwebsockets_FOUND`, `zstd_FOUND`). |
| `links.gcmake_readme` | yes | Where the error message sends users for install instructions. `cmake_find_module` / `components_doc` optional. |
| `namespace_config`, `targets`, `mutually_exclusive`, `debian_packages`, `emscripten_config`, `config_options` | | Same semantics as §8.1. |

### 8.3 `cmake_components_module` (`RawComponentsModuleDep`)

Like `cmake_module`, plus:

| Field | Purpose |
| ----- | ------- |
| `cmakelists_usage.link_format` | `Target` (link `<link_value><component target>` — openssl's `OpenSSL::` + `SSL`/`Crypto`) or `Variable` (link `${<link_value>}` — wxWidgets' `wxWidgets_LIBRARIES`). |
| `cmakelists_usage.found_var` / `link_value` | As above. |
| `components` | The component map (same shape as `targets`), including inter-component `requires` (the wxWidgets component graph). Used components are passed to `find_package( ... COMPONENTS ... )` in reverse build order. |

## 9. Custom find module lifecycle

A registry `Find<X>.cmake` travels through four contexts — it must work identically in all of them:

1. **Registry** — source of truth, loaded by gcmake-rust.
2. **Generated project** — copied to `<project>/cmake/modules/`, which the generated root
   CMakeLists puts on `CMAKE_MODULE_PATH`. `find_package( <X> MODULE )` finds it there. Because
   the copy shadows any builtin module of the same name, wrapper modules that delegate to the
   builtin must clear `CMAKE_MODULE_PATH` around their inner `include( Find<X> )` to avoid
   including themselves recursively (see zlib/openssl wrappers).
3. **Install tree** — installed to `${CMAKE_INSTALL_LIBDIR}/cmake/<project>/modules/` via
   `add_to_custom_find_modules_list`.
4. **Downstream consumer** — the installed project's config file temporarily points
   `CMAKE_MODULE_PATH` at that installed `modules/` dir and re-runs `find_dependency( <X> MODULE )`
   on the consumer's machine.

Consequences: the module must be **self-contained** (no references to registry paths or generation-
time state), **re-runnable** (results cached by `find_*` must be invalidated when hints change —
the `_<X>_PREVIOUSLY_SEARCHED_FOR_*` pattern), and **quiet-failure-capable** (report through
`<found_var>` / FPHSA rather than fataling itself; the generated code owns the fatal error).

## 10. Emscripten and cross-compilation semantics

A dependency has one of three Emscripten postures, declared via `emscripten_config`:

1. **Internally supported port** — `is_internally_supported: true` and/or `link_flag:
   "-sUSE_SDL=2"`. Under Emscripten the normal import (find_package / CPMAddPackage / custom
   populate) is **skipped entirely** (`if( NOT USING_EMSCRIPTEN )` wrapping), and the flag is
   applied to consuming targets' compile and link options. `is_flag_link_time_only: true` (glfw's
   `-sUSE_GLFW=3`) restricts it to link options.
2. **Works under Emscripten like anywhere else** — `can_cross_compile: true`, no
   `emscripten_config`. The dep builds as part of the project with emcc.
3. **Unsupported / conditionally supported** — no flag, `can_cross_compile: false`, or an explicit
   guard in a hook (cppfront's pre_load fatals on `EMBED_CPPFRONT` + Emscripten, with the
   documented escape hatch of using a system installation).

`can_cross_compile` also gates general cross-compilation expectations and is surfaced by
`gcmake-rust predep-info`.

## 11. Glossary

| Term | Meaning |
| ---- | ------- |
| **Registry** | The gcmake-dependency-configs repository (`~/.gcmake/gcmake-dependency-configs`). |
| **Adapter / configuration** | One registry directory: `dep_config.yaml` + hooks for one dependency. |
| **Hook** | One of `pre_load.cmake`, `post_load.cmake`, `custom_populate.cmake`, `Find<X>.cmake`. |
| **Phase A / Phase B** | The import block (§6.2) / the apply block (§6.5) of generated code. |
| **Shape** | One of the seven structural kinds of configuration (see [procedures](dependency_config_procedures.md#procedure-b--author-the-configuration-by-shape)). |
| **Problem class (`PC-*`)** | A named recurring problem with a standard solution (see [the catalog](dependency_problem_classes.md)). |
| **Usage conditional** | The generated `if(...)` that skips a dependency entirely when nothing enabled consumes it (§6.1). |
| **Install mode** | `FULL` (headers + dev artifacts) or `MINIMAL` (runtime only), per target and per predefined dependency (§6.4). |
| **Superbuild** | The consuming project plus all subdirectory dependencies and GCMake dependencies configured in one CMake run. |
| **Dep cache** | `~/.gcmake/dep-cache`, CPM's source cache shared across projects. |
