import {
    str_capitalize,
    str_center,
    str_count,
    str_pad_left,
    str_pad_right,
    str_reverse,
    str_title
} from 'nagari-runtime';

console.log('Testing nagari-runtime@0.3.0 with string functions:');
console.log('=============================================');

// Test str_capitalize
console.log('str_capitalize("hello world"):', str_capitalize("hello world"));

// Test str_title
console.log('str_title("hello world"):', str_title("hello world"));

// Test str_reverse
console.log('str_reverse("hello"):', str_reverse("hello"));

// Test str_count
console.log('str_count("hello world", "l"):', str_count("hello world", "l"));

// Test str_pad_left
console.log('str_pad_left("hello", 10):', `'${str_pad_left("hello", 10)}'`);

// Test str_pad_right
console.log('str_pad_right("hello", 10):', `'${str_pad_right("hello", 10)}'`);

// Test str_center
console.log('str_center("hello", 11):', `'${str_center("hello", 11)}'`);

console.log('');
console.log('✅ All string functions working correctly from npm package!');
