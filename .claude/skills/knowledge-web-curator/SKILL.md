---
name: knowledge-web-curator
description: Use when evolving apps/knowledge-web after a topic closeout or knowledge-base update. Maintains the interactive knowledge atlas as a curated learning product, not a Markdown projection.
---

# Knowledge Web Curator

Use this skill when updating `apps/knowledge-web`.

## Role

The knowledge web is an interactive learning atlas. It should help the user understand mechanisms that are hard to grasp from Markdown alone.

Do not build a documentation mirror. Do not automatically convert Markdown into pages. Read the knowledge sources, understand the mechanism, then decide whether an interaction, comparison, state machine, timeline, or visual model would make the knowledge easier to learn.

The web version must not be shallower than Markdown. It should add learning value through readable visual structure, mechanism diagrams, decision tables, scenario walkthroughs, source traceability, and interaction. Prefer a bright, readable theme unless the user explicitly asks for a dark theme.

## Inputs

Before editing, inspect the relevant current sources:

- `knowledge-base/**/*.md`
- current or closeout topic `reflection/candidate-map.md`
- current or closeout topic `reflection/closeout.md`
- relevant demo code and guides only when they explain the mechanism

## Iteration Rule

For every proposed website change, state the learning value in one sentence:

```text
This interaction helps the learner understand <mechanism> by letting them <action>.
```

If that sentence is weak, do not add the UI.

## What Belongs On The Web

Good additions:

- state machine walkthroughs
- policy comparison controls
- event timelines
- layered architecture diagrams with selectable layers
- decision tables that expose trade-offs and boundaries
- first-principles mechanism diagrams that clarify why the system exists
- source trace links
- small scenario simulators

Poor additions:

- raw Markdown rendering
- long prose copied from notes
- generic slogans about learning
- decoration that does not clarify the mechanism
- shallow cards that summarize the Markdown with less detail
- every knowledge-base entry as a page by default

## Required Checks

After editing:

```bash
make knowledge-web-check
```

Then run the local site and verify in the browser:

- first screen is about the knowledge being learned, not about the website philosophy
- visual theme is bright/readable unless the user requested otherwise
- the page contains enough mechanism detail that it can teach without opening the Markdown beside it
- important mechanisms have diagrams, comparisons, or walkthroughs rather than plain summary cards
- at least one interaction changes the visible explanation
- source links are present for traceability
- mobile width has no horizontal overflow

## Commit Hygiene

Do not commit generated dependency/build artifacts:

```text
apps/knowledge-web/node_modules/
apps/knowledge-web/dist/
```
