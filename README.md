# Nexus Agent

> An open-source, full-terminal AI coding agent built in Rust.

**Nexus** is a blazing-fast, extensible coding agent for your terminal. It runs as a full-screen TUI (built with [ratatui](https://ratatui.rs/)), connects to any OpenAI-compatible LLM, and provides a layered cognitive stack for understanding, writing, verifying, and reasoning about code.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ⚡ Nexus │ model │ ● idle │ ⏱ 1.2s │ round 3/20 │ tok 1.2k/4k               │ Header
├──────────┬──────────────────────────────────────┬───────────────────────────┤
│ 📁 Files │ 💬 Conversation (streaming)         │ 🧠 Cognitive              │ Sidebar
├──────────┴──────────────────────────────────────┴───────────────────────────┤
│ ▸ Multi-line input                                                     ↵    │ Input
├─────────────────────────────────────────────────────────────────────────────┤
│ F1 Help │ Tab Focus │ Ctrl+C Quit                                              │ Status
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Why Nexus?

Most coding agents are SaaS lock-in, web apps, or thin wrappers around a single model. **Nexus is different:**

- **Local-first** — your code, your API keys, your data
- **Provider-agnostic** — any OpenAI-compatible endpoint (kimchi, OpenAI, LocalAI, Ollama, etc.)
- **Full-terminal** — a real TUI, not a scrolling CLI. Multi-pane layout with file tree, conversation, cognitive state, and input
- **Layered cognitive stack** — three crates that compose into something more powerful than any one tool:
  - `nexus-brain` — pure reasoning engine (verifier, thought chain, risk analyzer, semantic search, code graph, hypothesis engine)
  - `nexus-cognitive` — LLM-callable tool wrappers around brain
  - `nexus-intel` — adaptive learning and persistent memory
- **Zero compromises** — 0 warnings, 125+ tests, type-safe Rust end-to-end

---

## Quick Start

### 1. Install

```bash
git clone https://github.com/vitkuz573/nexus-agent
cd nexus-agent
cargo build --release
```

The binary will be at `target/release/nexus`.

### 2. Configure a provider

```bash
./target/release/nexus init \
  --name kimchi \
  --url https://llm.kimchi.dev/openai/v1 \
  --key <your-api-key> \
  --model minimax-m3
```

The config is stored at `~/.config/nexus-agent/config.toml`.

### 3. Launch the TUI

```bash
./target/release/nexus
```

That's it — you're in a full-screen terminal interface. Type a question, press Enter, watch the agent work.

### Non-interactive use

```bash
./target/release/nexus run "what does this repo do?"
./target/release/nexus providers
./target/release/nexus config
```

---

## Workspace Structure

```
nexus-agent/
├── crates/
│   ├── nexus-config/      TOML config, provider management, settings
│   ├── nexus-client/      OpenAI-compat HTTP client + streaming SSE parser
│   ├── nexus-tools/       Tool trait, registry, bash/read/write/list/grep
│   ├── nexus-core/        Agent loop, context, memory, cognitive scaffold
│   ├── nexus-brain/       Pure reasoning engine (verify, think, risks, search, graph)
│   ├── nexus-cognitive/   LLM-callable tools wrapping brain
│   ├── nexus-intel/       Adaptive learning, pattern matching, long-term memory
│   └── nexus-cli/         Full-screen TUI + CLI subcommands
├── tests/                 Integration tests
├── docker/                Dockerfile for container builds
└── Cargo.toml             Workspace root
```

---

## Cognitive Stack

Nexus is structured as a **layered cognitive architecture** — each crate has a clear role, and they compose without circular dependencies.

### `nexus-brain` — the reasoning engine

Pure logic, no async, no I/O. The foundation.

| Module | What it does |
|---|---|
| `scaffold` | `CognitiveScaffold` — the 6-phase protocol (Understand → Analyze → Design → Implement → Verify → Reflect) |
| `thought` | `ThoughtChain` — tree of `ThoughtNode`s with confidence, parent/children links, branching, trace replay |
| `verify` | `CodeVerifier` — adaptive checks based on code complexity: syntax, patterns, error handling, edge cases, minimalism, consistency, docs, safety |
| `memory` | `MemoryPalace` — room-based memory with items, connections, importance, pruning |
| `diff` | `SemanticDiffEngine` — line-based diff with semantic weighting and impact scoring |
| `graph` | `CodeGraphBuilder` — dependency graph with cyclomatic complexity, coupling, cohesion, god modules |
| `hypothesis` | `HypothesisEngine` — A/B test two code approaches, pick the winner |
| `risk` | `RiskAnalyzer` — detects `unsafe`, `unwrap`/`expect`, memory leaks, infinite loops, hardcoded secrets, SQL injection |
| `search` | `NeuralSearch` — exact + semantic + structural + behavioral matching |
| `architect` | `AutoArchitect` — detects god objects, long methods, design patterns |

### `nexus-cognitive` — the LLM tool surface

Thin wrappers that expose brain capabilities as `ToolInstance` implementations:

- `ThinkTool` — invokes the cognitive scaffold
- `VerifyCodeTool` — runs the code verifier
- `AnalyzeRisksTool` — runs the risk analyzer
- `SearchCodeTool` — runs neural search across the codebase
- `RecallMemoryTool` — recalls from the memory palace

### `nexus-intel` — adaptive learning

The persistent, learning layer:

- `AdaptiveLearner` — records interactions, extracts patterns, suggests approaches/tools, tracks per-tool success rates
- `PatternMatcher` — built-in Rust patterns (Result handling, async/tokio, Option combinators)
- `SuccessPredictor` — predicts confidence, approach, tools, rounds, risk factors from similar historical tasks
- `LongTermMemory` — categorized entries (Decision/Pattern/Error/Learning/Preference/Context) with importance, access counts, pruning, JSON export/import

---

## TUI Keybindings

| Key | Action |
|---|---|
| `Enter` | Send message |
| `Shift+Enter` | New line (multi-line input) |
| `Tab` / `Shift+Tab` | Switch focus between panels |
| `↑` / `↓` | Recall input history |
| `PgUp` / `PgDn` | Scroll conversation |
| `F1` | Toggle help overlay |
| `Ctrl+C` | Quit |

### Slash commands

| Command | Action |
|---|---|
| `/help`, `/?` | Show help |
| `/clear`, `/c` | Clear conversation |
| `/tools` | List available tools |
| `/theme` | Cycle through themes |
| `/model`, `/m` | Show current model |
| `/providers` | List configured providers |
| `/verify` | Re-run verifier on last code |
| `/history` | Show input history |
| `/save <file>` | Save conversation |
| `/quit`, `/q`, `/exit` | Exit Nexus |

---

## Roadmap

- [x] Workspace + 8 crates
- [x] OpenAI-compatible client + streaming
- [x] Tool system (bash, read, write, list, grep)
- [x] Cognitive scaffold + thought chains
- [x] Context-aware code verifier
- [x] Full-screen TUI with multi-panel layout
- [x] Slash commands + multi-line input
- [x] 125+ tests, 0 warnings
- [ ] **Streaming token display** (token-by-token rendering in TUI)
- [ ] **Persistent long-term memory** across sessions (JSON persistence)
- [ ] **Loop detection** (auto-stop when model repeats the same tool call)
- [ ] **Smart bash tool** (filter `target/`, `node_modules/`, add timeout)
- [ ] **Tool call budget** (limit total tool calls per task)
- [ ] **System prompt presets** (analytical / creative / focused)
- [ ] **LSP integration** (real code intelligence)
- [ ] **Diff viewer** in TUI (review changes before applying)
- [ ] **Multi-session** (run multiple agents in parallel)
- [ ] **Web UI** (optional, for those who prefer it)

---

## Contributing

Contributions welcome. The codebase is small, well-tested, and the architecture is designed for extension:

- New **tools** → implement `ToolInstance` in `nexus-tools/src/tools/`
- New **brain modules** → add to `nexus-brain/src/`, register in `lib.rs`
- New **LLM providers** → implement the `LlmProvider` trait (or just configure a new OpenAI-compatible endpoint)
- New **TUI panels** → add to `nexus-cli/src/tui/panels/`, wire into `mod.rs::layout()`

Before submitting a PR:

```bash
cargo test         # all tests pass
cargo check        # 0 warnings
```

---

## License

MIT — see [LICENSE](LICENSE).

---

## Author

Vitaly Kuzyaev &lt;vitkuz573@gmail.com&gt;
