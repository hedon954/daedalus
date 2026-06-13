use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;
use toml_edit::DocumentMut;

fn repo_fixture() -> TempDir {
    let temp = TempDir::new().expect("temp dir");
    let repo = temp.path();
    fs::create_dir_all(repo.join("system/templates")).expect("templates dir");
    fs::create_dir_all(repo.join("workspaces/projects")).expect("workspaces dir");
    fs::write(repo.join("CLAUDE.md"), "# Test Daedalus Project\n").expect("root claude");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_template = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .join("system/templates/repo");
    copy_dir(&source_template, &repo.join("system/templates/repo"));
    let source_topic_template = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .join("system/templates/repo-topic");
    copy_dir(
        &source_topic_template,
        &repo.join("system/templates/repo-topic"),
    );
    let source_review_template = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .join("system/templates/review");
    copy_dir(
        &source_review_template,
        &repo.join("system/templates/review"),
    );
    let source_knowledge_entry_template = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .join("system/templates/knowledge");
    copy_dir(
        &source_knowledge_entry_template,
        &repo.join("system/templates/knowledge"),
    );
    temp
}

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("target dir");
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.expect("walk entry");
        let relative = entry.path().strip_prefix(source).expect("relative path");
        let target_path = target.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path).expect("dir");
        } else {
            fs::copy(entry.path(), target_path).expect("copy file");
        }
    }
}

fn active_topic_dir(task_dir: &Path) -> PathBuf {
    task_dir.join("topics/main")
}

fn write_core_topic_artifacts(topic_dir: &Path) {
    for path in [
        "guides/02-repo-scout/README.md",
        "notes/03-socratic-coach/README.md",
        "notes/04-debugger-guide/README.md",
        "notes/05-arch-analyzer/README.md",
        "notes/06-code-reader/README.md",
        "demo/design.md",
        "demo/README.md",
        "notes/09-biz-solver/README.md",
    ] {
        let path = topic_dir.join(path);
        fs::create_dir_all(path.parent().expect("parent")).expect("artifact parent");
        fs::write(path, "# artifact\n").expect("artifact");
    }
}

fn write_demo_manifest(topic_dir: &Path, package_name: &str) {
    let manifest = topic_dir.join("demo/Cargo.toml");
    fs::create_dir_all(manifest.parent().expect("demo dir")).expect("demo dir");
    fs::write(
        manifest,
        format!("[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\n"),
    )
    .expect("demo manifest");
}

fn rust_analyzer_linked_projects(repo_root: &Path) -> Vec<String> {
    let settings = fs::read_to_string(repo_root.join(".vscode/settings.json")).expect("settings");
    let value: serde_json::Value = serde_json::from_str(&settings).expect("settings json");
    value
        .get("rust-analyzer.linkedProjects")
        .and_then(|item| item.as_array())
        .expect("linked projects")
        .iter()
        .filter_map(|item| item.as_str().map(ToOwned::to_owned))
        .collect()
}

fn complete_core_topic_stages(repo: &TempDir, task_dir: &Path) {
    let stages = [
        "01-goal-aligner",
        "02-repo-scout",
        "03-socratic-coach",
        "04-debugger-guide",
        "05-arch-analyzer",
        "06-code-reader",
        "07-demo-architecture",
        "08-demo-coder",
        "09-biz-solver",
    ];
    for (index, stage) in stages.iter().enumerate() {
        if index > 0 {
            Command::cargo_bin("daedalus")
                .expect("binary")
                .current_dir(repo.path())
                .args([
                    "state",
                    "enter",
                    stage,
                    "--project-dir",
                    task_dir.to_str().expect("utf8"),
                    "--reason",
                    "Move test topic through core learning stages.",
                ])
                .assert()
                .success();
        }
        Command::cargo_bin("daedalus")
            .expect("binary")
            .current_dir(repo.path())
            .args([
                "state",
                "complete",
                stage,
                "--project-dir",
                task_dir.to_str().expect("utf8"),
                "--reason",
                "Core learning stage is complete for closeout lifecycle test.",
            ])
            .assert()
            .success();
    }
}

fn read_toml(path: &Path) -> DocumentMut {
    fs::read_to_string(path)
        .expect("toml file")
        .parse::<DocumentMut>()
        .expect("valid toml")
}

fn toml_string(doc: &DocumentMut, key: &str) -> String {
    doc.get(key)
        .and_then(|item| item.as_str())
        .unwrap_or_default()
        .to_owned()
}

fn project_active_topic(doc: &DocumentMut) -> String {
    doc.get("project")
        .and_then(|item| item.as_table())
        .and_then(|table| table.get("active_topic"))
        .and_then(|item| item.as_str())
        .unwrap_or_default()
        .to_owned()
}

