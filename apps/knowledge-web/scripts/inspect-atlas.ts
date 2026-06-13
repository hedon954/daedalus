import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

type Check = {
  ok: boolean;
  message: string;
};

const root = resolve(import.meta.dirname, "../../..");
const app = resolve(root, "apps/knowledge-web/src/App.tsx");
const source = readFileSync(app, "utf8");

const sourcePathPattern = /knowledge-base\/[A-Za-z0-9_./-]+\.md/g;
const sourcePaths = [...new Set(source.match(sourcePathPattern) ?? [])];

const checks: Check[] = [
  {
    ok: !source.includes("renderMarkdown") && !source.includes("markdownToHtml"),
    message: "atlas must not render Markdown documents as HTML",
  },
  {
    ok: sourcePaths.length >= 4,
    message: "atlas should keep source links for traceability",
  },
  {
    ok: sourcePaths.every((path) => existsSync(resolve(root, path))),
    message: "all source links in atlas must point to existing knowledge-base files",
  },
  {
    ok: ["Agent 本地命令执行安全", "ReAct 工具运行时", "Sandbox 的第一性原理", "Rust 流式 Agent", "终端 Agent UI"].every(
      (label) => source.includes(label),
    ),
    message: "atlas should keep the current five learning modules",
  },
  {
    ok: ["onClick", "useState", "scenario-tabs", "diagram-picker", "MermaidDiagram"].every((token) =>
      source.includes(token),
    ),
    message: "atlas should remain interactive, not a static reading page",
  },
  {
    ok: ["codeAnchors", "failureModes", "checks", "mermaid.render"].every((token) => source.includes(token)),
    message: "atlas should include diagrams, code anchors, failure modes, and review checks",
  },
];

const failed = checks.filter((check) => !check.ok);

for (const check of checks) {
  console.log(`${check.ok ? "ok" : "fail"} - ${check.message}`);
}

if (failed.length > 0) {
  process.exitCode = 1;
}
