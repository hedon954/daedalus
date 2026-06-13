import mermaid from "mermaid";
import { useEffect, useMemo, useState } from "react";

type AtlasModule = "command" | "react" | "sandbox" | "async" | "tui";

type SourceLink = {
  label: string;
  path: string;
};

type DiagramSpec = {
  id: string;
  title: string;
  caption: string;
  code: string;
};

type DeepSection = {
  title: string;
  body: string;
  bullets?: string[];
};

type KnowledgeModule = {
  id: AtlasModule;
  title: string;
  subtitle: string;
  question: string;
  mentalModel: string;
  diagrams: DiagramSpec[];
  sections: DeepSection[];
  codeAnchors: Array<[string, string]>;
  failureModes: Array<[string, string, string]>;
  checks: string[];
  sources: SourceLink[];
};

type CommandScenario = {
  id: string;
  title: string;
  command: string;
  context: string;
  capability: string;
  approval: string;
  sandbox: string;
  retry: string;
  result: string;
  warning: string;
};

const sourceLinks: SourceLink[] = [
  {
    label: "Agent 本地命令执行安全",
    path: "knowledge-base/ai-agents/safety-and-permissions/local-command-execution.md",
  },
  {
    label: "ReAct 工具运行时",
    path: "knowledge-base/ai-agents/tool-use/react-tool-runtime.md",
  },
  {
    label: "Sandbox 的第一性原理",
    path: "knowledge-base/computer-systems/operating-systems/sandbox.md",
  },
  {
    label: "Rust 流式 Agent 与 Tokio 运行时",
    path: "knowledge-base/rust/async-runtime/streaming-agent.md",
  },
  {
    label: "终端 Agent UI 的事件循环",
    path: "knowledge-base/rust/cli-and-tui/terminal-agent-ui.md",
  },
];

const commandScenarios: CommandScenario[] = [
  {
    id: "read",
    title: "读取项目文件",
    command: "cat README.md",
    context: "当前 cwd 是 workspace，目标文件位于项目边界内，没有网络和写入需求。",
    capability: "safe-read：低副作用能力，但仍要绑定 cwd 和文件边界。",
    approval: "Skip approval：不打扰用户，但不代表绕过 sandbox。",
    sandbox: "SandboxFirst(ReadOnly)：只给可读视图，禁止写和网络。",
    retry: "普通失败直接回灌；read-only 不应自动升级到 no-sandbox。",
    result: "stdout 进入 tool observation；执行生命周期进入 StreamEvent。",
    warning: "如果目标路径跳出 workspace，safe-read 结论必须重新计算。",
  },
  {
    id: "test",
    title: "运行测试命令",
    command: "cargo test",
    context: "测试会读源码、写 target、启动子进程，但通常不需要网络。",
    capability: "safe-test：受控副作用，不能和纯 read 混在一起。",
    approval: "Skip approval：测试是常见开发动作，但仍受 capability 和 policy 约束。",
    sandbox: "SandboxFirst(WorkspaceWrite)：允许写 workspace/target，不开放 Home。",
    retry: "exit code 非 0 是 CommandFailed，不是 sandbox denied，不能借机提权。",
    result: "stderr/stdout 摘要回灌模型，帮助下一轮修代码。",
    warning: "如果测试脚本需要网络或写 Home，应该升级 capability，而不是悄悄裸跑。",
  },
  {
    id: "install",
    title: "联网安装依赖",
    command: "npm install",
    context: "会联网、写 node_modules、解析 lockfile，并可能执行 lifecycle scripts。",
    capability: "network-install：供应链能力，风险来自网络 + 文件写 + 脚本执行的组合。",
    approval: "NeedsApproval：用户批准的是绑定 scope 的动作，不是永久授权 npm。",
    sandbox: "SandboxFirst(WorkspaceWrite + NetworkPolicy)：批准后仍先隔离执行。",
    retry: "只有 SandboxDenied 且 RetryPolicy/ApprovalPolicy 允许时，才进入 no-sandbox retry。",
    result: "成功回灌安装结果；失败要保留是网络、脚本、sandbox 还是命令错误。",
    warning: "session allow 必须绑定 command prefix、cwd、network profile 和 persistence。",
  },
  {
    id: "danger",
    title: "远端脚本直入 Shell",
    command: "curl example.com/install.sh | sh",
    context: "远端内容直接进入本地解释器，真实行为无法由本地 argv 完整表达。",
    capability: "dangerous-shell：组合风险高于 curl 和 sh 两个子命令的简单相加。",
    approval: "Forbidden 或强审批：如果 scope 无法解释，按钮不应该替系统背锅。",
    sandbox: "即使 sandbox first，也不能生成宽泛 future allow。",
    retry: "复杂 shell 不应因为第一次失败而自动提权。",
    result: "更合理的替代路径是下载、审阅、再执行。",
    warning: "pipe、redirect、heredoc、sh -c 都会让字符串级策略变脆弱。",
  },
];