fn topic_lifecycle(doc: &DocumentMut) -> String {
    doc.get("topic")
        .and_then(|item| item.as_table())
        .and_then(|table| table.get("lifecycle"))
        .and_then(|item| item.as_str())
        .unwrap_or_default()
        .to_owned()
}

fn project_topic_lifecycle(doc: &DocumentMut, slug: &str) -> Option<String> {
    doc.get("topics")
        .and_then(|item| item.as_array_of_tables())
        .and_then(|topics| {
            topics.iter().find_map(|topic| {
                (topic["slug"].as_str() == Some(slug))
                    .then(|| topic["lifecycle"].as_str().unwrap_or_default().to_owned())
            })
        })
}

fn first_review_id(reviews_root: &Path) -> String {
    let mut ids: Vec<String> = fs::read_dir(reviews_root)
        .expect("reviews root")
        .filter_map(|entry| {
            let entry = entry.expect("review entry");
            let path = entry.path();
            if path.is_dir() {
                path.file_name()
                    .and_then(|value| value.to_str())
                    .map(ToOwned::to_owned)
            } else {
                None
            }
        })
        .collect();
    ids.sort();
    ids.into_iter().next().expect("review id")
}

#[test]
fn command_rejects_non_daedalus_directory() {
    let temp = TempDir::new().expect("temp dir");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(temp.path())
        .args([
            "init",
            "repo-learning",
            "illegal",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "not a daedalus project directory",
        ));
}

#[test]
fn command_accepts_daedalus_subdirectory() {
    let repo = repo_fixture();
    let subdir = repo.path().join("system/templates");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(subdir)
        .args([
            "init",
            "repo-learning",
            "from-subdir",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("workspaces/projects/from-subdir"));
}

#[test]
fn init_repo_learning_creates_state_and_rendered_markdown() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "My Learning Task",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/my-learning-task");
    let topic_dir = active_topic_dir(&task_dir);
    assert!(task_dir.join("CLAUDE.md").exists());
    assert!(!task_dir.join(".daedalus/CLAUDE.md").exists());
    assert!(task_dir.join("shared/README.md").exists());
    assert!(task_dir.join("shared/source-index.md").exists());
    assert!(task_dir.join(".daedalus/reviews/README.md").exists());
    assert!(task_dir.join("topics/.gitkeep").exists());
    assert!(topic_dir.join(".daedalus/reviews/README.md").exists());
    assert!(topic_dir.join("demo/.gitkeep").exists());
    assert!(topic_dir.join("guides/.gitkeep").exists());
    assert!(topic_dir.join("reflection/README.md").exists());
    assert!(topic_dir.join("reflection/candidate-map.md").exists());
    assert!(topic_dir.join("notes/.gitkeep").exists());
    assert!(topic_dir.join("reflection/closeout.md").exists());
    assert!(task_dir.join("source/.gitignore").exists());
    assert!(task_dir.join("source/pull_source.sh").exists());
    assert!(task_dir.join(".daedalus/state.toml").exists());
    assert!(task_dir.join(".daedalus/project-map.md").exists());
    assert!(task_dir.join(".daedalus/topic-board.md").exists());
    assert!(topic_dir.join(".daedalus/outcome-map.md").exists());
    assert!(topic_dir.join(".daedalus/validation-log.md").exists());
    assert!(task_dir.join("CLAUDE.md").exists());
    assert!(task_dir.join(".daedalus/state.md").exists());
    assert!(
        repo.path()
            .join("workspaces/.daedalus/current.toml")
            .exists()
    );
    assert!(
        repo.path()
            .join("workspaces/.daedalus/project-index.toml")
            .exists()
    );
    assert!(repo.path().join("workspaces/current-project").exists());
    assert!(repo.path().join("workspaces/current-topic").exists());
    assert!(!repo.path().join("workspaces/02-learning").exists());
    assert!(!repo.path().join("workspaces/03-completed").exists());
    assert!(!repo.path().join("workspaces/04-abandoned").exists());
    assert!(repo.path().join(".ignore").exists());
    assert!(repo.path().join(".cursorignore").exists());
    assert!(repo.path().join(".claude/settings.json").exists());
    assert!(repo.path().join(".claude/settings.json").exists());
    assert!(topic_dir.join(".daedalus/task-card.md").exists());
    assert!(topic_dir.join(".daedalus/artifact-index.md").exists());
}

