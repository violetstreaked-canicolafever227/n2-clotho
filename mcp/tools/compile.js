// tools/compile.js — clotho_compile: Compile .n2 source to a specific target language
const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const TARGET_EXTENSIONS = {
    rust: '.n2rs', c: '.n2c', cpp: '.n2c2',
    go: '.n2go', python: '.n2py', ts: '.n2ts',
};

function registerCompileTools(server, z, compiler) {
    server.tool(
        'clotho_compile',
        'Compile a .n2 AI behavioral contract to a specific target language. Outputs executable contract code in the selected language.',
        {
            source: z.string().describe('Absolute path to the .n2 source file OR raw .n2 source code'),
            target: z.enum(['rust', 'c', 'cpp', 'go', 'python', 'ts'])
                .describe('Target language for compilation'),
        },
        async ({ source, target }) => {
            try {
                // Detect if source is a file path or raw code
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
                    // WASM path — use bundled compiler
                    const wasm = compiler.module;
                    const ast = wasm.parse_n2_wasm(n2Source);
                    const validation = wasm.validate_n2_wasm(n2Source);
                    const validationResult = JSON.parse(validation);

                    if (validationResult.errors && validationResult.errors.length > 0) {
                        return {
                            content: [{ type: 'text', text: `❌ Validation failed:\n${validationResult.errors.join('\n')}` }],
                            isError: true,
                        };
                    }

                    const summary = [
                        `🧵 **Clotho Compile** — ${target} target (WASM)`,
                        `📄 Source: ${sourceName}`,
                        `✅ Validation passed`,
                        ``,
                        `**AST:**`,
                        '```json',
                        ast,
                        '```',
                    ];

                    return { content: [{ type: 'text', text: summary.join('\n') }] };
                } else {
                    // Native binary path
                    const result = execFileSync(compiler.bin, ['compile', source, target], {
                        encoding: 'utf-8',
                        timeout: 30000,
                    });

                    const ext = TARGET_EXTENSIONS[target];
                    const basePath = source.replace(/\.n2$/, '');
                    const outPath = basePath + ext;

                    let compiled = '';
                    if (fs.existsSync(outPath)) {
                        compiled = fs.readFileSync(outPath, 'utf-8');
                    }

                    const langMap = { ts: 'typescript', cpp: 'cpp' };
                    const summary = [
                        `🧵 **Clotho Compile** — ${target} target`,
                        `📄 Source: ${path.basename(source)}`,
                        `📦 Output: ${path.basename(outPath)} (${compiled.length} bytes)`,
                        ``,
                        result.trim(),
                        ``,
                        `---`,
                        `**Compiled output:**`,
                        '```' + (langMap[target] || target),
                        compiled,
                        '```',
                    ];

                    return { content: [{ type: 'text', text: summary.join('\n') }] };
                }
            } catch (err) {
                const stderr = err.stderr ? err.stderr.toString() : err.message;
                return {
                    content: [{ type: 'text', text: `❌ Compilation error:\n${stderr}` }],
                    isError: true,
                };
            }
        }
    );
}

module.exports = { registerCompileTools };
