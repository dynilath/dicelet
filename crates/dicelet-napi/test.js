const { roll, parse } = require('./dicelet.win32-x64-msvc.node');

console.log('=== Dicelet NPM Test ===\n');

// Test 1: Basic dice
console.log('Test 1: Basic dice');
const r1 = roll('4d6k3');
console.log('  Input: 4d6k3');
console.log('  Full:', r1.full);
console.log('  Summary:', r1.summary);
console.log('  Detail:', r1.detail);
console.log('  Is set:', r1.isSet);
console.log();

// Test 2: Complex expression
console.log('Test 2: Complex expression');
const r2 = roll('(((4d6+3)/2+2d20)+4*1d6)*150%');
console.log('  Input: (((4d6+3)/2+2d20)+4*1d6)*150%');
console.log('  Full:', r2.full);
console.log();

// Test 3: Set (repeat)
console.log('Test 3: Set (repeat)');
const r3 = roll('6#4d6k3');
console.log('  Input: 6#4d6k3');
console.log('  Full:', r3.full);
console.log('  Is set:', r3.isSet);
console.log();

// Test 4: Set (braces)
console.log('Test 4: Set (braces)');
const r4 = roll('{4d6,3d6,2d6,1d6}');
console.log('  Input: {4d6,3d6,2d6,1d6}');
console.log('  Full:', r4.full);
console.log();

// Test 5: strtol recovery
console.log('Test 5: strtol recovery');
const r5 = roll('d20 + (d4+ 测试');
console.log('  Input: d20 + (d4+ 测试');
console.log('  Consumed:', JSON.stringify(r5.consumed));
console.log('  Tail:', JSON.stringify(r5.tail));
console.log('  Summary:', r5.summary);
console.log();

// Test 6: Set operation
console.log('Test 6: Set operation');
const r6 = roll('4#d20-{1,2,3,4}');
console.log('  Input: 4#d20-{1,2,3,4}');
console.log('  Full:', r6.full);
console.log();

// Test 7: Parse only
console.log('Test 7: Parse only');
const p7 = parse('d20 + (d4+ 测试');
console.log('  Input: d20 + (d4+ 测试');
console.log('  Success:', p7.success);
console.log('  Consumed:', JSON.stringify(p7.consumed));
console.log('  Tail:', JSON.stringify(p7.tail));
console.log();

// Test 8: No detail
console.log('Test 8: No detail');
const r8 = roll('4d6', { showDetail: false });
console.log('  Input: 4d6 (showDetail: false)');
console.log('  Full:', r8.full);
console.log('  Detail:', JSON.stringify(r8.detail));
console.log();

// Test 9: Constant expression
console.log('Test 9: Constant expression');
const r9 = roll('2+3*4');
console.log('  Input: 2+3*4');
console.log('  Summary:', r9.summary);
console.log();

console.log('=== All tests passed! ===');