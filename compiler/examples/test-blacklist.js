// Blacklist real-world test — validates n2-runtime.js blacklist parsing + matching
const path = require('path');
const fs = require('fs');

// Load n2-runtime.js directly
const runtimePath = path.resolve(__dirname, '../../../n2-browser/soul/lib/n2-runtime.js');
const { N2Runtime } = require(runtimePath);

console.log('=== N2 Blacklist Real-World Test ===\n');

let passed = 0;
let failed = 0;

function assert(condition, msg) {
    if (condition) {
        passed++;
        console.log(`  ✅ ${msg}`);
    } else {
        failed++;
        console.log(`  ❌ ${msg}`);
    }
}

// 1. Load rules
const rulesDir = path.resolve(__dirname, '../../../n2-browser/soul/rules');

if (!fs.existsSync(rulesDir)) {
    console.log('⚠️ soul/rules/ not found — skipping');
    process.exit(0);
}

const runtime = new N2Runtime();
runtime.loadRules(rulesDir);

const ruleNames = Object.keys(runtime.blacklists);
const patternCount = Object.values(runtime.blacklists)
    .reduce((sum, r) => sum + r.patterns.length, 0);
console.log(`1. Loaded ${ruleNames.length} rules, ${patternCount} patterns`);
console.log(`   Rules: ${ruleNames.join(', ')}\n`);

// Helper: check if a command is blocked by a SPECIFIC rule
function isBlockedByRule(cmd, ruleName) {
    const rule = runtime.blacklists[ruleName];
    if (!rule) return false;
    return rule.patterns.some(p => { p.lastIndex = 0; return p.test(cmd); });
}

// 2. NoAutoInstall rule tests
console.log('2. NoAutoInstall rule tests:');
assert(isBlockedByRule('npm install express', 'NoAutoInstall'), 'npm install');
assert(isBlockedByRule('npm install -g pkg', 'NoAutoInstall'), 'npm install -g');
assert(isBlockedByRule('npm i react', 'NoAutoInstall'), 'npm i (short)');
assert(isBlockedByRule('yarn add lodash', 'NoAutoInstall'), 'yarn add');
assert(isBlockedByRule('pip install flask', 'NoAutoInstall'), 'pip install');
assert(!isBlockedByRule('npm run dev', 'NoAutoInstall'), 'npm run dev (allowed)');
assert(!isBlockedByRule('npm test', 'NoAutoInstall'), 'npm test (allowed)');
assert(!isBlockedByRule('cargo build', 'NoAutoInstall'), 'cargo build (allowed)');

// 3. BlockDestructive rule tests
console.log('\n3. BlockDestructive rule tests:');
assert(isBlockedByRule('rm -rf /', 'BlockDestructive'), 'rm -rf');
assert(isBlockedByRule('Remove-Item -Recurse -Force .', 'BlockDestructive'), 'Remove-Item -Recurse -Force');
assert(isBlockedByRule('git push --force', 'BlockDestructive'), 'git push --force');
assert(isBlockedByRule('expo prebuild --clean', 'BlockDestructive'), 'expo prebuild --clean');
assert(isBlockedByRule('DROP TABLE users', 'BlockDestructive'), 'DROP TABLE');
assert(isBlockedByRule('TRUNCATE TABLE sessions', 'BlockDestructive'), 'TRUNCATE');
assert(isBlockedByRule('DELETE FROM orders', 'BlockDestructive'), 'DELETE FROM');
assert(isBlockedByRule('drop table Users', 'BlockDestructive'), 'drop table (case-insensitive)');
assert(!isBlockedByRule('git status', 'BlockDestructive'), 'git status (allowed)');
assert(!isBlockedByRule('git push', 'BlockDestructive'), 'git push (allowed, no --force)');

// 4. Edge case tests (checkBlacklist)
console.log('\n4. Edge case tests (checkBlacklist):');
assert(runtime.checkBlacklist(null).length === 0, 'null input returns empty');
assert(runtime.checkBlacklist('').length === 0, 'empty string returns empty');
assert(runtime.checkBlacklist(undefined).length === 0, 'undefined input returns empty');

// 5. Regex lastIndex safety test
console.log('\n5. lastIndex safety test:');
const testCmd = 'npm install express';
const v1 = runtime.checkBlacklist(testCmd);
const v2 = runtime.checkBlacklist(testCmd);
assert(v1.length === v2.length, `Consistent: call1=${v1.length}, call2=${v2.length}`);

// 6. Hot Reload readiness check
console.log('\n6. Hot Reload structure check:');
assert(typeof runtime.watchRules === 'function', 'watchRules() method exists');
assert(typeof runtime.stopWatching === 'function', 'stopWatching() method exists');
assert(typeof runtime._reloadFile === 'function', '_reloadFile() method exists');
assert(runtime._fileSourceMap !== undefined, '_fileSourceMap initialized');

console.log(`\n=== Results: ${passed} passed, ${failed} failed ===`);
process.exit(failed > 0 ? 1 : 0);
