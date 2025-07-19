// Basic test for nagari-runtime
import { str_capitalize, str_center, str_count, str_pad_left, str_pad_right, str_reverse, str_title } from '../dist/index.js';

describe('Nagari Runtime String Functions', () => {
    test('str_capitalize should capitalize first character', () => {
        expect(str_capitalize('hello world')).toBe('Hello world');
        expect(str_capitalize('HELLO')).toBe('HELLO');
        expect(str_capitalize('')).toBe('');
    });

    test('str_title should convert to title case', () => {
        expect(str_title('hello world')).toBe('Hello World');
        expect(str_title('programming with nagari')).toBe('Programming With Nagari');
    });

    test('str_reverse should reverse string', () => {
        expect(str_reverse('hello')).toBe('olleh');
        expect(str_reverse('Nagari')).toBe('iragaN');
        expect(str_reverse('')).toBe('');
    });

    test('str_count should count substring occurrences', () => {
        expect(str_count('programming with nagari is amazing', 'a')).toBe(4);
        expect(str_count('hello hello hello', 'hello')).toBe(3);
        expect(str_count('test', 'xyz')).toBe(0);
    });

    test('str_pad_left should left-pad string', () => {
        expect(str_pad_left('Nagari', 10)).toBe('    Nagari');
        expect(str_pad_left('test', 6, '*')).toBe('**test');
    });

    test('str_pad_right should right-pad string', () => {
        expect(str_pad_right('Nagari', 10)).toBe('Nagari    ');
        expect(str_pad_right('test', 6, '*')).toBe('test**');
    });

    test('str_center should center string', () => {
        expect(str_center('Nagari', 10)).toBe('  Nagari  ');
        expect(str_center('test', 8, '*')).toBe('**test**');
    });
});

describe('Basic Runtime Functionality', () => {
    test('runtime module loads correctly', () => {
        expect(typeof str_capitalize).toBe('function');
        expect(typeof str_title).toBe('function');
        expect(typeof str_reverse).toBe('function');
        expect(typeof str_count).toBe('function');
        expect(typeof str_pad_left).toBe('function');
        expect(typeof str_pad_right).toBe('function');
        expect(typeof str_center).toBe('function');
    });
});
