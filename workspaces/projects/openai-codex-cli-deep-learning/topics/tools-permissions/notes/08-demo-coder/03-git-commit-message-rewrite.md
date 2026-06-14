# Git Commit Message Rewrite

本文记录一次小技能点：如何批量重写当前分支的历史提交信息，并验证只改了 commit message，没有改代码内容。

## Navigation

- Stage: `08-demo-coder`
- Context: 整理 `learning/codex` 分支历史，使提交信息统一符合 `type(scope): 中文描述` 规范
- Artifact advanced: repo 工程协作习惯、提交历史可读性
- Source of skill: 实际执行过的本地分支历史改写

## 这个操作解决什么问题

有时一个学习分支会积累很多提交，代码内容是对的，但 commit message 风格不统一：

```text
推进 Codex demo ReAct loop ...
实现复习计划与知识体系 CLI...
docs: 对齐 OpenAI-compatible ...
feat: 接入 OpenAI-compatible ...
```

如果这是只有自己使用的本地/个人分支，可以把历史 message 整理成统一格式，让后续 review、回看、归档更清晰。

这个操作的本质不是“修改旧 commit”，而是：

```text
读取旧 commit
  -> 保留同一份文件快照
  -> 替换 commit message
  -> 生成新的 commit
```

因为 commit hash 包含 message、parent、tree 等信息，所以 message 一变，该 commit hash 会变；后续 commit 的 parent hash 也会跟着变。

## 适用边界

适合：

- 个人分支。
- 尚未被多人基于旧 hash 协作的分支。
- 只想整理 message，不想改代码内容。
- 有明确的旧 message 到新 message 的映射关系。

谨慎：

- 已经被别人拉取并基于其开发的共享分支。
- 发布 tag 已经指向旧 commit 的历史。
- 不确定是否只改 message，无法验证 tree 是否一致。

## 脚本结构

可以写一个临时脚本作为 `git filter-branch --msg-filter`：

```bash
#!/usr/bin/env bash
set -euo pipefail

msg="$(cat)"
subject="${msg%%$'\n'*}"
body=""

if [[ "$msg" == *$'\n'* ]]; then
  body="${msg#*$'\n'}"
fi

new_subject="$subject"

case "$subject" in
  "推进 Codex demo ReAct loop"* )
    new_subject="feat(demo): 推进 Codex demo ReAct loop 与 IDE 同步"
    ;;

  "实现复习计划与知识体系 CLI"* )
    new_subject="feat(cli): 实现复习计划与知识体系 CLI"
    ;;

  "实现复习与知识萃取第一性原理门槛"* )
    new_subject="feat(learning): 实现复习与知识萃取第一性原理门槛"
    ;;

  "规划复习与知识体系萃取能力"* )
    new_subject="docs(plan): 规划复习与知识体系萃取能力"
    ;;

  "重构 repo learning 为多专题项目模型"* )
    new_subject="feat(repo-learning): 重构为多专题项目模型"
    ;;

  "docs: 对齐 OpenAI-compatible 流式接入指南"* )
    new_subject="docs(demo): 对齐 OpenAI-compatible 流式接入指南"
    ;;

  "feat: 接入 OpenAI-compatible 流式 LLM 雏形"* )
    new_subject="feat(demo): 接入 OpenAI-compatible 流式 LLM 雏形"
    ;;

  "feat: 以终点产物重构 repo learning 流程"* )
    new_subject="feat(repo-learning): 以终点产物重构学习流程"
    ;;

  "Merge branch 'main' into learning/codex" )
    new_subject="chore(git): 合并 main 到 learning/codex"
    ;;
esac

if [[ -n "$body" && "$body" != "$msg" ]]; then
  printf '%s\n%s\n' "$new_subject" "$body"
else
  printf '%s\n' "$new_subject"
fi
```

关键点：

- `msg="$(cat)"`：从 stdin 读取当前 commit 的旧 message。
- `subject="${msg%%$'\n'*}"`：取第一行作为标题。
- `case "$subject"`：按旧标题匹配并生成新标题。
- `printf`：把新 message 输出到 stdout，Git 会把 stdout 当作新 commit message。
- body 默认保留，避免误删原本有价值的说明。

## 执行步骤

先留备份分支：

```bash
git branch backup/learning-codex-before-message-rewrite
```

执行 message filter：

```bash
chmod +x /tmp/rewrite-daedalus-message.sh

git filter-branch -f \
  --msg-filter /tmp/rewrite-daedalus-message.sh \
  origin/main..HEAD
```

验证代码树没有变：

```bash
git diff --quiet backup/learning-codex-before-message-rewrite HEAD
```

如果命令返回 0，说明新旧 HEAD 的文件内容一致。

查看新的提交图：

```bash
git log --oneline --decorate --graph --boundary origin/main..HEAD
```

确认没问题后清理备份引用：

```bash
git branch -D backup/learning-codex-before-message-rewrite
git update-ref -d refs/original/refs/heads/learning/codex
```

## 验收标准

完成后至少检查三件事：

```bash
git status --short --branch
git log --oneline --decorate --graph --boundary origin/main..HEAD
git diff --quiet <old-head-or-backup> HEAD
```

期望结果：

- 工作区干净。
- 提交信息统一。
- 新旧 HEAD 的代码树一致。
- 如果远端分支已经存在，新旧 commit hash 会不同，需要明确知道是否要 force push。

## 重要风险

历史重写会改变 commit hash。多人协作场景下，如果别人本地还有旧 hash，再同步时会出现分叉。

如果确认要更新远端个人分支，应该使用：

```bash
git push --force-with-lease origin learning/codex
```

`--force-with-lease` 比 `--force` 稍安全：它会确认远端仍是自己以为的旧状态，避免覆盖别人刚推上去的新提交。

## 一个实际教训

写 commit message 或脚本参数时，如果内容里有反引号，避免放在双引号里：

```bash
git commit -m "docs: 记录 `make build` 输出"
```

在 shell 中，双引号里的反引号可能触发命令替换，导致意外执行 `make build`。更稳妥的写法是：

```bash
git commit -m 'docs: 记录 `make build` 输出'
```

这类细节看起来小，但在自动化 git 操作里很容易制造非预期副作用。
