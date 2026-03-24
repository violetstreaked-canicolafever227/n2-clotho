// tools/backends.js — clotho_backends: List all supported compilation targets
const { execFileSync } = require('child_process');

function registerBackendTools(server, z, compilerBin) {
    server.tool(
        'clotho_backends',
        'List all supported compilation target languages and their file extensions. Returns the available backends for the Clotho multi-target compiler.',
        {},
        async () => {
            try {
                const result = execFileSync(compilerBin, ['backends'], {
                    encoding: 'utf-8',
                    timeout: 5000,
                });

                return { content: [{ type: 'text', text: result.trim() }] };
            } catch (err) {
                return {
                    content: [{ type: 'text', text: `❌ Error: ${err.message}` }],
                    isError: true,
                };
            }
        }
    );
}

module.exports = { registerBackendTools };
