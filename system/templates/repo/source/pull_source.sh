#!/usr/bin/env bash
set -euo pipefail

# 用法：
#   REPO_URL="https://github.com/example/repo.git" \
#   REPO_DIR="repo" \
#   PINNED_REF="main" \
#   ./pull_source.sh
#
# 说明：
# - 该脚本只负责可复现地拉取学习原材料。
# - 外部源码默认被 source/.gitignore 忽略，不应提交到 daedalus 仓库。
# - 拉取后会删除外部仓库自带的 .git 目录，避免嵌套 git 仓库污染当前项目。

REPO_URL="${REPO_URL:-}"
REPO_DIR="${REPO_DIR:-}"
PINNED_REF="${PINNED_REF:-}"

if [[ -z "$REPO_URL" || -z "$REPO_DIR" || -z "$PINNED_REF" ]]; then
  echo "error: REPO_URL, REPO_DIR and PINNED_REF are required" >&2
  exit 1
fi

if [[ -e "$REPO_DIR" ]]; then
  echo "skip: $REPO_DIR already exists"
  exit 0
fi

git clone --depth 1 "$REPO_URL" "$REPO_DIR"
git -C "$REPO_DIR" fetch --depth 1 origin "$PINNED_REF" || true
git -C "$REPO_DIR" checkout "$PINNED_REF"
rm -rf "$REPO_DIR/.git"

echo "ok: source pulled into $REPO_DIR at $PINNED_REF"
