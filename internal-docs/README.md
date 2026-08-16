# GCMake Internal Documentation

This folder holds documentation for **developers of GCMake itself** — including maintainers of the
[gcmake-dependency-configs](https://github.com/scupit/gcmake-dependency-configs) registry. Nothing
here is required reading for people merely *using* GCMake; user-facing documentation lives in
[docs/](../docs/Docs_Home.md).

> **NOTE:** These documents govern the dependency configuration registry, but deliberately live in
> the `gcmake-rust` repository. The registry repo is cloned into every user's `~/.gcmake` directory,
> so internal docs placed there would be distributed to all users.

## The Predefined Dependency Documentation Set

Read in this order if you're new. Each document serves one distinct moment of work:

| Document | Kind | Open it when you're asking... |
| -------- | ---- | ----------------------------- |
| [Predefined Dependency System Reference](predefined_dependency_system_reference.md) | Descriptive | "How does this machinery actually work?" (onboarding, or debugging a hook) |
| [Dependency Problem Classes](dependency_problem_classes.md) | Descriptive | "What problems does this dependency have, and what's the known solution shape?" (analysis, review) |
| [Dependency Configuration Standards](dependency_config_standards.md) | Normative | "Is this configuration correct and idiomatic?" (writing, reviewing) |
| [Dependency Configuration Procedures](dependency_config_procedures.md) | Operational | "What do I do next?" (authoring a new config, updating an old one, verifying) |
| [Dependency Configuration Known Issues](dependency_config_known_issues.md) | Living backlog | "What's currently wrong, stale, or blocked on tool features?" |

**Agent entry point:** dependency-configuration work (new configs, updates, fixes) is driven by
the [`/dep-config` project skill](../.claude/skills/dep-config/SKILL.md), which routes into these
documents and defines the human-in-the-loop escalation protocol. Humans can read the documents
directly in the order below.

The documents cross-reference each other instead of repeating content:

- The **reference** explains the machinery once, so the other docs don't have to.
- The **problem classes** catalog names each recurring problem (`PC-*` IDs) and points at the
  standards rule and procedure template that solve it.
- The **standards** say what a conforming configuration must look like.
- The **procedures** say how to produce one, and carry the copyable templates.
- **Known issues** tracks divergence between the current registry and the standards.

## Maintenance Expectations

- When gcmake-rust changes anything the generated CMakeLists exposes to hook scripts (variable
  names, function signatures, splice ordering), update the reference doc **and** audit the registry
  for hooks depending on the old behavior. See the
  ["co-evolution rule"](dependency_config_standards.md#9-registry-hygiene--co-evolution) — this
  exact failure has happened before.
- When a new problem class is discovered, add it to the catalog with a new `PC-*` ID (IDs are
  kebab-case names, not numbers, so insertions never renumber existing entries).
- When a defect is fixed or a workaround is obsoleted by a tool feature, update Known Issues.
