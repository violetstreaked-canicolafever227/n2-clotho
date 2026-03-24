// n2-clotho MCP Server — Multi-Target AI Contract Compiler
// Supports both stdio and SSE transport modes

const { McpServer } = require('@modelcontextprotocol/sdk/server/mcp.js');
const { StdioServerTransport } = require('@modelcontextprotocol/sdk/server/stdio.js');
const { z } = require('zod');
const path = require('path');
const pkg = require('./package.json');

// Resolve compiler binary path
const COMPILER_BIN = path.resolve(__dirname, '..', 'compiler', 'target', 'release',
    process.platform === 'win32' ? 'n2-compiler.exe' : 'n2-compiler'
);

const server = new McpServer({
    name: 'n2-clotho',
    version: pkg.version,
});

// ═══════════════════════════════════════════════════════
// Tool Registration
// ═══════════════════════════════════════════════════════

const { registerCompileTools } = require('./tools/compile');
const { registerBatchTools } = require('./tools/batch');
const { registerValidateTools } = require('./tools/validate');
const { registerBackendTools } = require('./tools/backends');
const { registerInspectTools } = require('./tools/inspect');

registerCompileTools(server, z, COMPILER_BIN);
registerBatchTools(server, z, COMPILER_BIN);
registerValidateTools(server, z, COMPILER_BIN);
registerBackendTools(server, z, COMPILER_BIN);
registerInspectTools(server, z, COMPILER_BIN);

// ═══════════════════════════════════════════════════════
// Boot
// ═══════════════════════════════════════════════════════

async function boot() {
    const mode = process.env.CLOTHO_TRANSPORT || 'stdio';

    if (mode === 'sse') {
        const { SSEServerTransport } = require('@modelcontextprotocol/sdk/server/sse.js');
        const http = require('http');
        const port = parseInt(process.env.CLOTHO_PORT || '3200', 10);

        let transport = null;

        const httpServer = http.createServer(async (req, res) => {
            res.setHeader('Access-Control-Allow-Origin', '*');
            res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
            res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

            if (req.method === 'OPTIONS') {
                res.writeHead(204);
                res.end();
                return;
            }

            const url = new URL(req.url, `http://localhost:${port}`);

            if (url.pathname === '/sse') {
                transport = new SSEServerTransport('/messages', res);
                await server.connect(transport);
            } else if (url.pathname === '/messages' && req.method === 'POST') {
                if (transport) {
                    await transport.handlePostMessage(req, res);
                } else {
                    res.writeHead(400);
                    res.end('No active SSE connection');
                }
            } else if (url.pathname === '/health') {
                res.writeHead(200, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({
                    status: 'ok',
                    name: 'n2-clotho',
                    version: pkg.version,
                    compiler: COMPILER_BIN,
                }));
            } else {
                res.writeHead(404);
                res.end('Not Found');
            }
        });

        httpServer.listen(port, () => {
            console.log(`[n2-clotho] SSE server running on port ${port}`);
            console.log(`[n2-clotho] Health: http://localhost:${port}/health`);
        });
    } else {
        const transport = new StdioServerTransport();
        await server.connect(transport);
    }
}

boot().catch(err => {
    console.error(`[n2-clotho] Fatal: ${err.message}`);
    process.exit(1);
});
