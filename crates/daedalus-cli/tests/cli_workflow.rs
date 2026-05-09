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

#[test]
fn command_rejects_non_daedalus_directory() {
    let temp = TempDir::new().expect("temp dir");
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(temp.path())
        .args(["init", "repo-learning", "illegal"])
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
        .args(["init", "repo-learning", "from-subdir"])
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
        .args(["init", "repo-learning", "My Learning Task"])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/02-learning/my-learning-task");
    assert!(task_dir.join("CLAUDE.md").exists());
    assert!(!task_dir.join(".daedalus/CLAUDE.md").exists());
    assert!(task_dir.join("demo/.gitkeep").exists());
    assert!(task_dir.join("guides/.gitkeep").exists());
    assert!(task_dir.join("notes/.gitkeep").exists());
    assert!(task_dir.join("source/.gitignore").exists());
    assert!(task_dir.join("source/pull_source.sh").exists());
    assert!(task_dir.join(".daedalus/state.toml").exists());
    assert!(task_dir.join(".daedalus/validation-log.md").exists());
    let claude = fs::read_to_string(task_dir.join("CLAUDE.md")).expect("CLAUDE.md");
    assert!(claude.contains("@.daedalus/state.md"));
    assert!(claude.contains("[`.daedalus/state.toml`](.daedalus/state.toml)"));
    assert!(claude.contains("`guides/` 用于保存 Agent 生成的行动指南"));
    let state_md = fs::read_to_string(task_dir.join(".daedalus/state.md")).expect("state.md");
    assert!(state_md.contains("# 学习状态"));
    assert!(state_md.contains("[`.daedalus/state.toml`](state.toml)"));
    assert!(state_md.contains("当前阶段：`01-goal-aligner`"));
    assert!(state_md.contains("生命周期：`active`"));
    assert!(state_md.contains("Workspace Bucket：`02-learning`"));
    assert!(
        state_md.contains(
            "[`guides/02-repo-selection-guide.md`](../guides/02-repo-selection-guide.md)"
        )
    );
    assert!(state_md.contains("`stage.status` 只能是"));
    assert!(state_md.contains("`transition.approval_source` 只能是"));
    let state_toml = fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("state.toml");
    assert!(state_toml.contains("stage.status 只能是"));
    assert!(state_toml.contains("task.lifecycle 只能是"));
    assert!(state_toml.contains("lifecycle = \"active\""));
    assert!(state_toml.contains("workspace_bucket = \"02-learning\""));
    assert!(state_toml.contains("transition.action 只能是"));
    let task_card =
        fs::read_to_string(task_dir.join(".daedalus/task-card.md")).expect("task-card.md");
    assert!(task_card.contains("# 学习任务卡"));
    assert!(task_card.contains("## 角色边界"));
    let artifact_index =
        fs::read_to_string(task_dir.join(".daedalus/artifact-index.md")).expect("artifact-index");
    assert!(artifact_index.contains("> `状态` 列只能使用"));
    assert!(artifact_index.contains("[`.daedalus/task-card.md`](task-card.md)"));
    assert!(artifact_index.contains("[`guides/`](../guides)"));
    assert!(
        artifact_index.contains(
            "[`guides/02-repo-selection-guide.md`](../guides/02-repo-selection-guide.md)"
        )
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
        .args(["init", "repo-learning", "first"])
        .assert()
        .success();

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["init", "repo-learning", "second"])
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
        .args(["init", "repo-learning", "comments"])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/02-learning/comments");
    let state_path = task_dir.join(".daedalus/state.toml");
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
            "--task-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Move to repo selection after goal alignment.",
        ])
        .assert()
        .success();

    let updated = fs::read_to_string(state_path).expect("updated state");
    assert!(updated.contains("# custom operator note"));
    assert!(updated.contains("action = \"enter\""));
}

#[test]
fn complete_rejects_missing_artifact_without_force() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["init", "repo-learning", "missing-artifact"])
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
            "--task-dir",
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
            "--task-dir",
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
        .args(["init", "repo-learning", "pending-complete"])
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
            "--task-dir",
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
        .args(["init", "repo-learning", "resume-single-active"])
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
            "--task-dir",
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
            "--task-dir",
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
            "--task-dir",
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
            "--task-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Source access is restored.",
        ])
        .assert()
        .success();

    let state = fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("state");
    assert_eq!(state.matches("status = \"active\"").count(), 1);
    assert!(
        state.contains("id = \"02-repo-scout\"\ntitle = \"选择学习仓库\"\nstatus = \"active\"")
    );
    assert!(state.contains(
        "id = \"03-socratic-coach\"\ntitle = \"提出 Repo 递进问题\"\nstatus = \"blocked\""
    ));
}

