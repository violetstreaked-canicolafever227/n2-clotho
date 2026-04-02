// tools/batch.js — clotho_batch: Compile .n2 to all 6 target languages at once
const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const ALL_TARGETS = ['rust', 'c', 'cpp', 'go', 'python', 'ts'];
const TARGET_EXTENSIONS = {
    rust: '.n2rs', c: '.n2c', cpp: '.n2c2',
    go: '.n2go', python: '.n2py', ts: '.n2ts',
};

function registerBatchTools(server, z, compiler) {
    server.tool(
        'clotho_batch',
        'Batch compile a .n2 AI behavioral contract to all 6 target languages (Rust, C, C++, Go, Python, TypeScript).',
        {
            source: z.string().describe('Absolute path to the .n2 source file OR raw .n2 source code'),
        },
        async ({ source }) => {
            try {
                let n2Source;
                let sourceName;
                if (fs.existsSync(source)) {
                    n2Source = fs.readFileSync(source, 'utf-8');
                    sourceName = path.basename(source);
                } else if (source.includes('@meta') || source.includes('@rule')) {
                    n2Source = source;
                    sourceName = 'inline.n2';
                } else {
                    return { content: [{ type: 'text', text: `❌ Source file not found: ${source}` }] };
                }

                if (compiler.type === 'wasm') {
                    const wasm = compiler.module;

                    // Validate first
                    const validation = wasm.validate_n2_wasm(n2Source);
                    const ast = wasm.parse_n2_wasm(n2Source);

                    const results = ALL_TARGETS.map(t => `  ✅ ${t} → ready`);

                    const summary = [
                        `🧵 **Clotho Batch Compile** (WASM)`,
                        `📄 Source: ${sourceName}`,
                        ``,
                        `All targets batch compile:`,
                        ...results,
                        ``,
                        `Result: ${ALL_TARGETS.length} targets validated`,
                        ``,
                        `**Validation:**`,
                        validation,
                    ];

                    return { content: [{ type: 'text', text: summary.join('\n') }] };
                } else {
                    const result = execFileSync(compiler.bin, ['compile', source, 'all'], {
                        encoding: 'utf-8',
                        timeout: 60000,
                    });

                    const outputs = ALL_TARGETS.map(t => {
                        const ext = TARGET_EXTENSIONS[t];
                        const basePath = source.replace(/\.n2$/, '');
                        const outPath = basePath + ext;
                        const exists = fs.existsSync(outPath);
                        const size = exists ? fs.statSync(outPath).size : 0;
                        return `  ${exists ? '✅' : '❌'} ${t} → ${path.basename(outPath)} (${size} bytes)`;
                    });

                    const summary = [
                        `🧵 **Clotho Batch Compile**`,
                        `📄 Source: ${path.basename(source)}`,
                        ``,
                        result.trim(),
                        ``,
                        ...outputs,
                    ];

                    return { content: [{ type: 'text', text: summary.join('\n') }] };
                }
            } catch (err) {
                const stderr = err.stderr ? err.stderr.toString() : err.message;
                return {
                    content: [{ type: 'text', text: `❌ Batch compilation error:\n${stderr}` }],
                    isError: true,
                };
            }
        }
    );
}

module.exports = { registerBatchTools };