#[test]
fn ide_sync_rust_analyzer_updates_topic_demo_manifest() {
    let repo = repo_fixture();
    fs::create_dir_all(repo.path().join("crates")).expect("crates dir");
    fs::write(repo.path().join("crates/Cargo.toml"), "[workspace]\n").expect("crates manifest");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "Ide Demo",
            "--topic",
            "tools-permissions",
            "--title",
            "Tools Permissions",
        ])
        .assert()
        .success();

    let settings_dir = repo.path().join(".vscode");
    fs::create_dir_all(&settings_dir).expect("settings dir");
    fs::write(
        settings_dir.join("settings.json"),
        r#"{
  "editor.formatOnSave": true,
  "rust-analyzer.linkedProjects": [
    "crates/Cargo.toml",
    "workspaces/projects/ide-demo/demo/Cargo.toml"
  ]
}
"#,
    )
    .expect("settings");

    let demo_manifest = repo
        .path()
        .join("workspaces/projects/ide-demo/topics/tools-permissions/demo/Cargo.toml");
    fs::write(
        &demo_manifest,
        "[package]\nname = \"ide-demo\"\nversion = \"0.1.0\"\n",
    )
    .expect("demo manifest");

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["ide", "sync-rust-analyzer"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "ok: rust-analyzer linkedProjects synced",
        ))
        .stdout(predicates::str::contains(
            "workspaces/projects/ide-demo/topics/tools-permissions/demo/Cargo.toml",
        ));

    let settings = fs::read_to_string(settings_dir.join("settings.json")).expect("synced settings");
    assert!(settings.contains("\"editor.formatOnSave\": true"));
    let linked_projects = rust_analyzer_linked_projects(repo.path());
    assert!(linked_projects.contains(&"crates/Cargo.toml".to_owned()));
    assert!(linked_projects.contains(
        &"workspaces/projects/ide-demo/topics/tools-permissions/demo/Cargo.toml".to_owned()
    ));
    assert!(linked_projects.contains(&"workspaces/current-topic/demo/Cargo.toml".to_owned()));
    assert!(!linked_projects.contains(&"workspaces/projects/ide-demo/demo/Cargo.toml".to_owned()));
}

#[test]
fn review_start_creates_topic_review_without_reopening_learning_state() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "review-topic",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/review-topic");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "review",
            "start",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--topic",
            "main",
            "--mode",
            "rebuild",
            "--goal",
            "重建工具权限状态机。",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("action: review-start"));

    let reviews_root = active_topic_dir(&task_dir).join(".daedalus/reviews");
    let review_id = first_review_id(&reviews_root);
    let review_dir = reviews_root.join(&review_id);
    assert!(review_dir.join("state.toml").exists());
    assert!(review_dir.join("state.md").exists());
    assert!(review_dir.join("review-plan.md").exists());
    assert!(review_dir.join("mastery-map.md").exists());
    assert!(review_dir.join("question-bank.md").exists());
    let review_state = fs::read_to_string(review_dir.join("state.toml")).expect("review state");
    assert!(review_state.contains("target_type = \"topic\""));
    assert!(review_state.contains("target = \"main\""));
    assert!(review_state.contains("mode = \"rebuild\""));

    let topic_state = fs::read_to_string(active_topic_dir(&task_dir).join(".daedalus/state.toml"))
        .expect("topic state");
    assert!(topic_state.contains("lifecycle = \"active\""));
    assert!(topic_state.contains("current_phase = \"01-goal-aligner\""));

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "review",
            "list",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("topic:main"))
        .stdout(predicates::str::contains(&review_id));
}

#[test]
fn review_session_complete_requires_user_answers_before_review_completion() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "review-session",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/review-session");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "review",
            "start",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--topic",
            "main",
            "--mode",
            "recall",
            "--goal",
            "检查核心概念主动回忆。",
        ])
        .assert()
        .success();
    let reviews_root = active_topic_dir(&task_dir).join(".daedalus/reviews");
    let review_id = first_review_id(&reviews_root);

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "review",
            "session",
            "start",
            &review_id,
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--session-id",
            "first",
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "review",
            "session",
            "complete",
            &review_id,
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--session-id",
            "first",
            "--reason",
            "Session has been discussed.",
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "review",
            "complete",
            &review_id,
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Try to close without user answers.",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("lacks user answers"));

    let session_path = reviews_root
        .join(&review_id)
        .join("sessions")
        .join("first.md");
    let mut session = fs::read_to_string(&session_path).expect("session");
    session = session.replace(
        "## User Answers\n",
        "## User Answers\n\n用户能从现实制约解释核心抽象。\n",
    );
    session = session.replace(
        "## Calibration\n",
        "## Calibration\n\nAgent 校准：回答覆盖了不变量和 trade-off。\n",
    );
    fs::write(&session_path, session).expect("write session");

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "review",
            "complete",
            &review_id,
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Review session contains user answers and calibration.",
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "review",
            "validate",
            &review_id,
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("ok: review valid"));
}

