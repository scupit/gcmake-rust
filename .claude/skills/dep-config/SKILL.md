---
name: dep-config
description: Author, update, fix, or verify a predefined dependency configuration in the gcmake-dependency-configs registry. Use when asked to add GCMake support for a C/C++ library, update or fix an existing dependency configuration (including known-issues items), or re-verify a stale one. Inputs — new configs need a dependency name and upstream repo URL; updates need the dependency name.
argument-hint: "new <name> <repo-url> | update <name> [target-version] | (no args: triage candidates)"
---

# Dependency Configuration Work Protocol

You are working on the **predefined dependency compatibility layer**: per-library adapter
configurations in the `gcmake-dependency-configs/` submodule that make third-party C/C++
projects consumable by GCMake. The complete knowledge base for this work lives in
`internal-docs/` — this skill tells you where to start, which document answers which question,
and when to hand a decision back to the human. **Do not improvise procedure or style: everything
has a documented standard, and deviations get flagged in review.**

## Step 0 — Orient (always, before touching anything)

1. Read [internal-docs/README.md](../../../internal-docs/README.md) for the document map, then:
   - [dependency_config_procedures.md](../../../internal-docs/dependency_config_procedures.md) —
     the workflows you will follow (Procedures A–E) and the copyable templates (T1–T8).
   - [dependency_config_standards.md](../../../internal-docs/dependency_config_standards.md) —
     normative rules; your work must pass its §10 checklist for the relevant shape.
   - [dependency_problem_classes.md](../../../internal-docs/dependency_problem_classes.md) —
     consult during analysis; every hook you write must correspond to a `PC-*` class.
   - [predefined_dependency_system_reference.md](../../../internal-docs/predefined_dependency_system_reference.md) —
     consult when you need to know how the machinery works (splice points, runtime contract,
     YAML schema).
   - [dependency_config_known_issues.md](../../../internal-docs/dependency_config_known_issues.md) —
     **check it first for the dependency you're touching**; it may already list defects, gaps, or
     a refresh priority for it.
2. Confirm the working setup:
   - Registry checkout: `gcmake-dependency-configs/` (git submodule, normally on `develop`).
     Edit configs **here**.
   - Upstream source clones go in `reference-repos/` (gitignored for this purpose). Prefer
     `git clone --filter=blob:none`.
   - Tool: `cargo build`, then use `target/debug/gcmake-rust`.
   - End-to-end verification uses **scratch projects** in a temp directory outside this repo.
     `gcmake-test-project/` is stale and not a usable harness.

## Step 1 — Resolve the mode and inputs

| Mode | Required from the human | If missing |
| ---- | ----------------------- | ---------- |
| `new <name> <repo-url>` | Registry directory name (must be lowercase, must not already exist) and the upstream repository URL | **Ask.** Never guess which of several similarly-named repos/forks is intended. |
| `update <name> [target-version]` | Existing config name; optionally the upstream version to target | Target version may default to latest stable — state your choice in the report. |
| *(no args)* | — | Triage: summarize the known-issues refresh priorities and confirmed defects, recommend one, and ask which to work on. |

A "fix" request (e.g. working a known-issues entry) is `update` mode; the known-issues entry is
your starting worklist, but still re-verify its claims against the current code first.

## Step 2 — Execute the documented procedure

Follow [the procedures document](../../../internal-docs/dependency_config_procedures.md) exactly;
in summary:

- **New config:** Procedure A (produce the nine-question analysis worksheet — it identifies the
  shape and the firing problem classes) → Procedure B playbook for that shape, using the
  templates → Procedure D verification → standards §10 checklist.
- **Update:** Procedure C (era archaeology: `git log -1 -- <dir>/` in the registry, check out the
  upstream commit from that date in `reference-repos/`, diff upstream's CMake forward) →
  re-run worksheet questions whose answers changed → apply per shape playbook → Procedure D →
  checklist.
- **Wrapper repo needed** (Procedure E): stop and escalate — see triggers below.

Working-tree mechanics for verification (Procedure D):

1. The running tool reads the registry at `~/.gcmake/gcmake-dependency-configs`, not this
   submodule. Before end-to-end tests: check that copy is clean
   (`git -C ~/.gcmake/gcmake-dependency-configs status`), then copy your edited `<dep>/`
   directory over its counterpart there.
2. After verification, restore it: `git -C ~/.gcmake/gcmake-dependency-configs checkout -- <dep>`
   (or leave in place only if the human says so).
3. Run every verification-matrix row the current machine supports. Rows you cannot run (missing
   toolchain, would require system-wide installs) are **reported as skipped with reasons**, never
   silently passed.

## Step 3 — Documentation upkeep (part of the work, not optional)

- Every non-obvious decision gets an in-place comment (standards §8), including rejected
  alternatives.
- Update [known issues](../../../internal-docs/dependency_config_known_issues.md): mark fixed
  items, add newly discovered defects/gaps, refresh the staleness row for the config you touched.
- If you discovered a genuinely new recurring problem, add a `PC-*` entry to the catalog and its
  row in the reverse index (per internal-docs/README.md maintenance expectations).
- Commit **in the submodule** with a message that names the upstream version(s) you verified
  against (standards 9.2). Leave the parent repo's submodule pointer for the human unless asked.

## Human-in-the-loop protocol

Default posture: **complete the work autonomously.** Do not ask about anything answerable from
the internal docs, the standards, upstream source, the registry's git history, or by running a
verification — go find out. When you do escalate, present the specific decision, the options,
and your recommended choice with reasoning.

**Stop and ask the human when:**

1. **Inputs are missing or ambiguous** — no repo URL for a new config; multiple plausible
   upstreams/forks; the requested name collides with an existing config.
2. **Shape choice is a policy call, not a technical one** — e.g. subdirectory vs. system-installed
   module where the deciding factor is build-cost tolerance (standards 2.2 territory). Present
   the trade-off; the maintainer sets the policy.
3. **The fix ladder requires resources you can't create** — forking upstream, creating a wrapper
   repository under the maintainer's account, or filing an upstream PR (PC-broken-upstream
   rungs 2–3). Prepare the patch/design; the human executes the ownership step.
4. **Verification is blocked** — a required toolchain isn't installed, or a test would require a
   system-wide install (never install anything system-wide without approval). Ask whether to
   proceed with the reduced matrix or wait.
5. **A standards rule appears wrong or incomplete for this case** — propose the amendment;
   never silently deviate.
6. **The change would break existing users** — renaming exposed target names, namespaces, or
   YAML keys that appear in users' `cmake_data.yaml` files.
7. **The correct fix lives in gcmake-rust itself** (the writer or CMake utility library), not the
   registry — propose it and get approval before touching tool code; the co-evolution rule
   (standards 9.1) applies to whoever implements it.
8. **Anything outward-facing** — never `git push` (registry or parent repo) without explicit
   approval; a push ships to every user on their next `dep-config update`.

## Completion report

End with a report containing, in order:

1. **What was done** — one paragraph, mode + dependency + shape.
2. **Worksheet summary** (new configs) or **era diff summary** (updates): the key findings and
   the `PC-*` classes addressed, with one line each on how.
3. **Files created/changed** in the registry.
4. **Standards checklist result** for the shape (pass, or itemized deviations with reasons).
5. **Verification matrix**: each row run → result; each row skipped → reason.
6. **Escalations** raised and how they were resolved.
7. **Documentation updates** made (known issues, catalog, comments) and the exact commit
   message used (naming the verified upstream version).
8. **Follow-ups** for the human: anything deferred, blocked, or recommended next.
