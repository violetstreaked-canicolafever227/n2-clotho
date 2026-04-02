// tools/validate.js — clotho_validate: Full validation pipeline
const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function registerValidateTools(server, z, compiler) {
    server.tool(
        'clotho_validate',
        'Validate a .n2 AI behavioral contract. Runs syntax check, schema validation, and state machine integrity verification.',
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
                    const result = wasm.validate_n2_wasm(n2Source);

                    const summary = [
                        `🧵 **Clotho Validate** (WASM)`,
                        `📄 Source: ${sourceName}`,
                        ``,
                        result,
                    ];

                    return { content: [{ type: 'text', text: summary.join('\n') }] };
                } else {
                    const result = execFileSync(compiler.bin, ['validate', source], {
                        encoding: 'utf-8',
                        timeout: 30000,
                    });

                    const summary = [
                        `🧵 **Clotho Validate**`,
                        `📄 Source: ${sourceName}`,
                        ``,
                        result.trim(),
                    ];

                    return { content: [{ type: 'text', text: summary.join('\n') }] };
                }
            } catch (err) {
                const stderr = err.stderr ? err.stderr.toString() : err.message;
                return {
                    content: [{ type: 'text', text: `❌ Validation error:\n${stderr}` }],
                    isError: true,
                };
            }
        }
    );
}

module.exports = { registerValidateTools };