#[test]
fn knowledge_template_index_link_check_and_validate() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "knowledge-flow",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/knowledge-flow");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "knowledge",
            "template",
            "pattern",
            "agent-command-safety",
            "--title",
            "Agent Command Safety",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("action: knowledge-template"));
    let pattern = repo
        .path()
        .join("knowledge-base/patterns/agent-command-safety.md");
    assert!(pattern.exists());
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["knowledge", "index"])
        .assert()
        .success()
        .stdout(predicates::str::contains("action: knowledge-index"));
    let index = fs::read_to_string(repo.path().join("knowledge-base/index.toml")).expect("index");
    assert!(index.contains("patterns/agent-command-safety.md"));

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["knowledge", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("knowledge: patterns"))
        .stdout(predicates::str::contains("Agent Command Safety"));

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["knowledge", "link-check"])
        .assert()
        .success()
        .stdout(predicates::str::contains("action: knowledge-link-check"));

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["knowledge", "extract"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unrecognized subcommand"));

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["validate", task_dir.to_str().expect("utf8"), "--knowledge"])
        .assert()
        .success();
}

#[test]
fn init_enforces_wip_one_by_default() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "first",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "second",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("task already active"));
}

#[test]
fn enter_preserves_state_toml_comments() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "comments",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/comments");
    let state_path = active_topic_dir(&task_dir).join(".daedalus/state.toml");
    let mut state = fs::read_to_string(&state_path).expect("state");
    state.push_str("\n# custom operator note\n");
    fs::write(&state_path, state).expect("write state");

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Move to repo selection after goal alignment.",
        ])
        .assert()
        .success();

    let updated = fs::read_to_string(state_path).expect("updated state");
    assert!(updated.contains("# custom operator note"));
    assert!(updated.contains("action = \"enter\""));
    assert!(updated.contains("actor = \"daedalus-cli\""));
}

#[test]
fn complete_rejects_missing_artifact_without_force() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "missing-artifact",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/missing-artifact");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Start repo scout before checking required artifacts.",
        ])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing required artifact"));
}

#[test]
fn complete_rejects_pending_stage() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "pending-complete",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/pending-complete");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid stage state transition"));
}

#[test]
fn resume_blocked_stage_keeps_single_active_stage() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "resume-single-active",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/resume-single-active");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Start repo scout.",
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "block",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Waiting for source access.",
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "03-socratic-coach",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Continue with question planning while source access is pending.",
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "resume",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Source access is restored.",
        ])
        .assert()
        .success();

    let state = fs::read_to_string(active_topic_dir(&task_dir).join(".daedalus/state.toml"))
        .expect("state");
    assert_eq!(state.matches("status = \"active\"").count(), 1);
    assert!(
        state.contains("id = \"02-repo-scout\"\ntitle = \"确认专题学习素材\"\nstatus = \"active\"")
    );
    assert!(state.contains(
        "id = \"03-socratic-coach\"\ntitle = \"提出专题递进问题\"\nstatus = \"blocked\""
    ));
}

#[test]
fn rollback_reopens_target_stage_and_resets_later_stages() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "rollback-stage",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/rollback-stage");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "01-goal-aligner",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Goal alignment is documented.",
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Start repo scout.",
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--force",
            "--reason",
            "Repo selection evidence exists outside the required artifact.",
            "--approval-source",
            "artifact-equivalent",
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "03-socratic-coach",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Start question roadmap.",
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "rollback",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Revisit repo selection assumptions before continuing.",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("action: rollback"))
        .stdout(predicates::str::contains("stage: 02-repo-scout"));

    let state = fs::read_to_string(active_topic_dir(&task_dir).join(".daedalus/state.toml"))
        .expect("state");
    assert_eq!(state.matches("status = \"active\"").count(), 1);
    assert!(state.contains("current_phase = \"02-repo-scout\""));
    assert!(
        state.contains("id = \"01-goal-aligner\"\ntitle = \"对齐专题学习目标\"\nstatus = \"done\"")
    );
    assert!(
        state.contains("id = \"02-repo-scout\"\ntitle = \"确认专题学习素材\"\nstatus = \"active\"")
    );
    assert!(state.contains(
        "id = \"03-socratic-coach\"\ntitle = \"提出专题递进问题\"\nstatus = \"pending\""
    ));
    assert!(state.contains("action = \"rollback\""));
    assert!(state.contains("next_action = \"已回退到 `02-repo-scout`"));

    let state_md = fs::read_to_string(active_topic_dir(&task_dir).join(".daedalus/state.md"))
        .expect("state.md");
    assert!(state_md.contains("当前阶段：`02-repo-scout`"));
    assert!(state_md.contains("状态：`active`"));
    assert!(state_md.contains("执行 `rollback`"));
}

#[test]
fn rollback_rejects_unreached_pending_stage() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "rollback-pending",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/rollback-pending");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "rollback",
            "03-socratic-coach",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Try to rollback to a stage that has not been reached.",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid stage state transition"));
}

