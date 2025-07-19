use crate::value::Value;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    LoadConst = 0x01,
    LoadName = 0x02,
    StoreName = 0x03,
    CallFunc = 0x04,
    Return = 0x05,
    JumpIfFalse = 0x06,
    Jump = 0x07,
    Pop = 0x08,
    BinaryAdd = 0x09,
    BinarySubtract = 0x0A,
    BinaryMultiply = 0x0B,
    BinaryDivide = 0x0C,
    BinaryModulo = 0x0D,
    BinaryEqual = 0x0E,
    BinaryNotEqual = 0x0F,
    BinaryLess = 0x10,
    BinaryGreater = 0x11,
    BinaryLessEqual = 0x12,
    BinaryGreaterEqual = 0x13,
    Print = 0x14,
    BuildList = 0x15,
    BuildDict = 0x16,
    GetItem = 0x17,
    SetItem = 0x18,
    ForIter = 0x19,
    BreakLoop = 0x1A,
    ContinueLoop = 0x1B,
    SetupLoop = 0x1C,
    PopBlock = 0x1D,
    Await = 0x1E,
}

impl Opcode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Opcode::LoadConst),
            0x02 => Some(Opcode::LoadName),
            0x03 => Some(Opcode::StoreName),
            0x04 => Some(Opcode::CallFunc),
            0x05 => Some(Opcode::Return),
            0x06 => Some(Opcode::JumpIfFalse),
            0x07 => Some(Opcode::Jump),
            0x08 => Some(Opcode::Pop),
            0x09 => Some(Opcode::BinaryAdd),
            0x0A => Some(Opcode::BinarySubtract),
            0x0B => Some(Opcode::BinaryMultiply),
            0x0C => Some(Opcode::BinaryDivide),
            0x0D => Some(Opcode::BinaryModulo),
            0x0E => Some(Opcode::BinaryEqual),
            0x0F => Some(Opcode::BinaryNotEqual),
            0x10 => Some(Opcode::BinaryLess),
            0x11 => Some(Opcode::BinaryGreater),
            0x12 => Some(Opcode::BinaryLessEqual),
            0x13 => Some(Opcode::BinaryGreaterEqual),
            0x14 => Some(Opcode::Print),
            0x15 => Some(Opcode::BuildList),
            0x16 => Some(Opcode::BuildDict),
            0x17 => Some(Opcode::GetItem),
            0x18 => Some(Opcode::SetItem),
            0x19 => Some(Opcode::ForIter),
            0x1A => Some(Opcode::BreakLoop),
            0x1B => Some(Opcode::ContinueLoop),
            0x1C => Some(Opcode::SetupLoop),
            0x1D => Some(Opcode::PopBlock),
            0x1E => Some(Opcode::Await),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: u32,
}

#[derive(Debug, Clone)]
pub struct BytecodeFile {
    pub constants: Vec<Value>,
    pub names: Vec<String>,
    pub instructions: Vec<Instruction>,
}

impl BytecodeFile {
    pub fn load(data: &[u8]) -> Result<Self, String> {
        let mut cursor = 0;

        // Check magic number
        if data.len() < 6 || &data[0..4] != b"NAG\x00" {
            return Err("Invalid bytecode file: missing magic number".to_string());
        }
        cursor += 4;

        // Check version
        let version = u16::from_le_bytes([data[cursor], data[cursor + 1]]);
        if version != 1 {
            return Err(format!("Unsupported bytecode version: {version}"));
        }
        cursor += 2;

        // Load constants
        if cursor + 4 > data.len() {
            return Err("Invalid bytecode file: truncated constants section".to_string());
        }
        let constants_count = u32::from_le_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]) as usize;
        cursor += 4;

        let mut constants = Vec::with_capacity(constants_count);
        for _ in 0..constants_count {
            let (constant, bytes_read) = Self::load_constant(&data[cursor..])?;
            constants.push(constant);
            cursor += bytes_read;
        }

        // Load names
        if cursor + 4 > data.len() {
            return Err("Invalid bytecode file: truncated names section".to_string());
        }
        let names_count = u32::from_le_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]) as usize;
        cursor += 4;

        let mut names = Vec::with_capacity(names_count);
        for _ in 0..names_count {
            let (name, bytes_read) = Self::load_string(&data[cursor..])?;
            names.push(name);
            cursor += bytes_read;
        }

        // Load instructions
        if cursor + 4 > data.len() {
            return Err("Invalid bytecode file: truncated instructions section".to_string());
        }
        let instructions_count = u32::from_le_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]) as usize;
        cursor += 4;

        let mut instructions = Vec::with_capacity(instructions_count);
        for _ in 0..instructions_count {
            if cursor + 5 > data.len() {
                return Err("Invalid bytecode file: truncated instruction".to_string());
            }

            let opcode = Opcode::from_u8(data[cursor])
                .ok_or_else(|| format!("Unknown opcode: 0x{:02x}", data[cursor]))?;
            cursor += 1;

            let operand = u32::from_le_bytes([
                data[cursor],
                data[cursor + 1],
                data[cursor + 2],
                data[cursor + 3],
            ]);
            cursor += 4;

            instructions.push(Instruction { opcode, operand });
        }

        Ok(BytecodeFile {
            constants,
            names,
            instructions,
        })
    }

    fn load_constant(data: &[u8]) -> Result<(Value, usize), String> {
        if data.is_empty() {
            return Err("Invalid constant: empty data".to_string());
        }

        let type_tag = data[0];
        let mut cursor = 1;

        match type_tag {
            0 => {
                // Int
                if cursor + 8 > data.len() {
                    return Err("Invalid int constant: insufficient data".to_string());
                }
                let value = i64::from_le_bytes([
                    data[cursor],
                    data[cursor + 1],
                    data[cursor + 2],
                    data[cursor + 3],
                    data[cursor + 4],
                    data[cursor + 5],
                    data[cursor + 6],
                    data[cursor + 7],
                ]);
                cursor += 8;
                Ok((Value::Int(value), cursor))
            }
            1 => {
                // Float
                if cursor + 8 > data.len() {
                    return Err("Invalid float constant: insufficient data".to_string());
                }
                let value = f64::from_le_bytes([
                    data[cursor],
                    data[cursor + 1],
                    data[cursor + 2],
                    data[cursor + 3],
                    data[cursor + 4],
                    data[cursor + 5],
                    data[cursor + 6],
                    data[cursor + 7],
                ]);
                cursor += 8;
                Ok((Value::Float(value), cursor))
            }
            2 => {
                // String
                let (string, bytes_read) = Self::load_string(&data[cursor..])?;
                cursor += bytes_read;
                Ok((Value::String(string), cursor))
            }
            3 => {
                // Bool
                if cursor + 1 > data.len() {
                    return Err("Invalid bool constant: insufficient data".to_string());
                }
                let value = data[cursor] != 0;
                cursor += 1;
                Ok((Value::Bool(value), cursor))
            }
            4 => {
                // None
                Ok((Value::None, cursor))
            }
            _ => Err(format!("Unknown constant type tag: {type_tag}")),
        }
    }

    fn load_string(data: &[u8]) -> Result<(String, usize), String> {
        if data.len() < 4 {
            return Err("Invalid string: insufficient length data".to_string());
        }

        let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let cursor = 4;

        if cursor + length > data.len() {
            return Err("Invalid string: insufficient string data".to_string());
        }

        let string = String::from_utf8(data[cursor..cursor + length].to_vec())
            .map_err(|e| format!("Invalid UTF-8 string: {e}"))?;

        Ok((string, cursor + length))
    }
}
