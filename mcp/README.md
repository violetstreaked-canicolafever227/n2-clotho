# n2-Clotho MCP Server

Clotho multi-target compiler's MCP server.
Enables AI agents to directly compile/validate `.n2` contracts.

## Installation

```bash
npm install n2-clotho
```

## Tools

| Tool | Description |
|------|-------------|
| `clotho_compile` | Compile to a specific target language |
| `clotho_batch` | Batch compile to all 6 targets |
| `clotho_validate` | Syntax + schema + state machine validation |
| `clotho_backends` | List supported backends |
| `clotho_inspect` | Read compiled contract contents |

## MCP Configuration

```json
{
  "mcpServers": {
    "n2-clotho": {
      "command": "node",
      "args": ["node_modules/n2-clotho/server.js"]
    }
  }
}
```

## WASM Runtime

This package includes a pre-built WASM binary (364KB) — no Rust toolchain required.
The compiler runs entirely in Node.js via WebAssembly.

## Links

- [GitHub](https://github.com/choihyunsus/n2-clotho) — Full documentation & source
- [npm](https://www.npmjs.com/package/n2-clotho)