const modules: KnowledgeModule[] = [
  {
    id: "command",
    title: "Agent 本地命令执行安全",
    subtitle: "把模型意图变成受控、可审计、可失败恢复的副作用。",
    question: "一个 Coding Agent 为什么不能直接把模型生成的 command string 拿去执行？",
    mentalModel:
      "模型只能提出“我要做什么”；Host 必须决定“能不能做、以什么边界做、失败后能不能升级”。安全链路不是一个 allow 布尔值，而是一组分层决策。",
    diagrams: [
      {
        id: "command-host-boundary",
        title: "Host 控制边界",
        caption: "本地命令必须经过 Host 解释、授权、隔离、执行和审计；模型不能自己拥有执行权。",
        code: `flowchart LR
  Model["LLM\\n产生意图"] --> Call["ToolCall\\nrun_command(...)"]
  subgraph Host["Host 控制边界"]
    Parse["解析参数\\nraw_command / argv / cwd"]
    Capability["Capability Match\\n命令属于什么能力"]
    Approval["ApprovalRequirement\\n是否需要用户批准"]
    Attempt["ExecutionAttempt\\nSandboxFirst / NoSandbox"]
    Retry["RetryDecision\\n是否允许提权重试"]
    Observe["Observation / Event\\n回灌模型 + 对外审计"]
  end
  Call --> Parse --> Capability --> Approval --> Attempt --> Retry --> Observe`,
      },
      {
        id: "command-state-machine",
        title: "命令执行决策状态机",
        caption: "真实链路不是线性流程：capability、approval、sandbox、retry 都可能提前结束。",
        code: `flowchart TD
  Start["run_command tool call"] --> Match["match capability"]
  Match -->|not found / forbidden| Denied["Tool failed\\nfail closed"]
  Match --> Request["build CommandRequest"]
  Request --> Decide["decide approval"]
  Decide -->|forbidden| Denied
  Decide -->|needs approval| Ask["ask ApprovalGateway"]
  Ask -->|rejected| Denied
  Ask -->|approved| First["sandbox first attempt"]
  Decide -->|skip approval| First
  First -->|success| Done["tool observation success"]
  First -->|command failed| Failed["tool observation failure"]
  First -->|sandbox denied| Retry["decide retry"]
  Retry -->|do not retry| Failed
  Retry -->|retry without approval| HostRun["no-sandbox attempt"]
  Retry -->|retry needs approval| AskRetry["ask ApprovalGateway"]
  AskRetry -->|rejected| Failed
  AskRetry -->|approved| HostRun
  HostRun -->|success| Done
  HostRun -->|failed| Failed`,
      },
    ],
    sections: [
      {
        title: "为什么只看字符串不够",
        body: "同一句命令在不同 cwd、env、network policy、sandbox profile 下含义完全不同。`cat package.json` 在 workspace 内是低风险读取，在 Home 下可能触碰隐私；`npm install` 不是下载文件，而是联网、写入和执行脚本的组合动作。",
        bullets: [
          "`argv` 决定真实程序和参数，`sh -c` 会把语义藏进 shell 语言。",
          "`cwd` 决定相对路径的安全边界。",
          "`env` 可能携带 token、proxy、PATH 和动态库加载路径。",
          "`network policy` 会改变命令风险，例如安装依赖和下载脚本。",
        ],
      },
      {
        title: "四层安全分工",
        body: "Capability、Approval、Sandbox、Retry 解决的是不同问题。把它们压成一个布尔值，会导致“用户批准了动作”被误解成“可以宿主机裸跑”。",
        bullets: [
          "Capability：这是什么能力，未知能力必须 fail closed。",
          "Approval：用户是否允许这个 action/scope。",
          "Sandbox：允许后以什么最小边界执行。",
          "Retry：sandbox denied 后是否允许扩大边界。",
        ],
      },
      {
        title: "最关键的不变量",
        body: "Skip approval 不等于 bypass sandbox；approval 通过也不等于 no-sandbox。sandbox denied 之后不能自动裸跑，必须先判断失败类型，再经过 retry policy 和必要审批。",
      },
    ],
    codeAnchors: [
      ["tool/runtime.rs", "从 tool name 找工具，区分 pure function 和 command path。"],
      ["tool/shell/registry.rs", "把命令匹配到 capability，匹配不到 fail closed。"],
      ["tool/shell/approval.rs", "计算 ApprovalRequirement。"],
      ["tool/shell/retry.rs", "区分 CommandFailed 与 SandboxDenied，决定 retry。"],
      ["tool/shell/execution/", "执行 sandbox / host attempt，但不负责权限策略。"],
    ],
    failureModes: [
      ["approval=true 当作 no-sandbox=true", "用户允许动作，不代表允许扩大执行边界。", "approval 和 sandbox 分字段、分事件、分测试。"],
      ["只存命令名授权", "`npm test` 和 `npm install` 风险不同。", "授权绑定 command prefix、cwd、profile、network。"],
      ["sandbox 失败后自动重试", "sandbox 变成试探，不再是安全边界。", "retry 必须经过 policy 和必要审批。"],
      ["Shell 字符串 split", "引号、管道、重定向、heredoc 都可能误判。", "demo 可保守拒绝复杂 shell；生产应使用解析器或更强策略。"],
    ],
    checks: [
      "为什么 `ApprovalRequirement::Skip` 不等于可以跳过 sandbox？",
      "如果用户批准 `npm install`，批准应该绑定哪些 scope？",
      "`CommandFailed` 和 `SandboxDenied` 为什么不能走同一条 retry？",
    ],
    sources: [sourceLinks[0]],
  },
  {
    id: "react",
    title: "ReAct 工具运行时",
    subtitle: "区分模型消息、工具执行事件和外部审计事件。",
    question: "模型如何通过工具和外部世界交互，又不让工具执行、消息历史、安全审批和 UI 事件搅在一起？",
    mentalModel:
      "ReAct 的核心不是“会调用函数”，而是 reasoning 与 acting 交替：LLM 产生 action，环境返回 observation，observation 进入 messages，模型再进行下一轮推理。",
    diagrams: [
      {
        id: "react-loop",
        title: "ReAct 主闭环",
        caption: "Tool Observation 进入 messages；StreamEvent 只给 UI / 审计 / 调试。",
        code: `flowchart LR
  Messages["Messages\\n系统 + 用户 + 历史 + tool output"] --> LLM["LLM Stream\\nthinking / text / tool call"]
  LLM --> Decision{"有 tool call?"}
  Decision -->|否| Final["Final Answer"]
  Decision -->|是| Runtime["ToolRuntime\\nplan + execute"]
  Runtime --> Observation["Tool Observation\\ntool message"]
  Observation --> Messages
  Runtime -.-> Events["StreamEvent\\nUI / 审计 / 调试"]`,
      },
      {
        id: "tool-runtime-plan",
        title: "先规划再执行",
        caption: "ToolRuntimePlan 让未知工具 fail closed，并把 pure function、command、未来 MCP 工具拆开。",
        code: `flowchart TD
  Call["ToolCallFinished\\nname + arguments + index"] --> Find["find tool definition"]
  Find -->|unknown| Failed["ToolRuntimeResult::Failed"]
  Find --> Plan{"ToolRuntimePlan"}
  Plan -->|PureFunction| Pure["run_pure_function"]
  Plan -->|Command| Command["run_shell_command"]
  Pure --> Result["ToolRuntimeResult"]
  Command --> Result`,
      },
      {
        id: "tool-batch",
        title: "同轮多工具并发",
        caption: "同一批 tool call 并发执行，但结果必须按 index/call_id 稳定回灌。",
        code: `flowchart LR
  Batch["tool call batch"] --> A["call 0"]
  Batch --> B["call 1"]
  Batch --> C["call 2"]
  A --> RA["result 0"]
  B --> RB["error 1"]
  C --> RC["result 2"]
  RA --> Sort["sort by index"]
  RB --> Sort
  RC --> Sort
  Sort --> Messages["append tool messages\\n0, 1, 2"]`,
      },
    ],
    sections: [
      {
        title: "三种东西不能混",
        body: "ToolCall 是模型提出的动作请求；Tool execution event 是给 UI/日志/审计看的生命周期；ToolObservation 是给模型下一轮推理的结构化结果。",
        bullets: [
          "事件进入 messages 会污染模型上下文。",
          "observation 不进入 messages，模型就无法基于工具结果继续推理。",
          "外部没有事件，用户无法相信工具没有被绕过执行。",
        ],
      },
      {
        title: "为什么同一批工具可以并发",
        body: "同一个 LLM response 里的多个 tool call 通常不应互相依赖，否则后置参数其实无法可靠生成。并发执行后按 index 回灌，可以让模型一次性看到多个结果或多个错误。",
      },
      {
        title: "什么时候不能这么做",
        body: "如果工具有强副作用、共享资源锁、事务一致性或执行顺序约束，就不能简单互不影响地并发，需要 batch-level policy。",
      },
    ],
    codeAnchors: [
      ["agent/react.rs", "维护 messages、调用 LLM、收集 tool calls、回灌 tool messages。"],
      ["tool/runtime.rs", "工具规划、并发执行、结果按 index 回收。"],
      ["tool/event_emitter.rs", "工具生命周期事件出口。"],
      ["agent/stream_event.rs", "对外流式事件协议。"],
    ],
    failureModes: [
      ["tool result 直接当最终答案", "复杂任务断掉，模型不再基于 observation 推理。", "结果 append 为 tool message，再进入下一轮 LLM。"],
      ["event 和 observation 混用", "UI 噪音污染模型，或模型缺少结构化结果。", "分两条管线：event 给外部，observation 给模型。"],
      ["未知 tool 默认执行", "模型拼错名字也可能进入危险路径。", "`find_tool` miss 后 fail closed。"],
      ["并发结果乱序回灌", "messages 与 call id/index 错位。", "按 index 或 call id 稳定排序。"],
    ],
    checks: [
      "为什么 tool execution event 不应该直接进入 LLM messages？",
      "如果一个 response 里有三个 tool call，为什么通常可以并发？",
      "什么情况下同一批 tool call 不应该互不影响地并发？",
    ],
    sources: [sourceLinks[1], sourceLinks[0]],
  },
  {
    id: "sandbox",
    title: "Sandbox 的第一性原理",
    subtitle: "不要赌代码会不会作恶，而是限制它能观察的世界和能造成的副作用。",
    question: "当你必须运行一段不完全可信的代码时，如何限制它能观察什么、修改什么、连接哪里？",
    mentalModel:
      "子进程不是在真空中执行，它默认继承文件系统视图、环境变量、网络、用户权限、cwd、进程可见性和 stdio。Sandbox 的本质是缩小这个继承世界。",
    diagrams: [
      {
        id: "process-inheritance",
        title: "子进程默认继承的世界",
        caption: "本地命令危险，是因为它获得了一组来自宿主机的能力，而不是纯函数输入输出。",
        code: `flowchart TD
  Parent["Agent process"] --> Child["Child process"]
  Parent --> FS["文件系统视图"]
  Parent --> Env["环境变量\\nPATH / token / proxy"]
  Parent --> Net["网络能力"]
  Parent --> User["用户 / 组权限"]
  Parent --> Cwd["当前工作目录"]
  Parent --> Proc["进程 / IPC 可见性"]
  Parent --> Stdio["stdin / stdout / stderr"]`,
      },
      {
        id: "sandbox-family",
        title: "Sandbox 技术族谱",
        caption: "Sandbox 不是一种技术；不同方案限制的是不同维度。",
        code: `flowchart TD
  Sandbox["Sandbox family"] --> Mac["MAC policy\\nsandbox-exec / App Sandbox"]
  Sandbox --> Ns["Namespaces\\nLinux process view"]
  Sandbox --> Cg["cgroups\\nresource control"]
  Sandbox --> Seccomp["seccomp\\nsyscall filter"]
  Sandbox --> Container["Container\\n组合 namespaces/cgroups/fs/caps"]
  Sandbox --> VM["VM\\n硬件虚拟化边界"]
  Sandbox --> Lang["Language sandbox\\nruntime API boundary"]`,
      },
      {
        id: "sandbox-retry",
        title: "Sandbox denied 后的重试门禁",
        caption: "sandbox denied 后不能自动裸跑；必须经过 retry policy 和必要审批。",
        code: `flowchart TD
  Run["SandboxFirst"] --> Success["success"]
  Run --> CmdFail["command failed"]
  Run --> Denied["sandbox denied"]
  CmdFail --> ReturnFail["return failure observation"]
  Denied --> Policy["retry policy + approval policy"]
  Policy -->|not allowed| ReturnFail
  Policy -->|allowed without approval| Host["NoSandbox retry"]
  Policy -->|needs approval| Ask["ApprovalGateway"]
  Ask -->|rejected| ReturnFail
  Ask -->|approved| Host`,
      },
    ],
    sections: [
      {
        title: "Approval 和 Sandbox 不是一层",
        body: "Approval 是“人是否允许做”；Sandbox 是“允许后以什么边界做”。用户批准安装依赖，不代表依赖脚本可以读 Home、外传 token 或修改任意路径。",
      },
      {
        title: "隔离维度要说清楚",
        body: "说“我有 sandbox”没有意义，必须说明限制的是文件系统、网络、进程视图、用户权限、系统调用、资源还是语言 API。不同方案覆盖不同维度。",
      },
      {
        title: "为什么先模拟，再接 OS runner",
        body: "如果状态机还没稳定，直接接真实 sandbox 会把策略问题和 OS 细节混在一起。先用 SimulatedExecutionRunner 验证 approval/retry/event/ReAct observation，再用 OsExecutionRunner 暴露真实 argv、quoting、profile 和 stderr 分类问题。",
      },
    ],
    codeAnchors: [
      ["tool/shell/execution/simulated_execution_runner.rs", "先模拟 sandbox 结果，验证状态机。"],
      ["tool/shell/execution/os_execution_runner.rs", "用 macOS sandbox-exec 执行真实命令。"],
      ["tool/shell/retry.rs", "sandbox denied 后判断是否允许 no-sandbox retry。"],
    ],
    failureModes: [
      ["把 sandbox 当万能安全", "它可能只限制部分维度，例如文件但不限制网络。", "明确每种方案限制的层。"],
      ["把 command failed 当 sandbox denied", "测试失败也会被误触发提权。", "错误分类必须进入 ExecutionFailure。"],
      ["自动 no-sandbox retry", "等于 sandbox 只是试探而不是边界。", "retry 经过 policy 和 approval。"],
      ["语言 sandbox 替代 OS sandbox", "native escape / FFI / 子进程仍可能逃逸。", "按威胁模型选择隔离层。"],
    ],
    checks: [
      "为什么 approval 通过不等于可以 no-sandbox？",
      "Linux namespace 和 cgroup 分别解决什么问题？",
      "为什么语言 sandbox 不能替代 OS sandbox？",
    ],
    sources: [sourceLinks[2], sourceLinks[0]],
  },
  {
    id: "async",
    title: "Rust 流式 Agent 与 Tokio 运行时",
    subtitle: "用 Future、Waker、Stream 和 mpsc 构造持续吐事件的 Agent runtime。",
    question: "如何让 LLM token、工具事件、审批事件和 UI 刷新持续输出，而不是让用户等一个最终结果？",
    mentalModel:
      "Rust async 的核心不是线程，而是 Future 状态机。Future 被 poll 后要么 Ready，要么保存 Waker 并 Pending；事件就绪后 runtime 再 poll 它。",
    diagrams: [
      {
        id: "future-waker",
        title: "Future / Waker 底层链路",
        caption: "Pending 不是睡眠，而是保存 Waker 后把执行权还给 runtime。",
        code: `sequenceDiagram
  participant Task as Future / task
  participant Runtime as Tokio runtime
  participant Driver as IO driver
  participant OS as OS event queue
  Runtime->>Task: poll()
  Task->>Driver: register fd / timer / channel interest
  Task-->>Runtime: Poll::Pending + Waker
  Runtime->>Runtime: run other ready tasks
  OS-->>Driver: readiness / completion event
  Driver->>Runtime: wake task
  Runtime->>Task: poll() again
  Task-->>Runtime: Poll::Ready(value)`,
      },
      {
        id: "spawn-mpsc",
        title: "spawn + mpsc 的事件出口",
        caption: "后台 Agent task 生产事件，外部 Stream 按自己的节奏消费事件，bounded channel 提供 backpressure。",
        code: `flowchart LR
  AgentTask["后台 Agent task\\nrun_agent_loop"] --> Tx["mpsc::Sender<StreamEvent>"]
  Tx --> Rx["mpsc::Receiver<StreamEvent>"]
  Rx --> Stream["外部 EventStream"]
  Stream --> TUI["TUI / caller"]`,
      },
      {
        id: "sse-parse",
        title: "OpenAI-compatible SSE 到内部事件",
        caption: "chunk、line、delta、message completion 是不同层，不能混为一谈。",
        code: `flowchart LR
  Http["HTTP bytes stream"] --> Utf8["UTF-8 text buffer"]
  Utf8 --> Lines["split SSE lines"]
  Lines --> Data["data: ..."]
  Data --> Json["parse JSON event"]
  Json --> Delta["merge delta\\ntext / thinking / tool args"]
  Delta --> Event["StreamEvent"]`,
      },
    ],
    sections: [
      {
        title: "为什么 Vec<Event> 不够",
        body: "Vec 只能结束后一次性返回。Agent 在一次对话中可能持续产生 thinking、text delta、tool started、approval request、sandbox denied、retry attempt 和 final output，UI 必须边运行边看到进展。",
      },
      {
        title: "async move 与 tokio::spawn",
        body: "`async move` 只是创建 Future，可以在当前函数内 join；`tokio::spawn` 把 Future 交给 runtime 独立调度，通常要求 Send + 'static，因为任务可能在当前栈帧返回后继续运行。",
      },
      {
        title: "为什么不用一个巨大 try_stream!",
        body: "小逻辑可以用 `try_stream!`，但 ReAct loop 要同时处理 LLM stream、tool batch、approval channel、错误传播和 UI 节奏，`spawn + mpsc` 更容易把生产和消费解耦。",
      },
    ],
    codeAnchors: [
      ["agent/llm/openai.rs", "解析 SSE stream，合并 delta。"],
      ["agent/react.rs", "组织 LLM stream、tool call、observation。"],
      ["tool/runtime.rs", "并发执行 tool call batch，按 index 回收。"],
      ["cli/event_loop.rs", "同时处理用户输入和 Agent stream event。"],
    ],
    failureModes: [
      ["bytes chunk 直接当 JSON", "网络 chunk 可能在任意位置切开。", "保留 buffer，只解析完整 SSE line。"],
      ["每个 delta 都 append 新消息", "UI 出现碎片化 thinking/text。", "同类 delta 合并成当前 block。"],
      ["spawn 捕获 &self", "生命周期或 Send + 'static 报错。", "用 Arc/owned data，或不用 spawn。"],
      ["无界 channel", "消费者慢时内存可能失控。", "使用 bounded channel 和 backpressure。"],
    ],
    checks: [
      "为什么 `async move` 不等于创建线程？",
      "为什么 `tokio::spawn` 往往要求 `'static`？",
      "如果 SSE JSON 被拆成两个 TCP chunk，代码应该在哪里缓存？",
    ],
    sources: [sourceLinks[3], sourceLinks[1]],
  },
  {
    id: "tui",
    title: "终端 Agent UI 的事件循环",
    subtitle: "把 key event、Agent stream、approval 和渲染归约成稳定状态机。",
    question: "为什么终端 Agent UI 不是 print 几行文本，而是一个多事件源驱动的状态系统？",
    mentalModel:
      "终端是一块字符网格。程序必须自己读取事件、维护状态、决定何时重绘。Ratatui 只负责根据 state 画一帧；业务状态必须由 event loop 和 reducer 维护。",
    diagrams: [
      {
        id: "tui-architecture",
        title: "TUI 架构分层",
        caption: "Agent 不直接 print，UI 不直接跑工具；事件循环汇合多个事件源，state reducer 归约状态，view 只负责表达。",
        code: `flowchart LR
  Crossterm["Crossterm\\ninput / raw mode / terminal events"] --> Loop["event_loop.rs"]
  Agent["Agent Stream\\nStreamEvent"] --> Loop
  Loop --> App["app.rs\\nstate reducer"]
  App --> View["view.rs\\nRatatui widgets"]
  View --> Terminal["terminal frame"]`,
      },
      {
        id: "tui-event-loop",
        title: "统一事件循环",
        caption: "不能只等 key event；key、agent event、resize、tick 都能驱动状态更新。",
        code: `flowchart TD
  Event{"next event"} -->|Key| Key["handle key"]
  Event -->|Agent StreamEvent| Agent["append transcript / switch mode"]
  Event -->|Resize| Resize["recompute layout"]
  Event -->|Tick| Tick["optional redraw"]
  Key --> State["CliState"]
  Agent --> State
  Resize --> State
  Tick --> State
  State --> Draw["draw current frame"]`,
      },
      {
        id: "approval-ui",
        title: "Approval 是 UI mode",
        caption: "审批不是普通 prompt 输入，而是对特定 scope 的安全决策。",
        code: `flowchart TD
  Event["ToolApprovalRequest"] --> Mode["mode = ApprovalPending"]
  Mode --> Render["draw approval panel"]
  Render --> Key{"user key"}
  Key -->|approve| Send["send ToolApprovalResult approve"]
  Key -->|reject| SendReject["send ToolApprovalResult reject"]
  Send --> Running["mode = RunningAgent"]
  SendReject --> Running`,
      },
    ],
    sections: [
      {
        title: "为什么不能靠用户按键泵事件",
        body: "如果 UI 只有在 key event 到来时才 redraw，Agent stream 就会卡住，直到用户再敲键。正确做法是 key event、agent event、resize 和 tick 都进入统一 event loop。",
      },
      {
        title: "状态层比渲染层重要",
        body: "EditingPrompt、RunningAgent、ApprovalPending、Quit 是不同模式。没有明确 mode，Enter 可能在 approval pending 时被误当成 prompt submit，Agent running 时用户也可能提交新 prompt。",
      },
      {
        title: "Transcript 不是普通日志",
        body: "assistant text/thinking delta 要合并，tool 参数要摘要显示，长输出要支持滚动，手动滚动时不能被自动抢回底部。",
      },
    ],
    codeAnchors: [
      ["cli/event_loop.rs", "合并 key event 与 Agent stream event。"],
      ["cli/app.rs", "保存 input、transcript、mode、pending approval、scroll offset。"],
      ["cli/view.rs", "渲染 header、transcript、prompt/approval、footer。"],
      ["agent/stream_event.rs", "Agent 对外事件协议。"],
    ],
    failureModes: [
      ["阻塞等待键盘", "Agent stream 只有按键后才刷新。", "key event 和 agent event 都进入 event loop。"],
      ["view 修改业务状态", "渲染顺序影响行为，难以测试。", "view 只读 state。"],
      ["approval 用普通输入框", "用户不知道批准范围。", "单独 mode 和 approval panel。"],
      ["不合并 markdown delta", "assistant 输出难读，表格变形。", "transcript block 层合并，再渲染。"],
    ],
    checks: [
      "Ratatui 为什么不负责读取输入？",
      "为什么 event loop 不能只阻塞在 read key？",
      "approval pending 时 Enter 应该提交 prompt 吗？",
    ],
    sources: [sourceLinks[4], sourceLinks[3], sourceLinks[1]],
  },
];

