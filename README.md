# 🧵 Clotho — The Thread of Fate for AI Agents

> **Markdown rules are dead. Long live `.n2`.**

Clotho is a compiled instruction language for AI agents. It replaces fragile markdown-based rules (GEMINI.md, .cursorrules, CLAUDE.md) with **enforceable, type-checked, deterministic** specifications that agents cannot ignore.

Named after [Clotho](https://en.wikipedia.org/wiki/Clotho), the Greek goddess who spins the thread of fate — because once you define the rules, they become **destiny**.

```
# Before: GEMINI.md (a polite suggestion)
"Please don't run rm -rf. Thanks!"
→ AI: "Sure!" *runs rm -rf anyway*

# After: rules.n2 (compiled law)
@rule NoDestructive {
  blacklist: [/rm -rf/, /DROP TABLE/i]
}
→ AI attempts rm -rf → ❌ BLOCKED. No exceptions.
```

## 🎯 The Problem

### 🧵 Clotho in 4 Panels

<p align="center">
  <img src="docs/images/clotho-comic.png" alt="Clotho 4-panel comic" width="700" />
</p>

> **Panel 1**: AI reads your markdown rules... **Panel 2**: ...then ignores them anyway.
> **Panel 3**: Clotho arrives with `.n2` compiled law. **Panel 4**: Rules enforced. No exceptions.

Every AI coding tool has its own markdown-based rule system:

| Tool | Rule File | Enforcement |
|------|-----------|:-----------:|
| Gemini | `GEMINI.md` | 🙏 Hope |
| Cursor | `.cursorrules` | 🙏 Hope |
| Claude | `CLAUDE.md` | 🙏 Hope |
| Windsurf | `.windsurfrules` | 🙏 Hope |
| System Prompt | `system.txt` | 🙏 Hope |

The result? **Same rules, different results every time.** Agents "read" the rules but follow them inconsistently. There's no compilation, no validation, no enforcement — just vibes.

## 💡 The Solution

One language to replace them all:

```
Before (.md + .json + skill files):          After (.n2):
┌─────────────────────────────────┐    ┌──────────────────────┐
│ GEMINI.md      → behavior rules│    │                      │
│ .cursorrules   → editor rules  │    │   project.n2         │
│ workflows/*.md → skill steps   │    │                      │
│ config.json    → MCP settings  │    │   One file.          │
│ system.txt     → system prompt │    │   Compiled.          │
│ .env           → variables     │    │   Enforced.          │
└─────────────────────────────────┘    └──────────────────────┘
   6+ files, zero enforcement            1 file, full enforcement
```

Clotho introduces `.n2` — a compiled instruction language with:

| Feature | Description |
|---------|-------------|
| **Compilation** | PEG parser → AST → validated execution plan |
| **Type Safety** | Schema-defined types with constraints (`[required]`, `[range: 0..3]`) |
| **Contracts** | State machine behavioral contracts — violation = blocked action |
| **Enforcement** | `strict` / `warn` / `passive` modes per block |
| **Determinism** | Same `.n2` file → always the same execution plan |
| **SQL Queries** | Query your rules like a database: `SELECT * FROM rules WHERE scope = 'command'` |
| **Semantic Matching** | Ollama-powered intent → tool mapping (optional) |

## ⚡ Quick Start

### Installation

```bash
# From source (Rust required)
git clone https://github.com/choihyunsus/n2-clotho.git
cd n2-clotho/compiler
cargo build --release

# Binary is at target/release/n2-compiler
```

### Write Your First `.n2` File

```n2
# my-rules.n2 — Your first Clotho ruleset

@meta {
  name: "my-project-rules"
  version: "1.0.0"
  description: "AI agent behavior rules"
  enforce: strict
}

@rule NoAutoInstall {
  description: "Block unauthorized package installations"
  scope: command
  enforce: strict

  blacklist: [
    /npm install/,
    /yarn add/,
    /pip install/
  ]
}

@workflow Boot {
  description: "Session startup sequence"
  trigger: session_start
  enforce: strict

  step initialize {
    description: "Load project context"
    action: load_context()
    required: true
    timeout: 15s
  }

  step greet {
    depends_on: initialize
    action: compose_response()
  }
}
```

### Compile & Validate

```bash
# Parse and output AST
n2c my-rules.n2

# Full validation pipeline: parse → schema check → contract verify
n2c validate my-rules.n2

# Simulate state machine contracts
n2c simulate my-rules.n2

# Query rules with SQL
n2c query my-rules.n2 "SELECT * FROM rules WHERE enforce = 'strict'"
```

### 🔥 Real Output: Auto-Build Pipeline Validation

Here's actual `n2c validate` output from `auto-build.n2` — a complete build pipeline with state machine, security rules, and SQL queries:

```
🔧 n2c v0.3.0 — Clotho Compiler
📄 File: auto-build.n2

── Step 1: Parse ✅
📊 Blocks: @meta:1 @contract:1 @rule:1 @workflow:1 @query:1 | Total 5

── Step 2: Schema Validation
  ✅ All checks passed! 0 errors, 0 warnings

── Step 3: Contract Check
  State Machine: BuildLifecycle (initial: IDLE, 7 transitions)
     IDLE -[start_build]-> PLANNING
     PLANNING -[plan_complete]-> DESIGNING
     DESIGNING -[design_complete]-> IMPLEMENTING
     IMPLEMENTING -[code_complete]-> TESTING
     TESTING -[tests_pass]-> DEPLOYING
     TESTING -[tests_fail]-> IMPLEMENTING     ← auto-rollback on failure
     DEPLOYING -[deploy_complete]-> IDLE
  ✅ State machine integrity verified!

✅ Validation complete: all checks passed!
```

SQL query on the same file:

```
$ n2c query auto-build.n2 "SELECT * FROM rules"

📦 Registry: 1 rules, 1 contracts, 1 workflows, 0 tools

name           | scope   | enforce | checks | blacklist
───────────────+─────────+─────────+────────+──────────
NoBuildHazards | command | strict  | 0      | 4
```

> Every block is parsed, validated, and queryable. This is not a mockup — this is **real compiler output**.

## 📐 Language Specification

### 8 Block Types

Every `.n2` file is composed of `@`-prefixed blocks:

```n2
@meta { ... }           # File metadata (required)
@import { ... }         # Import other .n2 files
@schema { ... }         # Type & schema definitions
@contract { ... }       # Behavioral contracts (state machines)
@rule { ... }           # Enforcement rules (blacklists, checks)
@workflow { ... }       # Enforced step-by-step workflows
@query { ... }          # SQL-based rule queries
@semantic { ... }       # Semantic matching (Ollama integration)
```

### @contract — State Machine Behavioral Contracts

The most powerful block. Defines state transitions that **cannot be violated**:

```n2
@contract SessionLifecycle {
  scope: session
  states: SessionState

  transitions {
    IDLE -> BOOTING : on boot
    BOOTING -> READY : on boot_complete
    READY -> WORKING : on work_start
    WORKING -> WORKING : on work_log      # Self-transition allowed
    WORKING -> IDLE : on work_end
  }

  invariant {
    on work_start requires state == READY
    => "Must complete boot before starting work"

    on file_modify requires state == WORKING
    => "Must call work_start before modifying files"
  }
}
```

### @rule — Enforcement Rules

Define checks and blacklists that block actions in real-time:

```n2
@rule DestructiveCommandBlock {
  description: "Block destructive commands"
  scope: command
  enforce: strict

  blacklist: [
    /rm -rf/,
    /git push --force/,
    /DROP TABLE/i,
    /TRUNCATE/i,
    /expo prebuild --clean/
  ]
}
```

### @workflow — Enforced Workflows

Step-by-step execution flows with dependency chains, timeouts, and retry logic:

```n2
@workflow AutoBuild {
  description: "Automated build pipeline"
  trigger: on_command("build")
  enforce: strict
  interrupt: false          # No mid-process reporting

  step plan {
    action: generate_plan(topic: $INPUT)
    output -> $PLAN
  }

  step design {
    depends_on: plan
    action: create_design(prompt: $PLAN.ui_description)
    output -> $DESIGN
  }

  step implement {
    depends_on: design
    action: code_from_design(design: $DESIGN)
    output -> $CODE
  }

  step verify {
    depends_on: implement
    action: run_tests($CODE)
    expect {
      all_pass => continue
      fail => goto implement with { fix: $ERRORS }
    }
  }
}
```

### @query — SQL-Based Rule Queries

Query your rules like a relational database:

```n2
@query FindSecurityRules {
  sql {
    SELECT name, description, enforce
    FROM rules
    WHERE scope = 'command'
      AND enforce = 'strict'
    ORDER BY name
  }
}
```

```bash
$ n2c query rules.n2 "SELECT * FROM rules"

📋 Registry: 3 rules, 2 workflows, 1 contract

┌──────────────────────────┬─────────┬─────────┐
│ name                     │ scope   │ enforce │
├──────────────────────────┼─────────┼─────────┤
│ NoAutoInstall            │ command │ strict  │
│ DestructiveCommandBlock  │ command │ strict  │
│ NamingConvention         │ response│ strict  │
└──────────────────────────┴─────────┴─────────┘
```

## 🏗️ Compiler Pipeline

```
.n2 source file
    ↓
[1. Lexer]              PEG tokenization (pest)
    ↓
[2. Parser]             AST generation (N2File → Blocks)
    ↓
[3. Schema Validator]   Type & constraint checking
    ↓
[4. Contract Checker]   State machine integrity verification
    ↓
[5. Query Optimizer]    SQL query validation
    ↓
[6. Codegen]            Execution plan (.n2.lock)
    ↓
[7. Runtime]            Enforced execution ← requires Soul/QLN
```

**Stages 1–6** are handled by Clotho (this tool).
**Stage 7** is handled by the [N2 Soul](https://github.com/choihyunsus/soul) runtime.

## 🕸️ N2 Ecosystem Integration

Clotho is part of the N2 ecosystem — a suite of MCP-native tools for AI agent development:

```
┌─────────────────────────────────────────────────┐
│                  N2 Ecosystem                   │
│                                                 │
│  🧵 Clotho    → Compile rules (.n2 → AST)      │
│       ↓                                         │
│  🧠 Soul      → Runtime enforcement + memory    │
│       ↓                                         │
│  🕷️ Arachne   → Code context assembly           │
│       ↓                                         │
│  🛡️ Ark       → Security verification           │
│       ↓                                         │
│  🌐 N2 Browser → All-in-one AI browser          │
│                                                 │
└─────────────────────────────────────────────────┘
```

| Package | Role | npm |
|---------|------|-----|
| **Clotho** | Rule compiler & validator | `n2-clotho` |
| **Soul** | Agent memory & runtime enforcement | `n2-soul` |
| **Arachne** | Code context assembly (BM25 + semantic) | `n2-arachne` |
| **Ark** | Security gate & audit | `n2-ark` |
| **QLN** | Tool orchestration & routing | `n2-qln` |

### Standalone vs Integrated

| Mode | What You Get |
|------|-------------|
| **Standalone** (Clotho only) | Parse, validate, simulate, query `.n2` files |
| **+ Soul** | Runtime enforcement — contracts block violations in real-time |
| **+ Arachne** | Code-aware rules — contracts reference actual codebase context |
| **+ Full Stack** | Complete AI agent governance with memory, security, and tools |

## 🆚 Markdown Rules vs `.n2`

| Aspect | Markdown (GEMINI.md) | Clotho (.n2) |
|--------|---------------------|-------------|
| **Format** | Free-form text | Structured blocks |
| **Parsing** | Best-effort | PEG grammar → AST |
| **Validation** | None | Schema + contract + SQL |
| **Enforcement** | AI discretion 🙏 | Compiler-enforced ❌ |
| **State tracking** | None | State machine contracts |
| **Querying** | Ctrl+F | SQL queries |
| **Determinism** | ❌ Different every time | ✅ Same input → same plan |
| **Cross-agent** | Copy-paste | `@import` from shared `.n2` files |
| **Debugging** | Read the whole doc | `n2c validate` pinpoints errors |

## 📁 Project Structure

```
n2-clotho/
├── compiler/
│   ├── src/
│   │   ├── grammar.pest     # PEG grammar (179 rules)
│   │   ├── parser.rs        # .n2 → AST
│   │   ├── ast.rs           # AST type definitions
│   │   ├── validator.rs     # Schema validation
│   │   ├── contract.rs      # State machine runtime
│   │   ├── query.rs         # SQL query engine
│   │   ├── wasm.rs          # WASM bindings
│   │   ├── lib.rs           # Library entry
│   │   └── main.rs          # CLI entry (n2c)
│   ├── examples/
│   │   ├── soul-boot.n2        # Boot sequence example
│   │   ├── soul-full-rules.n2  # Full ruleset example
│   │   └── auto-build.n2       # Build pipeline + state machine
│   └── Cargo.toml
├── docs/
│   ├── 001-project-overview.md
│   ├── 002-qln2-architecture-research.md
│   ├── 003-n2-language-spec.md    # Full language specification
│   └── 004-n2-runtime-manual.md
└── README.md
```

## 🛡️ Built With

- **Rust** — Zero-cost abstractions, memory safety
- **pest** — PEG parser generator for the `.n2` grammar
- **serde** — Serialization for AST ↔ JSON
- **WASM** — Optional browser/Node.js compilation target

## 📄 License

Apache-2.0 — Free to use, modify, and distribute.

## 🔗 Links

- [N2 Soul](https://github.com/choihyunsus/soul) — Agent memory & runtime
- [N2 Arachne](https://github.com/choihyunsus/n2-arachne) — Code context assembly
- [N2 QLN](https://github.com/choihyunsus/n2-qln) — Tool orchestration & routing
- [N2 Ark](https://github.com/choihyunsus/n2-ark) — Security verification
- [Language Spec](docs/003-n2-language-spec.md) — Full `.n2` grammar specification

---

## ⭐ Star History

If you find Clotho helpful, please consider giving us a star! ⭐

[![Star History Chart](https://api.star-history.com/svg?repos=choihyunsus/n2-clotho&type=Date)](https://star-history.com/#choihyunsus/n2-clotho&Date)

---

> *"Markdown rules are suggestions. Clotho rules are law."*
