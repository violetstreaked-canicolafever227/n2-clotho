// tools/validate.js — clotho_validate: Validate .n2 source syntax and contract integrity
const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function registerValidateTools(server, z, compilerBin) {
    server.tool(
        'clotho_validate',
        'Validate a .n2 AI behavioral contract file. Checks syntax, schema validation, and state machine integrity without generating output files.',
        {
            source: z.string().describe('Absolute path to the .n2 source file'),
        },
        async ({ source }) => {
            try {
                if (!fs.existsSync(source)) {
                    return { content: [{ type: 'text', text: `❌ Source file not found: ${source}` }] };
                }

                const result = execFileSync(compilerBin, ['validate', source], {
                    encoding: 'utf-8',
                    timeout: 15000,
                });

                return {
                    content: [{
                        type: 'text',
                        text: `🧵 **Clotho Validate** — ${path.basename(source)}\n\n${result.trim()}`
                    }]
                };
            } catch (err) {
                const stderr = err.stderr ? err.stderr.toString() : '';
                const stdout = err.stdout ? err.stdout.toString() : '';
                return {
                    content: [{
                        type: 'text',
                        text: `❌ Validation failed:\n${stdout}\n${stderr}`
                    }],
                    isError: true,
                };
            }
        }
    );
}

module.exports = { registerValidateTools };