#[test]
fn complete_updates_next_action_to_next_stage() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["init", "repo-learning", "next-action"])
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
            "--task-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Goal alignment artifacts are ready.",
        ])
        .assert()
        .success();

    let state = fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("state");
    assert!(state.contains("next_action = \"进入 `02-repo-scout`"));
    let state_md = fs::read_to_string(task_dir.join(".daedalus/state.md")).expect("state.md");
    assert!(state_md.contains("下一步：进入 `02-repo-scout`"));
}

#[test]
fn force_requires_reason_and_approval_source() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["init", "repo-learning", "force"])
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
            "--task-dir",
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
            "--task-dir",
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
            "--task-dir",
            task_dir.to_str().expect("utf8"),
            "--force",
            "--reason",
            "Repo selection was documented in .daedalus/decision-log.md.",
            "--approval-source",
            "artifact-equivalent",
        ])
        .assert()
        .success();

    let state = fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("state");
    assert!(state.contains("approval_source = \"artifact-equivalent\""));
}

#[test]
fn final_stage_completion_requires_active_final_stage() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["init", "repo-learning", "final-stage-pending"])
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
            "--task-dir",
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
fn final_stage_completion_moves_task_to_completed() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["init", "repo-learning", "final-stage"])
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
            "--task-dir",
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
            "--task-dir",
            task_dir.to_str().expect("utf8"),
            "--reason",
            "Archive artifacts are ready.",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("ok: task completed"))
        .stdout(predicates::str::contains("lifecycle: completed"))
        .stdout(predicates::str::contains("moved: true"))
        .stdout(predicates::str::contains("from_task_dir:"))
        .stdout(predicates::str::contains(
            "workspaces/03-completed/final-stage",
        ));

    assert!(!task_dir.exists());
    assert!(
        repo.path()
            .join("workspaces/03-completed/final-stage/.daedalus/state.md")
            .exists()
    );
    let moved_state = fs::read_to_string(
        repo.path()
            .join("workspaces/03-completed/final-stage/.daedalus/state.toml"),
    )
    .expect("moved state");
    assert!(moved_state.contains("lifecycle = \"completed\""));
    assert!(moved_state.contains("workspace_bucket = \"03-completed\""));
    assert!(moved_state.contains("closed_at = "));
    assert!(moved_state.contains("close_reason = \"Archive artifacts are ready.\""));
    assert!(moved_state.contains("current_phase = \"10-archivist\""));
    assert!(!moved_state.contains("status = \"active\""));
    let decision_log = fs::read_to_string(
        repo.path()
            .join("workspaces/03-completed/final-stage/.daedalus/decision-log.md"),
    )
    .expect("decision log");
    assert!(decision_log.contains("决策：完成学习任务"));
    assert!(decision_log.contains("Archive artifacts are ready."));
}

#[test]
fn task_complete_moves_task_and_releases_wip() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["init", "repo-learning", "close-me"])
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
            "--task-dir",
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
    assert!(moved_state.contains("current_phase = \"10-archivist\""));
    assert!(!moved_state.contains("status = \"active\""));
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
        .args(["init", "repo-learning", "new-task"])
        .assert()
        .success();
}

#[test]
fn task_abandon_moves_task_and_records_reason() {
    let repo = repo_fixture();
    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["init", "repo-learning", "abandon-me"])
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
        .args(["init", "repo-learning", "collision"])
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
        .args(["init", "repo-learning", "lifecycle-mismatch"])
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
        .args(["init", "repo-learning", "validate"])
        .assert()
        .success();

    let task_dir = repo.path().join("workspaces/02-learning/validate");
    fs::remove_file(task_dir.join(".daedalus/todo.md")).expect("remove todo");

    Command::cargo_bin("daedalus")
        .expect("binary")
        .current_dir(repo.path())
        .args(["validate", task_dir.to_str().expect("utf8")])
        .assert()
        .failure()
        .stderr(predicates::str::contains("workspace validation failed"));
}
