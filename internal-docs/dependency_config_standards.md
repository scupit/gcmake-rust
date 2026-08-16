# Dependency Configuration Standards

Normative requirements for configurations in the
[gcmake-dependency-configs](../gcmake-dependency-configs/) registry. **MUST**/**MUST NOT** are hard
requirements; **SHOULD** requires a written in-place justification to deviate; **MAY** is
discretionary.

Background lives in the [system reference](predefined_dependency_system_reference.md); the *why*
behind most rules lives in the [problem-class catalog](dependency_problem_classes.md); the *how*
lives in the [procedures](dependency_config_procedures.md). Existing configs that violate these
rules are tracked in [known issues](dependency_config_known_issues.md) — new work MUST NOT copy a
violation from an existing config, even a config this document cites as an exemplar.

## 1. Conformance and scope

- **1.1** These standards apply to every file in a registry configuration directory:
  `dep_config.yaml`, `pre_load.cmake`, `post_load.cmake`, `custom_populate.cmake`,
  `Find<X>.cmake`, `README.md`.
- **1.2** A configuration MUST be derivable from a completed
  [analysis worksheet](dependency_config_procedures.md#procedure-a--analyze-the-dependency): every
  hook file and every non-required YAML field corresponds to an identified problem class, and
  every identified class is addressed.

## 2. Choosing the dependency type

- **2.1** Use `as_subdirectory` when the dependency builds correctly via
  `add_subdirectory`/CPM with tolerable build cost, or when it has no build system at all
  (`requires_custom_fetchcontent_populate: true`). This is the default choice: it gives users
  zero-setup, version-pinned builds.
- **2.2** Use a module type (`cmake_module` / `cmake_components_module`) when the dependency is a
  system facility (opengl, threads, openmp), a toolkit installation (cuda), or a project whose
  build cost makes per-project rebuilds unreasonable (sdl2 — confirmed to be purely a build-cost
  decision, not a technical blocker; wxwidgets follows the same reasoning).
- **2.3** `module_type` selection MUST follow the escalation ladder of
  [PC-finder-strategy](dependency_problem_classes.md#pc-finder-strategy): dep's own package config
  (`ConfigFile`) → working builtin module (`BuiltinFindModule`) → wrapper module → from-scratch
  module (`CustomFindModule`). Skipping a rung MUST be justified in a comment (see glew's recorded
  rejection of builtin `FindGLEW`).
- **2.4** Use `cmake_components_module` (not `cmake_module`) whenever the underlying
  `find_package` call meaningfully takes `COMPONENTS`.
- **2.5** A config MAY define multiple type sections, but note that `as_subdirectory` currently
  always wins selection; don't add a second section that can never be chosen without also filing
  the tool gap.
- **2.6** When no consumable upstream exists (a bare compiler/tool, or a missing abstraction),
  author a **wrapper repository** under GCMake ownership and configure it as a plain subdirectory
  dep ([Procedure E](dependency_config_procedures.md#procedure-e--author-a-wrapper-repository)).
  Do not encode a fake build for someone else's project directly in hooks.

## 3. Naming and casing

- **3.1** Configuration directory names MUST be lowercase. The directory name is the user-facing
  dependency name, the YAML linking namespace, and the install-mode variable key.
- **3.2** Exposed target names (the `target_configs`/`targets`/`components` keys) MUST be
  lowercase. `actual_target_name` records the real, correctly-cased upstream name.
- **3.3** `module_name` MUST exactly match the real `find_package` name, including case
  (`wxWidgets`, `CUDAToolkit`, `OpenSSL`).
- **3.4** Install-mode gates MUST use the exact lowercase directory name:
  `PROJECT_<dir_name>_INSTALL_MODE` (e.g. `PROJECT_openssl_INSTALL_MODE`). CMake variables are
  case-sensitive; a mis-cased gate silently never fires. (This exact defect exists in eight
  pre-2022-migration configs — see
  [known issues](dependency_config_known_issues.md#confirmed-defects).)
- **3.5** Hook-internal cache variables MUST be prefixed to avoid collisions: `GCMAKE_` for values
  users might inspect (`GCMAKE_ZSTD_DLL`), a leading underscore for internals
  (`_zstd_shared_import_lib`), and `_<X>_PREVIOUSLY_SEARCHED_FOR_*` for re-search detection.
  Internal cache entries SHOULD be `mark_as_advanced` or `CACHE INTERNAL`.
- **3.6** Idempotency flags MUST follow existing conventions: `ALREADY_CONFIGURED_<DEPNAME>`
  (post_load), `NOT TARGET <name>` (target creation), copy targets named
  `copy-<dep>-<what>` or `_copy_<dep>_dlls`.
- **3.7** Finder hint variables: when wrapping or using a builtin module, use the **builtin's own
  documented hint names** (`ZLIB_USE_STATIC_LIBS`, `OPENSSL_USE_STATIC_LIBS`,
  `GLEW_USE_STATIC_LIBS`, `THREADS_PREFER_PTHREAD_FLAG`). For from-scratch finders, use
  `<UPPERNAME>_ROOT` for the search path and `<UPPERNAME>_PREFER_STATIC` for static preference
  (zstd, brotli). Do not invent a third spelling.

## 4. dep_config.yaml standards

- **4.1** Required fields per type follow the schema
  ([reference §8](predefined_dependency_system_reference.md#8-dep_configyaml-field-reference));
  the YAML deserializer rejects unknown fields, so the schema is authoritative.
- **4.2** Subdirectory deps MUST set `can_cross_compile` honestly — verified, not assumed. If
  untested, set `false` with a TODO comment (see glfw's).
- **4.3** Subdirectory deps SHOULD provide both `git_method` and `url_method` when upstream
  publishes archives; MUST provide `url_method` when the git history is punishingly large
  (nlohmann_json) or when only archives exist (sqlite3, glew snapshots).
- **4.4** Module-type configs MUST set `links.gcmake_readme` (it's where the generated
  "not found" error sends users) and SHOULD set `links.cmake_find_module` for builtin modules.
  Subdirectory configs SHOULD set `links.github`.
- **4.5** `found_var` MUST be the module's real success variable, verified against the module's
  documentation or source — conventions vary (`OPENGL_FOUND`, `libwebsockets_FOUND`,
  `zstd_FOUND`).
- **4.6** Every install toggle upstream provides MUST be declared (`install_var` /
  `inverse_install_var`) rather than set manually in hooks — the writer's FULL/MINIMAL wiring
  depends on it ([PC-install-toggle](dependency_problem_classes.md#pc-install-toggle)). Test
  frameworks and other never-shipped deps MUST set `install_by_default: false`.
- **4.7** Flavor targets that cannot be linked together MUST be declared `mutually_exclusive`.
  Prerequisites MUST be declared with `requires` (use `a or b` for "any one satisfies"; order
  carries no meaning). Requirements on other registry deps MUST use `external_requires`, never a
  hook-side hack.
- **4.8** Platform- or toolchain-limited targets MUST carry constraint prefixes
  (`((windows))`, `(( cuda ))`) rather than relying on runtime failure. Exception: targets that
  are inert no-ops elsewhere MAY stay unconstrained with a comment (the emscripten shim's
  documented decision).
- **4.9** `debian_packages` SHOULD be provided for module-type deps that Debian packages
  (runtime and dev separated correctly). Subdirectory deps SHOULD NOT list packages for the
  dependency itself (the project builds its own copy), but SHOULD list packages for **system
  libraries their targets link at runtime** (sfml's openal/vorbis/X11/freetype lists). The one
  existing self-listing (sqlite3) is a confirmed mistake tracked in
  [known issues](dependency_config_known_issues.md#minor-defects).
- **4.10** User-tunable passthrough values MUST go through `config_options` (with a
  `cache_description`) rather than instructing users to set raw cache variables.

## 5. Hook-file standards

### 5.1 All hooks

- **5.1.1 Idempotency.** Every hook MUST be safe to execute more than once per configure — hooks
  are re-spliced into every GCMake project in a superbuild
  ([reference §6](predefined_dependency_system_reference.md#6-stage-4-code-generation--anatomy-of-the-generated-blocks)).
  Target creation guarded by `NOT TARGET`; one-shot logic guarded by `ALREADY_CONFIGURED_<DEP>`.
- **5.1.2 No unguarded global mutation.** A hook MUST NOT permanently modify state outside its
  dependency's namespace (`CMAKE_MODULE_PATH` appends of dep-owned dirs are the sanctioned
  exception). Anything temporarily modified (e.g. `CMAKE_FIND_LIBRARY_SUFFIXES`) MUST be saved
  and restored.
- **5.1.3 Contract discipline.** Hooks MUST only rely on the documented runtime contract
  ([reference §7](predefined_dependency_system_reference.md#7-the-configure-time-runtime-contract)),
  and MUST use the target-system variables (`TARGET_SYSTEM_IS_*`) for target-platform decisions
  vs. host variables (`CURRENT_SYSTEM_IS_*`) for configure/build-time-execution decisions.
- **5.1.4 Diagnostics.** Failure paths MUST `message( FATAL_ERROR ... )` with the exact path or
  value that was searched/expected, so users can act without reading the hook.

### 5.2 pre_load.cmake

- **5.2.1** pre_load is for things that must exist **before** the import: preempting upstream
  `option()` defaults, declaring finder hints, propagating project context, faking sibling
  discovery ([PC-sibling-discovery](dependency_problem_classes.md#pc-sibling-discovery)).
  It MUST NOT reference the dependency's targets or `<dep>_SOURCE_DIR` — they don't exist yet.
- **5.2.2 FORCE policy.** Preempt with non-forcing `option()` / `set(CACHE)` — first-set wins and
  users keep override power. `FORCE` MAY be used only when upstream force-sets or ignores the
  non-forced value, and MUST NOT be used on generically-named variables (`BUILD_EXAMPLES`)
  without an in-place comment weighing the collision risk (see raylib).
- **5.2.3** Cache descriptions MUST state GCMake provenance and the default:
  `"... GCMake sets this to OFF by default."`

### 5.3 post_load.cmake

- **5.3.1** post_load is for things that need the dependency's targets/results to exist: property
  repair, usage-requirement patching, module-path exposure, pollution cleanup, DLL distribution.
- **5.3.2** DLL distribution MUST follow the canonical recipe of
  [PC-dll-distribution](dependency_problem_classes.md#pc-dll-distribution) (template
  [T5](dependency_config_procedures.md#t5--canonical-dll-copyinstall-post_loadcmake)):
  target-system gate + idempotency flag; sharedness check; `find_file` pinned to the discovered
  installation with all implicit search paths disabled; hard-fail diagnostics; one guarded
  `ALL` copy target into `${MY_RUNTIME_OUTPUT_DIR}`; installation registration only under
  `if( DEFINED PROJECT_<dir_name>_INSTALL_MODE )` (rule 3.4 casing).
- **5.3.3** DLL copying SHOULD be user-disableable via a
  `<DEP>_WIN_SHOULD_COPY_DLL`-style cache option.
- **5.3.4** Property repair MUST tolerate partially-present targets (loop over candidates with
  `if( TARGET ... )` — googletest's target list varies with its options).
- **5.3.5** State restoration (pollution cleanup) MUST name the upstream file/line responsible in
  a comment, so the fix can be retired when upstream changes.

### 5.4 custom_populate.cmake

- **5.4.1** The script MUST set `<dep>_RELATIVE_DEP_PATH`, and the convention is `dep/<name>`
  ([reference §7.4](predefined_dependency_system_reference.md#74-dep_relative_dep_path)). Every
  `$<INSTALL_INTERFACE:...>` include dir MUST agree with it.
- **5.4.2** Source/header anchoring MUST use `${<dep>_SOURCE_DIR}` (the CPM checkout), never a
  path relative to the consuming project.
- **5.4.3** Every file list attached to a target MUST go through
  `gcmake_wrap_dep_files_in_generators`, and headers MUST be attached as
  `FILE_SET HEADERS` with `BASE_DIRS` so installation works
  ([reference §7.5](predefined_dependency_system_reference.md#75-file-list-generator-wrapping-gcmake-general-utilscmake)).
- **5.4.4** Include directories MUST be declared `SYSTEM` and MUST pair
  `$<BUILD_INTERFACE:...>` with the matching `$<INSTALL_INTERFACE:...>`.
- **5.4.5** Targets MUST be aliased into the config's namespace
  (`add_library( <ns>::<name> ALIAS <name> )`) matching `namespace_config`.
- **5.4.6** Header existence SHOULD be validated with `find_file( ... REQUIRED NO_CACHE )` when
  upstream renames files across versions — give the candidate-name list (stb's
  `stb_image_resize.h;stb_image_resize2.h`).
- **5.4.7** Platform system libraries required by the sources MUST be linked here
  ([PC-system-libs](dependency_problem_classes.md#pc-system-libs)).
- **5.4.8** Optional executables (sqlite3's shell) MUST default OFF behind a namespaced option.

## 6. Custom find module standards

A registry find module runs in four contexts — generation-time project, installed project,
downstream consumer, and repeat-configure — and MUST behave identically in all
([reference §9](predefined_dependency_system_reference.md#9-custom-find-module-lifecycle)).

- **6.1 Header contract.** The module MUST begin with a comment block documenting its input
  hints, produced targets, and defined variables (see `Findzstd.cmake`).
- **6.2 Self-containment.** No references to registry paths, generation-time variables, or
  network resources. The module must work as an installed artifact on a stranger's machine.
- **6.3 Re-search invalidation.** If any hint (`<X>_ROOT`, static preference) could change the
  result, the module MUST detect the change (`_<X>_PREVIOUSLY_SEARCHED_FOR_*` INTERNAL cache
  pattern) and `unset( <result vars> CACHE )` before searching.
- **6.4 Wrapper anatomy.** A wrapper module MUST clear `CMAKE_MODULE_PATH` around its inner
  `include( Find<X> )` (it shadows the builtin by name — skipping this recurses), and restore it
  after.
- **6.5 From-scratch anatomy.** Try `find_package( <x> CONFIG )` first when upstream *can*
  install a config file; fall back to `find_path`/`find_library` +
  `find_package_handle_standard_args`. Alias every outcome to the **same** stable namespaced
  target so consumers never care which path found it.
- **6.6 Search hygiene.** Static/shared preference via prepended
  `CMAKE_FIND_LIBRARY_SUFFIXES` (with `.lib`/`.a`/`.dll.a` Windows handling), always saved and
  restored (rule 5.1.2). `NAMES_PER_DIR` when multiple candidate names exist.
- **6.7 Failure etiquette.** The module reports through its found variable / FPHSA; it MUST NOT
  `FATAL_ERROR` on not-found (the generated code owns that error and points at the README).
  Nonsense usage MAY warn (`AUTHOR_WARNING` for components passed to a component-less module).
- **6.8 Side-channels for distribution.** If the library can be shared on Windows, the module
  MUST export what the DLL step needs (import-lib path variables) rather than making post_load
  re-discover it.

## 7. README standards

- **7.1** Module-type deps (anything a user must install) MUST have a README containing: quick
  links, per-platform installation options (apt package line, MSYS2/vcpkg/choco guidance where
  known), and — when building from source is expected — the exact verified commands.
- **7.2** Verified commands MUST state the toolchain they were verified with
  ("I use Ninja with the MSYS2 MinGW GCC 12.2.0 distribution. Your compiler and build tool may
  vary.").
- **7.3** Non-default configure flags MUST come with a reason, table-form when there are several
  (lws).
- **7.4** Branch/version requirements MUST state the consequence and the cause (yaml-cpp, crow,
  sfml, glm). Configs whose misuse *breaks user builds* MUST also be linked from the
  [registry README's IMPORTANT NOTES](../gcmake-dependency-configs/README.md).
- **7.5** Known troubleshooting (hint variables, stale-state traps) SHOULD be recorded the first
  time it costs anyone an hour (`OPENSSL_ROOT_DIR`, `wxWidgets_LIB_DIR`, lws's build-dir
  find_package trap).

## 8. Annotation and provenance

- **8.1** Every non-obvious decision MUST be commented **in place**: why an option is set, why a
  fork/branch is used, why an alternative was rejected (glew's builtin-module note), why a flag
  is or isn't `FORCE`d, which upstream file a trick targets (raylib naming `GlfwImport.cmake`).
  The registry's greatest existing strength is that it is self-documenting; keep it that way.
- **8.2** Known limitations and wish-list items MUST be recorded as in-place `TODO:` comments
  *and* mirrored in [known issues](dependency_config_known_issues.md) when they depend on
  gcmake-rust features.
- **8.3** Anything GCMake sets on the user's behalf MUST say so in its cache description
  (rule 5.2.3), so `cmake-gui`/`ccmake` users can tell GCMake's decisions from upstream defaults.

## 9. Registry hygiene & co-evolution

- **9.1 The co-evolution rule.** When gcmake-rust changes any name or behavior the hooks depend
  on (generated variable names, utility function signatures, splice ordering), the change is not
  done until the registry has been audited: `grep -r` the registry for the old name. The
  install-mode casing defect happened because the Nov 2022 lowercase migration renamed the
  generated variables and the hooks were never audited.
- **9.2** A configuration update MUST record (in the commit message) which upstream version(s) it
  was written and verified against — future archaeology
  ([Procedure C](dependency_config_procedures.md#procedure-c--update-an-existing-configuration))
  depends on it.
- **9.3** Registry changes MUST be verified per
  [Procedure D](dependency_config_procedures.md#procedure-d--verify-a-configuration) before
  merging to `develop` — users track `develop` by default via `gcmake-rust dep-config update`.
- **9.4** Version pins in examples/READMEs SHOULD be refreshed when a config is touched; a config
  untouched for years is presumed stale until re-verified (see the
  [staleness table](dependency_config_known_issues.md#staleness)).

## 10. Compliance checklists

### Every configuration

- [ ] Derivable from a completed analysis worksheet (1.2); all firing problem classes addressed
- [ ] Directory, target names lowercase; real names in `actual_target_name` (3.1–3.3)
- [ ] All install-mode gates use `PROJECT_<dir_name>_INSTALL_MODE`, exact lowercase (3.4)
- [ ] Links present (4.4); `can_cross_compile` verified (4.2)
- [ ] Requirements, exclusions, platform gates declared in YAML, not hooks (4.7, 4.8)
- [ ] All hooks idempotent; global state saved/restored (5.1.1, 5.1.2)
- [ ] Every non-obvious decision commented in place (8.1)
- [ ] Commit message records upstream version verified against (9.2)
- [ ] Verified per Procedure D (9.3)

### Additional, by shape

**Subdirectory (plain):** install toggle declared (4.6) · option hygiene complete — every
upstream `option()` reviewed (5.2) · download methods per 4.3.

**Subdirectory (custom populate):** `<dep>_RELATIVE_DEP_PATH` set to `dep/<name>` and consistent
with all INSTALL_INTERFACE dirs (5.4.1) · generator wrapping + FILE_SET everywhere (5.4.3) ·
SYSTEM includes, namespace aliases (5.4.4, 5.4.5) · platform system libs (5.4.7).

**Module (ConfigFile/Builtin):** `module_name` and `found_var` verified (3.3, 4.5) · hint
variables declared in pre_load with the module's own names (3.7) · README build/install
instructions (7.1) · DLL post_load if the dep can be shared on Windows (5.3.2).

**Module (custom find module):** header contract (6.1) · four-context self-containment (6.2) ·
re-search invalidation (6.3) · wrapper recursion guard or from-scratch dual-mode (6.4, 6.5) ·
no FATAL_ERROR on not-found (6.7) · DLL side-channels (6.8).
