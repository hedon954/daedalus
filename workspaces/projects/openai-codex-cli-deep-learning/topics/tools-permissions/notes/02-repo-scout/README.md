# Repo 选择记录

## 最终选择

- Repo：`openai/codex`
- URL：`https://github.com/openai/codex.git`
- 本地源码目录：`source/codex`
- 固定版本：`ebe75bb683b3c237aad9f039ab17b187048aa499`
- 固定版本说明：按用户确认的“先用 main 拉取，再记录当前 commit”方式，从 `refs/heads/main` 解析得到。

## 选择理由

- 与学习目标直接匹配：目标就是深入学习 OpenAI Codex CLI 源码。
- 学习密度高：覆盖生产级 Agent CLI 的入口、会话、上下文、工具调用、命令执行和权限边界。
- 可形成 mini demo：可以抽取“CLI 入口 -> Agent loop -> 工具/沙箱 -> 结果回传”的最小闭环。

## 风险接受

- 运行可能需要网络、登录、模型服务或本机权限配置。
- 仓库变化快，因此后续笔记以固定 commit 为准。
- 如果真实模型调用不可用，优先使用测试、mock 或本地可验证路径完成 runbook。

## 源码准备证据

- 用户确认由 Agent 使用 `source/pull_source.sh` 拉取源码，以验证脚本真实可用。
- Agent 使用 `PINNED_REF=ebe75bb683b3c237aad9f039ab17b187048aa499 bash ./pull_source.sh` 拉取源码。
- 脚本输出 `ok: source pulled into codex at ebe75bb683b3c237aad9f039ab17b187048aa499`。