mermaid.initialize({
  startOnLoad: false,
  securityLevel: "strict",
  theme: "base",
  themeVariables: {
    background: "#ffffff",
    primaryColor: "#eff6ff",
    primaryTextColor: "#17202a",
    primaryBorderColor: "#3b82f6",
    lineColor: "#64748b",
    secondaryColor: "#ecfdf5",
    tertiaryColor: "#fff7ed",
    fontFamily:
      'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", sans-serif',
  },
});

function App() {
  const [activeModule, setActiveModule] = useState<AtlasModule>("command");
  const [activeDiagram, setActiveDiagram] = useState(0);
  const [activeScenario, setActiveScenario] = useState("install");

  const module = useMemo(
    () => modules.find((item) => item.id === activeModule) ?? modules[0],
    [activeModule],
  );
  const diagram = module.diagrams[activeDiagram] ?? module.diagrams[0];
  const scenario = commandScenarios.find((item) => item.id === activeScenario) ?? commandScenarios[0];

  const switchModule = (next: AtlasModule) => {
    setActiveModule(next);
    setActiveDiagram(0);
  };

  return (
    <main className="app-shell">
      <header className="hero">
        <nav className="top-nav" aria-label="knowledge atlas navigation">
          <span className="brand">Daedalus Knowledge Atlas</span>
          <a href="#sources">来源索引</a>
        </nav>
        <div className="hero-grid">
          <section>
            <span className="eyebrow">Verified knowledge web</span>
            <h1>Codex 工具系统与权限系统</h1>
            <p>
              这不是 Markdown 镜像，而是一张可交互的学习地图：把命令执行安全、ReAct 工具运行时、
              Sandbox 隔离、Rust async 和 TUI 事件循环组织成机制图、状态机、代码落点和复习题。
            </p>
          </section>
          <aside className="hero-card">
            <strong>网页质量线</strong>
            <span>每个模块必须能独立讲清一个机制：先看第一性原理，再看流程图、代码落点、失败模式和迁移边界。</span>
          </aside>
        </div>
      </header>

      <section className="atlas-layout">
        <aside className="module-rail" aria-label="atlas modules">
          {modules.map((item) => (
            <button
              key={item.id}
              className={item.id === activeModule ? "module-button active" : "module-button"}
              onClick={() => switchModule(item.id)}
            >
              <span>{item.title}</span>
              <small>{item.subtitle}</small>
            </button>
          ))}
        </aside>

        <article className="knowledge-panel">
          <header className="module-header">
            <span className="eyebrow">核心问题</span>
            <h2>{module.question}</h2>
            <p>{module.mentalModel}</p>
          </header>

          {module.id === "command" && (
            <CommandScenarioExplorer
              scenario={scenario}
              activeScenario={activeScenario}
              onScenarioChange={setActiveScenario}
            />
          )}

          <section className="diagram-section">
            <div className="diagram-picker">
              {module.diagrams.map((item, index) => (
                <button
                  key={item.id}
                  className={index === activeDiagram ? "diagram-tab active" : "diagram-tab"}
                  onClick={() => setActiveDiagram(index)}
                >
                  {item.title}
                </button>
              ))}
            </div>
            <MermaidDiagram spec={diagram} />
          </section>

          <section className="deep-sections">
            {module.sections.map((section) => (
              <section className="deep-card" key={section.title}>
                <h3>{section.title}</h3>
                <p>{section.body}</p>
                {section.bullets && (
                  <ul>
                    {section.bullets.map((bullet) => (
                      <li key={bullet}>{bullet}</li>
                    ))}
                  </ul>
                )}
              </section>
            ))}
          </section>

          <TwoColumnEvidence module={module} />

          <section className="review-box">
            <div>
              <span className="eyebrow">自测问题</span>
              <h3>如果这些问题答不出来，就还没有真正掌握。</h3>
            </div>
            <ol>
              {module.checks.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ol>
          </section>
        </article>
      </section>

      <section className="source-band" id="sources">
        <div>
          <span className="eyebrow">Traceability</span>
          <h2>来源和索引</h2>
          <p>网页内容基于这些已验证知识条目重新策展。Markdown 仍是长期知识源，网页负责把复杂机制做成可读、可交互的理解模型。</p>
        </div>
        <div className="source-list">
          {sourceLinks.map((source) => (
            <code key={source.path}>
              {source.label}
              <span>{source.path}</span>
            </code>
          ))}
        </div>
      </section>
    </main>
  );
}

function CommandScenarioExplorer({
  scenario,
  activeScenario,
  onScenarioChange,
}: {
  scenario: CommandScenario;
  activeScenario: string;
  onScenarioChange: (id: string) => void;
}) {
  return (
    <section className="scenario-lab">
      <header>
        <span className="eyebrow">Scenario simulator</span>
        <h3>换一个命令，看安全链路如何重新解释它。</h3>
      </header>
      <div className="scenario-tabs">
        {commandScenarios.map((item) => (
          <button
            key={item.id}
            className={item.id === activeScenario ? "chip active" : "chip"}
            onClick={() => onScenarioChange(item.id)}
          >
            {item.title}
          </button>
        ))}
      </div>
      <div className="scenario-grid">
        <section className="command-card">
          <span className="eyebrow">Command</span>
          <h4>{scenario.command}</h4>
          <p>{scenario.context}</p>
          <blockquote>{scenario.warning}</blockquote>
        </section>
        <section className="policy-stack" aria-label="scenario-picker policy path">
          <PolicyRow label="Capability" value={scenario.capability} />
          <PolicyRow label="Approval" value={scenario.approval} />
          <PolicyRow label="Sandbox" value={scenario.sandbox} />
          <PolicyRow label="Retry" value={scenario.retry} />
          <PolicyRow label="Result" value={scenario.result} />
        </section>
      </div>
    </section>
  );
}

function PolicyRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="policy-row">
      <strong>{label}</strong>
      <span>{value}</span>
    </div>
  );
}