#[test]
fn complete_updates_next_action_to_next_stage() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "next-action",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/next-action");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "01-goal-aligner",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Goal alignment artifacts are ready.",
        ])
        .assert()
        .success();

    let state = fs::read_to_string(active_topic_dir(&task_dir).join(".daedalus/state.toml"))
        .expect("state");
    assert!(state.contains("next_action = \"进入 `02-repo-scout`"));
    assert!(state.contains("current_phase = \"02-repo-scout\""));
    let state_md = fs::read_to_string(active_topic_dir(&task_dir).join(".daedalus/state.md"))
        .expect("state.md");
    assert!(state_md.contains("当前阶段：`02-repo-scout`"));
    assert!(state_md.contains("状态：`pending`"));
    assert!(state_md.contains("下一步：进入 `02-repo-scout`"));
}

#[test]
fn force_requires_reason_and_approval_source() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "force",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/force");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Start repo scout before testing force approval.",
        ])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--force",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("force requires approval"));

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "02-repo-scout",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--force",
            "--reason",
            "Repo selection was documented in .daedalus/decision-log.md.",
            "--approval-source",
            "artifact-equivalent",
        ])
        .assert()
        .success();

    let state = fs::read_to_string(active_topic_dir(&task_dir).join(".daedalus/state.toml"))
        .expect("state");
    assert!(state.contains("approval_source = \"artifact-equivalent\""));
}

#[test]
fn final_stage_completion_requires_active_final_stage() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "final-stage-pending",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/final-stage-pending");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "10-reflection",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Try to close before entering the final stage.",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid stage state transition"));

    assert!(task_dir.exists());
}

#[test]
fn final_stage_completion_completes_active_topic_stage_only() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "final-stage",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/final-stage");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "10-reflection",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Archive stage is ready to close.",
        ])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "10-reflection",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Archive artifacts are ready.",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("ok: state transition completed"))
        .stdout(predicates::str::contains("stage: 10-reflection"))
        .stdout(predicates::str::contains("topic_dir:"));

    assert!(task_dir.exists());
    assert!(task_dir.starts_with(repo.path().join("workspaces/projects")));
    let topic_state = fs::read_to_string(active_topic_dir(&task_dir).join(".daedalus/state.toml"))
        .expect("topic state");
    assert!(topic_state.contains("current_phase = \"10-reflection\""));
    assert!(
        topic_state
            .contains("id = \"10-reflection\"\ntitle = \"专题回顾与知识归档\"\nstatus = \"done\"")
    );
}

#[test]
fn task_complete_marks_project_idle_and_keeps_stable_path() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "close-me",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/close-me");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "10-reflection",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Archive stage is ready for task completion.",
        ])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "abandon",
            "main",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Main topic is intentionally closed for this project completion test.",
        ])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "task",
            "complete",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Dry run is archived and ready to close.",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("ok: task completed"))
        .stdout(predicates::str::contains("lifecycle: idle"))
        .stdout(predicates::str::contains("moved: false"));

    assert!(task_dir.exists());
    let state = fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("state");
    assert!(state.contains("lifecycle = \"idle\""));
    assert!(state.contains("workspace_bucket = \"projects\""));
    assert!(state.contains("active_topic = \"\""));
    assert!(state.contains("slug = \"main\""));
    assert!(state.contains("lifecycle = \"abandoned\""));
    let current =
        fs::read_to_string(repo.path().join("workspaces/.daedalus/current.toml")).expect("current");
    assert!(current.contains("current_project = \"projects/close-me\""));
    assert!(current.contains("current_topic = \"\""));
    assert!(!repo.path().join("workspaces/current-project").exists());
    assert!(!repo.path().join("workspaces/current-topic").exists());
    let decision_log =
        fs::read_to_string(task_dir.join(".daedalus/decision-log.md")).expect("decision log");
    assert!(decision_log.contains("决策：完成学习任务"));
    assert!(decision_log.contains("Dry run is archived and ready to close."));
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "new-task",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();
}

#[test]
fn task_abandon_marks_project_abandoned_and_clears_current() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "abandon-me",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/abandon-me");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "task",
            "abandon",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Dry run scope changed and this task should leave active WIP.",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("ok: task abandoned"))
        .stdout(predicates::str::contains("moved: false"));

    assert!(task_dir.exists());
    let state = fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("state");
    assert!(state.contains("action = \"abandon\""));
    assert!(state.contains("lifecycle = \"abandoned\""));
    assert!(state.contains("workspace_bucket = \"projects\""));
    assert!(state.contains("closed_at = "));
    assert!(state.contains("close_reason = "));
    assert!(state.contains("Dry run scope changed"));
    assert!(!state.contains("status = \"active\""));
    let current =
        fs::read_to_string(repo.path().join("workspaces/.daedalus/current.toml")).expect("current");
    assert!(current.contains("current_project = \"\""));
    assert!(current.contains("current_topic = \"\""));
    assert!(!repo.path().join("workspaces/current-project").exists());
    assert!(!repo.path().join("workspaces/current-topic").exists());
    let decision_log =
        fs::read_to_string(task_dir.join(".daedalus/decision-log.md")).expect("decision log");
    assert!(decision_log.contains("决策：放弃学习任务"));
    assert!(decision_log.contains("Dry run scope changed"));
}

