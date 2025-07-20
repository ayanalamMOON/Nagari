# Nagari Runtime

Production-ready runtime utilities and interoperability layer for the Nagari programming language with enhanced f-string format specifier support, HTTP utilities, Python-like builtins, and comprehensive JavaScript integration.

## Version 0.3.1

### 🎉 New Features
- **Enhanced F-String Support**: Full Python-style format specifier compatibility
- **Percentage Formatting**: Complete support for percentage format specifiers (`:`, `:.N%`)
- **Extended Format Utilities**: Additional helper functions for currency, scientific notation, and number formatting
- **Improved String Formatting**: Enhanced padding, alignment, and formatting utilities

## Features

### Core Runtime
- **Python-like Builtins**: `len()`, `type()`, `str()`, `int()`, `float()`, `bool()`, `range()`
- **String Utilities**: Advanced string manipulation functions
- **Format Specifiers**: Complete Python f-string format specifier support
- **Async/Await**: Full async/await support with JavaScript interoperability
- **Type System**: Runtime type checking and conversion utilities

### Format Specifiers
- **Percentage**: `{value:%}`, `{value:.2%}`
- **Float**: `{value:.2f}`, `{value:.6f}`
- **Integer**: `{value:04d}`, `{value:d}`
- **Hex/Binary**: `{value:x}`, `{value:X}`, `{value:b}`, `{value:o}`
- **String Alignment**: `{value:<10s}`, `{value:>10s}`, `{value:^10s}`

### String Functions
```javascript
import {
  str_capitalize, str_title, str_reverse, str_count,
  str_pad_left, str_pad_right, str_center,
  format_percentage, format_currency, format_number_with_commas
} from 'nagari-runtime';

// String manipulation
str_capitalize("hello world");  // "Hello world"
str_title("hello world");       // "Hello World"
str_reverse("hello");           // "olleh"

// Formatting utilities
format_percentage(0.1534, 2);   // "15.34%"
format_currency(123.45);        // "$123.45"
format_number_with_commas(1234567); // "1,234,567"
```

### Interoperability
- **JavaScript Integration**: Seamless conversion between Nagari and JavaScript types
- **React Support**: Built-in JSX and React component support
- **Module System**: ES6/CommonJS module compatibility
- **Node.js Polyfills**: Browser compatibility layer

## Installation

```bash
npm install nagari-runtime
```

## Usage

### Basic Import
```javascript
import { InteropRegistry, len, str, format_percentage } from 'nagari-runtime';

// Initialize runtime
InteropRegistry.initialize();

// Use Python-like functions
console.log(len([1, 2, 3]));     // 3
console.log(str(true));          // "true"
console.log(format_percentage(0.25)); // "25.00%"
```

### Format Specifiers
The runtime works seamlessly with Nagari's transpiled f-string format specifiers:

```nagari
# Nagari code
value = 0.1534
print(f"Percentage: {value:.2%}")
print(f"Currency: ${value * 100:.2f}")
```

Transpiles to:
```javascript
console.log(`Percentage: ${(value * 100).toFixed(2) + '%'}`);
console.log(`Currency: $${(value * 100).toFixed(2)}`);
```

### Advanced Features
```javascript
import {
  jsToNagari, nagariToJS,
  center_string, format_scientific
} from 'nagari-runtime';

// Type conversion
const nagariData = jsToNagari({ name: "Alice", age: 30 });
const jsData = nagariToJS(nagariData);

// Advanced formatting
center_string("Hello", 10, "*");  // "**Hello***"
format_scientific(1234.567, 2);  // "1.23e+3"
```

## Compatibility

- **Node.js**: >=14.0.0
- **Browsers**: ES2018+ (Chrome 63+, Firefox 55+, Safari 12+)
- **TypeScript**: Full type definitions included
- **Module Systems**: ESM and CommonJS

## Changelog

### 0.3.1 (Latest)
- ✅ **Enhanced F-String Support**: Complete Python-style format specifier compatibility
- ✅ **Percentage Formatting**: Full support for `:` and `:.N%` format specifiers
- ✅ **New Format Utilities**: `format_percentage()`, `format_currency()`, `format_number_with_commas()`
- ✅ **Improved String Functions**: Enhanced padding and alignment utilities
- ✅ **Better Documentation**: Comprehensive examples and usage guides

### 0.3.0
- Core runtime stability improvements
- Enhanced interoperability layer
- React/JSX support
- HTTP utilities

## License

MIT License - see LICENSE file for details.

## Contributing

Please read the contributing guidelines in the main Nagari repository: https://github.com/ayanalamMOON/Nagari

## Links

- **Main Repository**: https://github.com/ayanalamMOON/Nagari
- **Documentation**: https://github.com/ayanalamMOON/Nagari#readme
- **Issues**: https://github.com/ayanalamMOON/Nagari/issues
- **NPM Package**: https://www.npmjs.com/package/nagari-runtime
