use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

fn repo_fixture() -> TempDir {
    let temp = TempDir::new().expect("temp dir");
    let repo = temp.path();
    fs::create_dir_all(repo.join("system/templates")).expect("templates dir");
    fs::create_dir_all(repo.join("workspaces/02-learning")).expect("workspaces dir");
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
        .stdout(predicates::str::contains(
            "workspaces/02-learning/from-subdir",
        ));
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

    let task_dir = repo.path().join("workspaces/02-learning/my-learning-task");
    let topic_dir = active_topic_dir(&task_dir);
    assert!(task_dir.join("CLAUDE.md").exists());
    assert!(!task_dir.join(".daedalus/CLAUDE.md").exists());
    assert!(task_dir.join("shared/README.md").exists());
    assert!(task_dir.join("shared/source-index.md").exists());
    assert!(task_dir.join("topics/.gitkeep").exists());
    assert!(topic_dir.join("demo/.gitkeep").exists());
    assert!(topic_dir.join("guides/.gitkeep").exists());
    assert!(topic_dir.join("notes/.gitkeep").exists());
    assert!(task_dir.join("source/.gitignore").exists());
    assert!(task_dir.join("source/pull_source.sh").exists());
    assert!(task_dir.join(".daedalus/state.toml").exists());
    assert!(task_dir.join(".daedalus/project-map.md").exists());
    assert!(task_dir.join(".daedalus/topic-board.md").exists());
    assert!(topic_dir.join(".daedalus/outcome-map.md").exists());
    assert!(topic_dir.join(".daedalus/validation-log.md").exists());
    let claude = fs::read_to_string(task_dir.join("CLAUDE.md")).expect("CLAUDE.md");
    assert!(claude.contains("@.daedalus/state.md"));
    assert!(claude.contains("@.daedalus/project-map.md"));
    assert!(claude.contains("@topics/main/.daedalus/outcome-map.md"));
    assert!(claude.contains("Project root 的 `.daedalus/state.toml`"));
    assert!(claude.contains("Topic 的 `topics/<slug>/.daedalus/state.toml`"));
    assert!(claude.contains("`daedalus validate` 校验 project + active topic"));
    let state_md = fs::read_to_string(task_dir.join(".daedalus/state.md")).expect("state.md");
    assert!(state_md.contains("# Project 状态"));
    assert!(state_md.contains("[`.daedalus/state.toml`](state.toml)"));
    assert!(state_md.contains("Active Topic：`main`"));
    assert!(state_md.contains("生命周期：`active`"));
    assert!(state_md.contains("Workspace Bucket：`02-learning`"));
    assert!(state_md.contains("topics/main"));
    let state_toml = fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("state.toml");
    assert!(state_toml.contains("Project state 只描述"));
    assert!(state_toml.contains("task.lifecycle 只能是"));
    assert!(state_toml.contains("lifecycle = \"active\""));
    assert!(state_toml.contains("workspace_bucket = \"02-learning\""));
    assert!(state_toml.contains("active_topic = \"main\""));
    assert!(state_toml.contains("transition.action 只能是"));
    let task_card =
        fs::read_to_string(topic_dir.join(".daedalus/task-card.md")).expect("task-card.md");
    assert!(task_card.contains("# 专题学习任务卡"));
    let outcome_map =
        fs::read_to_string(topic_dir.join(".daedalus/outcome-map.md")).expect("outcome-map.md");
    assert!(outcome_map.contains("# Outcome Map"));
    assert!(outcome_map.contains("## North Star"));
    assert!(outcome_map.contains("## Stop Rules"));
    let artifact_index =
        fs::read_to_string(topic_dir.join(".daedalus/artifact-index.md")).expect("artifact-index");
    assert!(artifact_index.contains("> `状态` 列只能使用"));
    assert!(artifact_index.contains("[`.daedalus/task-card.md`](task-card.md)"));
    assert!(artifact_index.contains("[`.daedalus/outcome-map.md`](outcome-map.md)"));
    assert!(artifact_index.contains("[`guides/`](../guides)"));
    assert!(
        artifact_index
            .contains("[`guides/02-repo-scout/README.md`](../guides/02-repo-scout/README.md)")
    );
    assert!(artifact_index.contains("`草稿`"));
    assert!(artifact_index.contains("`不适用`"));
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

    let task_dir = repo.path().join("workspaces/02-learning/comments");
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

    let task_dir = repo.path().join("workspaces/02-learning/missing-artifact");
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

    let task_dir = repo.path().join("workspaces/02-learning/pending-complete");
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

    let task_dir = repo
        .path()
        .join("workspaces/02-learning/resume-single-active");
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

    let task_dir = repo.path().join("workspaces/02-learning/rollback-stage");
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

    let task_dir = repo.path().join("workspaces/02-learning/rollback-pending");
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

    let task_dir = repo.path().join("workspaces/02-learning/next-action");
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

    let task_dir = repo.path().join("workspaces/02-learning/force");
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

    let task_dir = repo
        .path()
        .join("workspaces/02-learning/final-stage-pending");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "complete",
            "10-archivist",
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

    let task_dir = repo.path().join("workspaces/02-learning/final-stage");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "10-archivist",
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
            "10-archivist",
            "--project-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Archive artifacts are ready.",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("ok: state transition completed"))
        .stdout(predicates::str::contains("stage: 10-archivist"))
        .stdout(predicates::str::contains("topic_dir:"));

    assert!(task_dir.exists());
    assert!(
        !repo
            .path()
            .join("workspaces/03-completed/final-stage")
            .exists()
    );
    let topic_state = fs::read_to_string(active_topic_dir(&task_dir).join(".daedalus/state.toml"))
        .expect("topic state");
    assert!(topic_state.contains("current_phase = \"10-archivist\""));
    assert!(
        topic_state.contains("id = \"10-archivist\"\ntitle = \"闭环专题学习\"\nstatus = \"done\"")
    );
}