#[test]
fn task_complete_rejects_project_with_unfinished_active_topic() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "collision",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/collision");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "task",
            "complete",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Try to close into an existing destination.",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("project has unfinished topics"));

    assert!(task_dir.exists());
    let state = fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("state");
    assert!(state.contains("lifecycle = \"active\""));
    assert!(state.contains("workspace_bucket = \"projects\""));
    assert!(!state.contains("lifecycle = \"idle\""));
}

#[test]
fn validate_reports_lifecycle_location_mismatch() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "lifecycle-mismatch",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/lifecycle-mismatch");
    let state_path = task_dir.join(".daedalus/state.toml");
    let state = fs::read_to_string(&state_path).expect("state");
    fs::write(
        &state_path,
        state.replace("lifecycle = \"active\"", "lifecycle = \"completed\""),
    )
    .expect("write state");

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["validate", task_dir.to_str().expect("utf8")])
        .assert()
        .failure()
        .stderr(predicates::str::contains("workspace validation failed"))
        .stderr(predicates::str::contains(
            "legacy task lifecycle `completed`",
        ));
}

#[test]
fn validate_reports_missing_required_file() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "validate",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/validate");
    fs::remove_file(active_topic_dir(&task_dir).join(".daedalus/todo.md")).expect("remove todo");

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["validate", task_dir.to_str().expect("utf8")])
        .assert()
        .failure()
        .stderr(predicates::str::contains("workspace validation failed"));
}

#[test]
fn topic_new_and_activate_manage_project_active_topic() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "topic-flow",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/topic-flow");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "new",
            "sub-agent",
            "--title",
            "Sub Agent",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("action: topic-new"));
    assert!(
        task_dir
            .join("topics/sub-agent/.daedalus/state.toml")
            .exists()
    );

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "activate",
            "sub-agent",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("action: topic-activate"));

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "list",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("sub-agent"))
        .stdout(predicates::str::contains("active"));

    let project_state =
        fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("project state");
    assert!(project_state.contains("active_topic = \"sub-agent\""));
    assert!(project_state.contains("slug = \"main\""));
    assert!(project_state.contains("lifecycle = \"blocked\""));
    assert!(project_state.contains("slug = \"sub-agent\""));
    assert!(project_state.contains("lifecycle = \"active\""));
}

#[test]
fn topic_await_reflection_releases_active_slot_and_preserves_closeout_pointer() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "closeout-flow",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/closeout-flow");
    let topic_dir = active_topic_dir(&task_dir);
    write_core_topic_artifacts(&topic_dir);
    write_demo_manifest(&topic_dir, "closeout-flow-demo");
    complete_core_topic_stages(&repo, &task_dir);

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "await-reflection",
            "main",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Core learning is done; user will write closeout during a focused block.",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("action: topic-await-reflection"));

    let project_state = read_toml(&task_dir.join(".daedalus/state.toml"));
    assert_eq!(project_active_topic(&project_state), "");
    assert_eq!(
        project_topic_lifecycle(&project_state, "main").as_deref(),
        Some("awaiting-reflection")
    );
    let topic_state = read_toml(&topic_dir.join(".daedalus/state.toml"));
    assert_eq!(topic_lifecycle(&topic_state), "awaiting-reflection");
    assert!(!repo.path().join("workspaces/current-project").exists());
    assert!(!repo.path().join("workspaces/current-topic").exists());
    assert!(!repo.path().join("workspaces/closeout-project").exists());
    assert!(repo.path().join("workspaces/closeout-topic").exists());
    let linked_projects = rust_analyzer_linked_projects(repo.path());
    assert!(linked_projects.contains(&"workspaces/closeout-topic/demo/Cargo.toml".to_owned()));
    assert!(
        linked_projects
            .contains(&"workspaces/projects/closeout-flow/topics/main/demo/Cargo.toml".to_owned())
    );
    let current = read_toml(&repo.path().join("workspaces/.daedalus/current.toml"));
    assert_eq!(toml_string(&current, "current_topic"), "");
    assert_eq!(
        toml_string(&current, "pending_closeout_topic"),
        "projects/closeout-flow/topics/main"
    );

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["validate", task_dir.to_str().expect("utf8")])
        .assert()
        .success();
}

