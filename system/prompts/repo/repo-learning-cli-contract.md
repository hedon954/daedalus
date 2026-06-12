# Repo Learning CLI Contract

## Purpose

Use this contract when an Agent needs deterministic daedalus lifecycle, topic, review, knowledge, render, or validation commands.

The CLI manages structure, state, indexes, and validation. It does not generate learning conclusions by itself; durable learning content comes from user-owned notes, user closeout retrospective, and Agent challenge/organization.

## Project And Topic Commands

```text
daedalus init repo-learning <project-name> --topic <topic-slug> --title <topic-title>
daedalus topic new <topic-slug> --title <topic-title>
daedalus topic activate <topic-slug>
daedalus topic await-reflection <topic-slug> --reason <reason>
daedalus topic complete <topic-slug> --reason <reason>
daedalus task complete --reason <reason>
```

Interpretation:

```text
topic await-reflection releases the daily active slot after stages 01-09 are done, while preserving one pending closeout debt.
topic complete closes one topic after user closeout retrospective and knowledge archival are done.
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

If unsure which topic is active, first resolve the project/topic from `workspaces/.daedalus/current.toml` or `workspaces/current-topic`, then read project `.daedalus/topic-board.md` and project/topic `.daedalus/state.toml` before running lifecycle commands.

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

Knowledge CLI commands do not extract, inspect, promote, or reorganize understanding. The CLI only provides stable structure, templates, indexes, link checks, and validation.
