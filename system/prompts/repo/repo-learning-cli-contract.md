# Repo Learning CLI Contract

## Purpose

Use this contract when an Agent needs deterministic daedalus lifecycle, topic, review, knowledge, render, or validation commands.

The CLI manages structure, state, indexes, and validation. It does not generate learning conclusions by itself; content still comes from Agent coaching plus user calibration.

## Project And Topic Commands

```text
daedalus init repo-learning <project-name> --topic <topic-slug> --title <topic-title>
daedalus topic new <topic-slug> --title <topic-title>
daedalus topic activate <topic-slug>
daedalus topic complete <topic-slug> --reason <reason>
daedalus task complete --reason <reason>
```

Interpretation:

```text
topic complete closes one topic.
task complete closes the whole learning project.
```

## Stage Commands

```text
daedalus state enter <stage-id>
daedalus state complete <stage-id>
daedalus state block <stage-id> --reason <reason>
daedalus state resume <stage-id> --reason <reason>

daedalus state enter <stage-id> --topic <topic-slug>
daedalus state complete <stage-id> --topic <topic-slug>
```

Interpretation:

```text
state complete <stage> completes the active topic's stage unless --topic is provided.
```

If unsure which topic is active, read project `.daedalus/topic-board.md` and `.daedalus/state.toml` before running lifecycle commands.

## Render And Validate

```text
daedalus state render
daedalus state render --project
daedalus validate
daedalus validate --all-topics
daedalus topic validate <topic-slug>
```

Use validation after lifecycle, template, migration, review, or knowledge structure changes.

## Review Commands

```text
daedalus review start --topic <topic-slug> --mode <mode> --goal <goal>
daedalus review session start <review-id>
daedalus review session complete <review-id> --reason <reason>
daedalus review complete <review-id> --reason <reason>
daedalus review validate <review-id>
```

Review is attached to a topic or project. It must not reopen or mutate completed learning lifecycle by accident.

## Knowledge CLI Foundation

```text
daedalus knowledge validate
daedalus knowledge index
daedalus knowledge link-check
daedalus knowledge template
```

Knowledge extraction, inspection, gap analysis, promotion, and reorganization are skill workflows, not deterministic CLI commands. The CLI only provides stable structure, templates, indexes, link checks, and validation.
