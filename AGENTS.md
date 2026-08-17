# Agent Instructions

These instructions apply to all writing in this repository: code comments, documentation,
and anything else meant to be read by a person.

## Match the project's existing writing style

Before writing comments or documentation, read a few existing ones near where you're
working and write in the same voice. Writing in this project uses plain, complete
sentences that explain reasoning directly, and walks through concrete situations rather
than abstract descriptions. The comment on `GatedRequirementGuard` in
`src/project_info/dependency_graph_mod/dependency_graph.rs` and the pages in
`docs/cmake_data_config/` are good references for that voice.

## Writing Code Comments

Comments are read by someone who just landed on that line of code. They haven't read the
surrounding comments, and they didn't watch the code being written. So before keeping a
comment, read it by itself with the code hidden. If someone who understands the project
in general (but not this specific code) can't tell what the code accomplishes and why it
matters, rewrite the comment.

### A comment should answer these questions, in this order

1. **What problem does this code solve?** Don't describe what the code mechanically
   does. If the purpose is simple, state it simply: "Stops conditionally-required
   dependencies from being downloaded when they aren't needed" is better than three
   sentences that circle the same point.
2. **When does this matter?** Give one concrete scenario.
3. **Why was it done this way?** Only answer this when there's another obvious way to
   write the code and a reader would wonder why it wasn't used. This should never be the
   only thing a comment says. A comment like "must use X here, not Y, otherwise Z
   happens" makes the reader work out the code's purpose from a description of failure.

If none of these questions has a useful answer, the comment isn't needed.

### Writing rules

- The first sentence states the code's purpose. Details come after it.
- Don't invent shorthand terms. Phrases like "it gates X" or "requirement-satisfaction
  semantics" name an idea without explaining it. Use the project's real vocabulary:
  features, targets, constraints, the generated CMake, downloading, linking, installing.
- To shorten a comment, remove whole sentences that don't teach the reader anything new.
  Don't compress sentences into fragments; everything that stays should be a complete
  sentence.
- When a variable is created in one place and used in another, put the full explanation
  where the variable is created. Where it's used, explain only what that particular use
  accomplishes. Don't repeat the explanation in both places, and don't narrate what the
  code visibly does.
- Describe consequences concretely. "Otherwise OpenSSL is downloaded and linked even
  when Crow's 'ssl' feature is disabled" is useful. "Otherwise things break" is not.
- When behavior splits into several cases, list the cases as short bullets. The
  `GatedRequirementGuard` comment referenced above shows this done well.

### Examples inside comments

Build examples from real, existing parts of the project: actual dependencies from the
registry, actual cmake_data.yaml properties, actual generated CMake behavior. Don't
invent hypothetical "foo/bar" scenarios when a real one exists.

Every example must make sense on its own. Set the scene inside the example itself: say
whose project this is and what role each named thing plays. For example, "in a project
that uses the Crow predefined dependency, this makes the generated CMake skip OpenSSL
unless Crow's 'ssl' feature is enabled" can be understood by itself. Without the leading
clause, it only makes sense if a nearby comment already explained the situation — and
readers don't read nearby comments.

### Worked example

The same line of code, through a real review cycle.

Bad. The invented terms ("gates", "requirement-satisfaction semantics") name ideas
without explaining them, so the reader learns nothing:

```rust
// This must be preserved: it gates both the auto-generated link and the
// requirement-satisfaction semantics.
```

Bad. This only justifies a choice, so the reader has to work out what the code is for
from a description of what would go wrong:

```rust
// The auto-created link must use the full requirement_product, not just the outer
// link's constraint. Otherwise a feature-conditional requirement would be downloaded
// and linked even while its feature is disabled.
```

Good. Purpose first, then one example that makes sense on its own:

```rust
// Stops conditionally-required dependencies from being downloaded, linked,
// and installed when they aren't actually needed. For example, in a project
// that uses the Crow predefined dependency, this is what makes the generated
// CMake skip OpenSSL entirely unless Crow's 'ssl' feature is enabled.
```

### Final check

Hide the code and read the comment again. By itself, it should tell you what problem the
code solves and when that matters.
