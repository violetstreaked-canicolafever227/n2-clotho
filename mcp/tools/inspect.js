// tools/inspect.js — clotho_inspect: Read and display compiled contract file contents
const fs = require('fs');
const path = require('path');

function registerInspectTools(server, z, compilerBin) {
    server.tool(
        'clotho_inspect',
        'Inspect a compiled .n2 contract file. Reads and displays the generated code with metadata. Supports .n2rs, .n2c, .n2c2, .n2go, .n2py, .n2ts files.',
        {
            file: z.string().describe('Absolute path to the compiled contract file (.n2rs, .n2c, .n2c2, .n2go, .n2py, .n2ts)'),
        },
        async ({ file }) => {
            try {
                if (!fs.existsSync(file)) {
                    return { content: [{ type: 'text', text: `❌ File not found: ${file}` }] };
                }

                const ext = path.extname(file);
                const validExts = ['.n2rs', '.n2c', '.n2c2', '.n2go', '.n2py', '.n2ts'];

                if (!validExts.includes(ext)) {
                    return {
                        content: [{
                            type: 'text',
                            text: `❌ Invalid file extension: ${ext}\nSupported: ${validExts.join(', ')}`
                        }]
                    };
                }

                const content = fs.readFileSync(file, 'utf-8');
                const stat = fs.statSync(file);

                const langMap = {
                    '.n2rs': 'rust', '.n2c': 'c', '.n2c2': 'cpp',
                    '.n2go': 'go', '.n2py': 'python', '.n2ts': 'typescript'
                };

                const targetMap = {
                    '.n2rs': 'Rust', '.n2c': 'C', '.n2c2': 'C++',
                    '.n2go': 'Go', '.n2py': 'Python', '.n2ts': 'TypeScript'
                };

                const lines = content.split('\n').length;

                const summary = [
                    `🧵 **Clotho Inspect** — ${path.basename(file)}`,
                    `🎯 Target: ${targetMap[ext] || 'Unknown'}`,
                    `📊 Size: ${stat.size} bytes | ${lines} lines`,
                    `📅 Modified: ${stat.mtime.toISOString()}`,
                    ``,
                    '```' + (langMap[ext] || ''),
                    content,
                    '```',
                ];

                return { content: [{ type: 'text', text: summary.join('\n') }] };
            } catch (err) {
                return {
                    content: [{ type: 'text', text: `❌ Inspect error: ${err.message}` }],
                    isError: true,
                };
            }
        }
    );
}

module.exports = { registerInspectTools };
