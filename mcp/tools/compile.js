// tools/compile.js — clotho_compile: Compile .n2 source to a specific target language
const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function registerCompileTools(server, z, compilerBin) {
    server.tool(
        'clotho_compile',
        'Compile a .n2 AI behavioral contract to a specific target language. Outputs executable contract code in the selected language.',
        {
            source: z.string().describe('Absolute path to the .n2 source file'),
            target: z.enum(['rust', 'c', 'cpp', 'go', 'python', 'ts'])
                .describe('Target language for compilation'),
        },
        async ({ source, target }) => {
            try {
                if (!fs.existsSync(source)) {
                    return { content: [{ type: 'text', text: `❌ Source file not found: ${source}` }] };
                }

                const result = execFileSync(compilerBin, ['compile', source, target], {
                    encoding: 'utf-8',
                    timeout: 30000,
                });

                // Read the compiled output
                const extMap = { rust: '.n2rs', c: '.n2c', cpp: '.n2c2', go: '.n2go', python: '.n2py', ts: '.n2ts' };
                const basePath = source.replace(/\.n2$/, '');
                const outPath = basePath + extMap[target];

                let compiled = '';
                if (fs.existsSync(outPath)) {
                    compiled = fs.readFileSync(outPath, 'utf-8');
                }

                const summary = [
                    `🧵 **Clotho Compile** — ${target} target`,
                    `📄 Source: ${path.basename(source)}`,
                    `📦 Output: ${path.basename(outPath)} (${compiled.length} bytes)`,
                    ``,
                    result.trim(),
                    ``,
                    `---`,
                    `**Compiled output:**`,
                    '```' + (target === 'ts' ? 'typescript' : target === 'cpp' ? 'cpp' : target) ,
                    compiled,
                    '```',
                ];

                return { content: [{ type: 'text', text: summary.join('\n') }] };
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