#[test]
fn awaiting_reflection_topic_can_complete_reflection_stage_and_topic() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "closeout-complete",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/closeout-complete");
    let topic_dir = active_topic_dir(&task_dir);
    write_core_topic_artifacts(&topic_dir);
    write_demo_manifest(&topic_dir, "closeout-complete-demo");
    complete_core_topic_stages(&repo, &task_dir);

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "await-reflection",
            "main",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Core learning is done; user will finish closeout later.",
        ])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "10-reflection",
            "--topic-dir",
            topic_dir.to_str().expect("utf8"),
            "--reason",
            "Closeout reflection and knowledge archival are complete.",
        ])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "complete",
            "main",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Closeout reflection and knowledge archival are complete.",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("action: topic-complete"));

    let project_state = read_toml(&task_dir.join(".daedalus/state.toml"));
    assert_eq!(
        project_topic_lifecycle(&project_state, "main").as_deref(),
        Some("completed")
    );
    let topic_state = read_toml(&topic_dir.join(".daedalus/state.toml"));
    assert_eq!(topic_lifecycle(&topic_state), "completed");
    assert!(!repo.path().join("workspaces/closeout-topic").exists());
    let current = read_toml(&repo.path().join("workspaces/.daedalus/current.toml"));
    assert_eq!(toml_string(&current, "pending_closeout_topic"), "");
}

#[test]
fn awaiting_reflection_topic_does_not_block_next_active_topic() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "next-topic-after-closeout",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo
        .path()
        .join("workspaces/projects/next-topic-after-closeout");
    let topic_dir = active_topic_dir(&task_dir);
    write_core_topic_artifacts(&topic_dir);
    write_demo_manifest(&topic_dir, "main-demo");
    complete_core_topic_stages(&repo, &task_dir);
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "await-reflection",
            "main",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Core learning is done; closeout is scheduled.",
        ])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "new",
            "ddia",
            "--title",
            "DDIA",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .success();
    let ddia_dir = task_dir.join("topics/ddia");
    write_demo_manifest(&ddia_dir, "ddia-demo");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "activate",
            "ddia",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .success();

    let project_state = read_toml(&task_dir.join(".daedalus/state.toml"));
    assert_eq!(project_active_topic(&project_state), "ddia");
    assert_eq!(
        project_topic_lifecycle(&project_state, "main").as_deref(),
        Some("awaiting-reflection")
    );
    assert_eq!(
        project_topic_lifecycle(&project_state, "ddia").as_deref(),
        Some("active")
    );
    let current = read_toml(&repo.path().join("workspaces/.daedalus/current.toml"));
    assert_eq!(
        toml_string(&current, "current_topic"),
        "projects/next-topic-after-closeout/topics/ddia"
    );
    assert_eq!(
        toml_string(&current, "pending_closeout_topic"),
        "projects/next-topic-after-closeout/topics/main"
    );
    assert!(repo.path().join("workspaces/current-project").exists());
    assert!(repo.path().join("workspaces/current-topic").exists());
    assert!(!repo.path().join("workspaces/closeout-project").exists());
    assert!(repo.path().join("workspaces/closeout-topic").exists());
    let linked_projects = rust_analyzer_linked_projects(repo.path());
    assert!(linked_projects.contains(&"workspaces/current-topic/demo/Cargo.toml".to_owned()));
    assert!(linked_projects.contains(&"workspaces/closeout-topic/demo/Cargo.toml".to_owned()));

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["validate", task_dir.to_str().expect("utf8"), "--all-topics"])
        .assert()
        .success();
}

#[test]
fn validate_rejects_missing_closeout_projection() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "missing-closeout-projection",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo
        .path()
        .join("workspaces/projects/missing-closeout-projection");
    let topic_dir = active_topic_dir(&task_dir);
    write_core_topic_artifacts(&topic_dir);
    complete_core_topic_stages(&repo, &task_dir);
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "await-reflection",
            "main",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Core learning is done; closeout is scheduled.",
        ])
        .assert()
        .success();

    fs::remove_file(repo.path().join("workspaces/closeout-topic")).expect("remove symlink");

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["validate", task_dir.to_str().expect("utf8")])
        .assert()
        .failure();
}

#[test]
fn topic_await_reflection_rejects_second_pending_closeout() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "single-closeout-debt",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/single-closeout-debt");
    let main_dir = active_topic_dir(&task_dir);
    write_core_topic_artifacts(&main_dir);
    complete_core_topic_stages(&repo, &task_dir);
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "await-reflection",
            "main",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Core learning is done; closeout is scheduled.",
        ])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "new",
            "ddia",
            "--title",
            "DDIA",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .success();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "activate",
            "ddia",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
        ])
        .assert()
        .success();

    let ddia_dir = task_dir.join("topics/ddia");
    write_core_topic_artifacts(&ddia_dir);
    complete_core_topic_stages(&repo, &task_dir);

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "await-reflection",
            "ddia",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Try to create a second closeout debt.",
        ])
        .assert()
        .failure();
}

