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
    assert!(task_dir.join(".daedalus/state.toml").exists());
    let state_md = fs::read_to_string(task_dir.join(".daedalus/state.md")).expect("state.md");
    assert!(state_md.contains("# 学习状态"));
    assert!(state_md.contains("[`.daedalus/state.toml`](state.toml)"));
    assert!(state_md.contains("当前阶段：`01-goal-aligner`"));
    assert!(state_md.contains("[`notes/repo-selection.md`](../notes/repo-selection.md)"));
    assert!(state_md.contains("`stage.status` 只能是"));
    assert!(state_md.contains("`transition.approval_source` 只能是"));
    let state_toml = fs::read_to_string(task_dir.join(".daedalus/state.toml")).expect("state.toml");
    assert!(state_toml.contains("stage.status 只能是"));
    assert!(state_toml.contains("transition.action 只能是"));
    let task_card =
        fs::read_to_string(task_dir.join(".daedalus/task-card.md")).expect("task-card.md");
    assert!(task_card.contains("# 学习任务卡"));
    let artifact_index =
        fs::read_to_string(task_dir.join(".daedalus/artifact-index.md")).expect("artifact-index");
    assert!(artifact_index.contains("> `状态` 列只能使用"));
    assert!(artifact_index.contains("[`.daedalus/task-card.md`](task-card.md)"));
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
