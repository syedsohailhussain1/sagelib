const sagelib = require('./sagelib.node');

console.log('--- Testing sagelib Core Engine ---');
const result = sagelib.helloWorld('Developer');
console.log('Result from Rust:', result);
console.log('-----------------------------------');