function MermaidDiagram({ spec }: { spec: DiagramSpec }) {
  const [svg, setSvg] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;
    const render = async () => {
      try {
        const id = `diagram-${spec.id}-${Math.random().toString(36).slice(2)}`;
        const result = await mermaid.render(id, spec.code);
        if (!cancelled) {
          setSvg(result.svg);
          setError("");
        }
      } catch (err) {
        if (!cancelled) {
          setSvg("");
          setError(err instanceof Error ? err.message : String(err));
        }
      }
    };

    render();
    return () => {
      cancelled = true;
    };
  }, [spec]);

  return (
    <section className="diagram-card">
      <header>
        <span className="eyebrow">Mermaid diagram</span>
        <h3>{spec.title}</h3>
        <p>{spec.caption}</p>
      </header>
      {error ? (
        <pre className="diagram-error">{error}</pre>
      ) : (
        <div className="mermaid-stage" dangerouslySetInnerHTML={{ __html: svg }} />
      )}
    </section>
  );
}

function TwoColumnEvidence({ module }: { module: KnowledgeModule }) {
  return (
    <section className="evidence-grid">
      <section className="code-anchors">
        <span className="eyebrow">代码落点</span>
        <h3>机制最终要落到代码边界。</h3>
        <div className="anchor-list">
          {module.codeAnchors.map(([file, responsibility]) => (
            <div key={file}>
              <code>{file}</code>
              <span>{responsibility}</span>
            </div>
          ))}
        </div>
      </section>

      <section className="failure-table">
        <span className="eyebrow">常见失败模式</span>
        <h3>这些坑会直接破坏系统边界。</h3>
        <div className="table-head">
          <span>失败模式</span>
          <span>问题</span>
          <span>修正</span>
        </div>
        {module.failureModes.map(([mode, problem, fix]) => (
          <div className="table-row" key={mode}>
            <strong>{mode}</strong>
            <span>{problem}</span>
            <span>{fix}</span>
          </div>
        ))}
      </section>
    </section>
  );
}

export default App;