#[test]
fn task_complete_moves_task_and_releases_wip() {
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

    let task_dir = repo.path().join("workspaces/02-learning/close-me");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args([
            "state",
            "enter",
            "10-archivist",
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
        .stdout(predicates::str::contains("lifecycle: completed"))
        .stdout(predicates::str::contains(
            "workspaces/03-completed/close-me",
        ));

    assert!(!task_dir.exists());
    assert!(
        repo.path()
            .join("workspaces/03-completed/close-me")
            .exists()
    );
    let moved_state = fs::read_to_string(
        repo.path()
            .join("workspaces/03-completed/close-me/.daedalus/state.toml"),
    )
    .expect("moved state");
    assert!(moved_state.contains("lifecycle = \"completed\""));
    assert!(moved_state.contains("workspace_bucket = \"03-completed\""));
    assert!(moved_state.contains("active_topic = \"\""));
    assert!(moved_state.contains("slug = \"main\""));
    assert!(moved_state.contains("lifecycle = \"abandoned\""));
    let decision_log = fs::read_to_string(
        repo.path()
            .join("workspaces/03-completed/close-me/.daedalus/decision-log.md"),
    )
    .expect("decision log");
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
fn task_abandon_moves_task_and_records_reason() {
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

    let task_dir = repo.path().join("workspaces/02-learning/abandon-me");
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
        .stdout(predicates::str::contains(
            "workspaces/04-abandoned/abandon-me",
        ));

    let moved_state = fs::read_to_string(
        repo.path()
            .join("workspaces/04-abandoned/abandon-me/.daedalus/state.toml"),
    )
    .expect("moved state");
    assert!(moved_state.contains("action = \"abandon\""));
    assert!(moved_state.contains("lifecycle = \"abandoned\""));
    assert!(moved_state.contains("workspace_bucket = \"04-abandoned\""));
    assert!(moved_state.contains("closed_at = "));
    assert!(moved_state.contains("close_reason = "));
    assert!(moved_state.contains("Dry run scope changed"));
    assert!(!moved_state.contains("status = \"active\""));
    let decision_log = fs::read_to_string(
        repo.path()
            .join("workspaces/04-abandoned/abandon-me/.daedalus/decision-log.md"),
    )
    .expect("decision log");
    assert!(decision_log.contains("决策：放弃学习任务"));
    assert!(decision_log.contains("Dry run scope changed"));
}

#[test]
fn task_move_refuses_to_overwrite_destination() {
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

    fs::create_dir_all(repo.path().join("workspaces/03-completed/collision"))
        .expect("completed collision dir");
    let task_dir = repo.path().join("workspaces/02-learning/collision");
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
        .stderr(predicates::str::contains(
            "task move destination already exists",
        ));

    assert!(task_dir.exists());
    let state = fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("state");
    assert!(state.contains("lifecycle = \"active\""));
    assert!(state.contains("workspace_bucket = \"02-learning\""));
    assert!(!state.contains("lifecycle = \"completed\""));
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

    let task_dir = repo
        .path()
        .join("workspaces/02-learning/lifecycle-mismatch");
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
        .stderr(predicates::str::contains("task lifecycle `completed`"));
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

    let task_dir = repo.path().join("workspaces/02-learning/validate");
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

    let task_dir = repo.path().join("workspaces/02-learning/topic-flow");
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

    let task_dir = repo.path().join("workspaces/02-learning/unfinished-topic");
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
    let task_dir = repo.path().join("workspaces/02-learning/legacy-task");
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
