# Nagari Bytecode Format Specification

## Version 0.1

### File Format

Nagari bytecode files use the `.nac` extension and follow this binary format:

```
Magic Number:    4 bytes (NAG\0)
Version:         2 bytes (major.minor)
Constants:       Variable length
Names:           Variable length
Instructions:    Variable length
```

### Constants Section

```
Count:           4 bytes (little-endian u32)
Constants:       Variable length array
```

Each constant has this format:

```
Type Tag:        1 byte
Data:            Variable length
```

Type tags:

- `0`: int (8 bytes, little-endian i64)
- `1`: float (8 bytes, little-endian f64)
- `2`: string (4 bytes length + UTF-8 data)
- `3`: bool (1 byte: 0=false, 1=true)
- `4`: none (no data)

### Names Section

```
Count:           4 bytes (little-endian u32)
Names:           Variable length array
```

Each name:

```
Length:          4 bytes (little-endian u32)
Data:            UTF-8 string
```

### Instructions Section

```
Count:           4 bytes (little-endian u32)
Instructions:    Fixed length array
```

Each instruction:

```
Opcode:          1 byte
Operand:         4 bytes (little-endian u32)
```

### Opcodes

| Opcode | Name | Description |
|--------|------|-------------|
| 0x01 | LoadConst | Push constant onto stack |
| 0x02 | LoadName | Push variable value onto stack |
| 0x03 | StoreName | Pop value and store in variable |
| 0x04 | CallFunc | Call function with N arguments |
| 0x05 | Return | Return from function |
| 0x06 | JumpIfFalse | Jump if top of stack is false |
| 0x07 | Jump | Unconditional jump |
| 0x08 | Pop | Pop top of stack |
| 0x09 | BinaryAdd | Add top two stack values |
| 0x0A | BinarySubtract | Subtract top two stack values |
| 0x0B | BinaryMultiply | Multiply top two stack values |
| 0x0C | BinaryDivide | Divide top two stack values |
| 0x0D | BinaryModulo | Modulo top two stack values |
| 0x0E | BinaryEqual | Compare top two stack values for equality |
| 0x0F | BinaryNotEqual | Compare top two stack values for inequality |
| 0x10 | BinaryLess | Less than comparison |
| 0x11 | BinaryGreater | Greater than comparison |
| 0x12 | BinaryLessEqual | Less than or equal comparison |
| 0x13 | BinaryGreaterEqual | Greater than or equal comparison |
| 0x14 | Print | Print N values from stack |
| 0x15 | BuildList | Build list from N stack values |
| 0x16 | BuildDict | Build dict from N key-value pairs |
| 0x17 | GetItem | Get item from collection |
| 0x18 | SetItem | Set item in collection |
| 0x19 | ForIter | Iterate over collection |
| 0x1A | BreakLoop | Break from loop |
| 0x1B | ContinueLoop | Continue loop |
| 0x1C | SetupLoop | Setup loop context |
| 0x1D | PopBlock | Pop block context |
| 0x1E | Await | Await async operation |
