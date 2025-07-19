import { str_capitalize, str_center, str_count, str_pad_left, str_pad_right, str_reverse, str_title } from 'nagari-runtime';

console.log('Testing str_capitalize:', str_capitalize('hello world'));
console.log('Testing str_title:', str_title('hello world'));
console.log('Testing str_reverse:', str_reverse('hello'));
console.log('Testing str_count:', str_count('hello world', 'l'));
console.log('Testing str_pad_left:', `'${str_pad_left('hello', 10)}'`);
console.log('Testing str_pad_right:', `'${str_pad_right('hello', 10)}'`);
console.log('Testing str_center:', `'${str_center('hello', 11)}'`);
console.log('All string function tests completed!');