#[test]
fn topic_await_reflection_rejects_unfinished_core_stages() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "early-closeout",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/early-closeout");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "await-reflection",
            "main",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Try to await reflection before core learning is done.",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot await reflection"));
}

#[test]
fn topic_complete_rejects_unfinished_topic() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "init",
            "repo-learning",
            "unfinished-topic",
            "--topic",
            "main",
            "--title",
            "Main Topic",
        ])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/projects/unfinished-topic");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "topic",
            "complete",
            "main",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Try to complete before the topic stages are done.",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unfinished stages"));

    let project_state =
        fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("project state");
    assert!(project_state.contains("active_topic = \"main\""));
    assert!(project_state.contains("lifecycle = \"active\""));
}

#[test]
fn migrate_repo_learning_multi_topic_moves_legacy_workspace() {
    let repo = repo_fixture();
    let task_dir = repo.path().join("workspaces/projects/legacy-task");
    fs::create_dir_all(task_dir.join(".daedalus")).expect("daedalus dir");
    fs::create_dir_all(task_dir.join("notes/06-code-reader")).expect("notes dir");
    fs::create_dir_all(task_dir.join("guides/06-code-reader")).expect("guides dir");
    fs::create_dir_all(task_dir.join("demo/src")).expect("demo dir");
    fs::create_dir_all(task_dir.join("source")).expect("source dir");
    fs::write(task_dir.join(".daedalus/task-card.md"), "# 旧任务卡\n").expect("task-card");
    fs::write(task_dir.join(".daedalus/outcome-map.md"), "# Outcome Map\n").expect("outcome");
    fs::write(task_dir.join(".daedalus/todo.md"), "# Todo\n").expect("todo");
    fs::write(task_dir.join("notes/06-code-reader/README.md"), "# Notes\n").expect("notes");
    fs::write(
        task_dir.join("guides/06-code-reader/README.md"),
        "# Guides\n",
    )
    .expect("guides");
    fs::write(task_dir.join("demo/README.md"), "# Demo\n").expect("demo");
    fs::write(
        task_dir.join("source/pull_source.sh"),
        "REPO_URL=\"https://example.com/old.git\"\n",
    )
    .expect("pull source");
    fs::write(
        task_dir.join(".daedalus/state.toml"),
        r#"
[task]
name = "legacy-task"
kind = "repo-learning"
created_at = "2026-05-01 00:00:00"
lifecycle = "active"
workspace_bucket = "02-learning"
current_phase = "01-goal-aligner"
next_action = "继续旧任务。"

[[stages]]
id = "01-goal-aligner"
title = "对齐 Repo 学习目标"
status = "active"
required_artifacts = [".daedalus/task-card.md", ".daedalus/outcome-map.md"]

[[transitions]]
stage = "01-goal-aligner"
action = "init"
timestamp = "2026-05-01 00:00:00"
actor = "daedalus-cli"
reason = "初始化旧任务。"
"#,
    )
    .expect("state");

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "migrate",
            "repo-learning-multi-topic",
            task_dir.to_str().expect("utf8"),
            "--topic",
            "tools-permissions",
            "--title",
            "工具与权限",
            "--execute",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("ok: migration completed"));

    let topic_dir = task_dir.join("topics/tools-permissions");
    assert!(task_dir.join(".daedalus/project-map.md").exists());
    assert!(task_dir.join("shared/source-index.md").exists());
    assert!(topic_dir.join("notes/06-code-reader/README.md").exists());
    assert!(topic_dir.join("guides/06-code-reader/README.md").exists());
    assert!(topic_dir.join("demo/README.md").exists());
    assert!(topic_dir.join(".daedalus/task-card.md").exists());
    assert!(!task_dir.join("notes").exists());
    assert!(!task_dir.join("guides").exists());
    assert!(!task_dir.join("demo").exists());

    let project_state =
        fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("project state");
    assert!(project_state.contains("mode = \"multi-topic\""));
    assert!(project_state.contains("created_at = \"2026-05-01 00:00:00\""));
    assert!(project_state.contains("active_topic = \"tools-permissions\""));
    assert!(project_state.contains("action = \"migrate\""));
    assert!(project_state.contains("继续 active topic `tools-permissions`"));
    let topic_state =
        fs::read_to_string(topic_dir.join(".daedalus/state.toml")).expect("topic state");
    assert!(topic_state.contains("[topic]"));
    assert!(topic_state.contains("slug = \"tools-permissions\""));
    assert!(topic_state.contains("current_phase = \"01-goal-aligner\""));
    let pull_source = fs::read_to_string(task_dir.join("source/pull_source.sh")).expect("source");
    assert!(pull_source.contains("https://example.com/old.git"));
}
