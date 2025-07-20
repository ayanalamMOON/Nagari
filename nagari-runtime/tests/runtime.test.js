// Basic test for nagari-runtime
import { format_currency, format_percentage, str_capitalize, str_center, str_count, str_pad_left, str_pad_right, str_reverse, str_title } from '../dist/index.js';

describe('Nagari Runtime String Functions', () => {
    test('str_capitalize should capitalize first character', () => {
        expect(str_capitalize('hello world')).toBe('Hello world');
        expect(str_capitalize('HELLO')).toBe('Hello');
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
        expect(str_count('programming with nagari is amazing', 'a')).toBe(5);
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

describe('Formatting Functions', () => {
    test('format_percentage should format as percentage', () => {
        expect(format_percentage(0.25)).toBe('25.00%');
        expect(format_percentage(0.1534, 1)).toBe('15.3%');
        expect(format_percentage(0.123456, 3)).toBe('12.346%');
        expect(format_percentage(1.0)).toBe('100.00%');
    });

    test('format_currency should format as currency', () => {
        expect(format_currency(123.45)).toBe('$123.45');
        expect(format_currency(99.9, 1)).toBe('$99.9');
        expect(format_currency(50, 2, '€')).toBe('€50.00');
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
