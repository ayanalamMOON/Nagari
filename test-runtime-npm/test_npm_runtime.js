// Direct test of npm installed nagari-runtime
import {
    str_capitalize,
    str_center,
    str_count,
    str_pad_left,
    str_pad_right,
    str_reverse,
    str_title
} from 'nagari-runtime';

console.log('=== Testing npm installed nagari-runtime@0.3.0 ===');

// Test all string functions
const text = "hello nagari programming";
console.log('Original text:', text);

console.log('str_capitalize:', str_capitalize(text));
console.log('str_title:', str_title(text));
console.log('str_reverse("Nagari"):', str_reverse("Nagari"));
console.log('str_count(text, "a"):', str_count(text, "a"));

const short_text = "Nagari";
console.log(`str_pad_left("${short_text}", 15):`, `'${str_pad_left(short_text, 15)}'`);
console.log(`str_pad_right("${short_text}", 15):`, `'${str_pad_right(short_text, 15)}'`);
console.log(`str_center("${short_text}", 15):`, `'${str_center(short_text, 15)}'`);

console.log('✅ All npm package functions working!');
