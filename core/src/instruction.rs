use crate::{gather, ibits, sext, ubits};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instruction {
    // RV32I: R-Type
    Add(RType),
    Sub(RType),
    Xor(RType),
    Or(RType),
    And(RType),
    Sll(RType),
    Srl(RType),
    Sra(RType),
    Slt(RType),
    Sltu(RType),
}

impl Instruction {
    pub fn decode(i: u32) -> Option<Self> {
        let opcode = ubits!(i, 6:0);
        let funct3 = ubits!(i, 14:12);
        let funct7 = ubits!(i, 31:25);

        let instr = match opcode {
            0x33 => match (funct3, funct7) {
                (0x0, 0x00) => Self::Add(RType::decode(i)),
                (0x0, 0x20) => Self::Sub(RType::decode(i)),
                (0x4, 0x00) => Self::Xor(RType::decode(i)),
                (0x6, 0x00) => Self::Or(RType::decode(i)),
                (0x7, 0x00) => Self::And(RType::decode(i)),
                (0x1, 0x00) => Self::Sll(RType::decode(i)),
                (0x5, 0x00) => Self::Srl(RType::decode(i)),
                (0x5, 0x20) => Self::Sra(RType::decode(i)),
                (0x2, 0x00) => Self::Slt(RType::decode(i)),
                (0x3, 0x00) => Self::Sltu(RType::decode(i)),
                _ => return None,
            },
            _ => return None,
        };

        Some(instr)
    }
}

/// Register Type Instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RType {
    pub rd: usize,
    pub rs1: usize,
    pub rs2: usize,
}

impl RType {
    pub fn decode(i: u32) -> Self {
        Self {
            rd: ubits!(i, 11:7) as usize,
            rs1: ubits!(i, 19:15) as usize,
            rs2: ubits!(i, 24:20) as usize,
        }
    }
}

/// Immediate Type Instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IType {
    pub rd: u8,
    pub rs1: u8,
    pub imm: i32,
}

impl IType {
    pub fn decode(i: u32) -> Self {
        Self {
            rd: ubits!(i, 11:7) as u8,
            rs1: ubits!(i, 19:15) as u8,
            imm: ibits!(i, 31:20),
        }
    }
}

/// Store Type Instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SType {
    pub rs1: u8,
    pub rs2: u8,
    pub imm: i32,
}

impl SType {
    pub fn decode(i: u32) -> Self {
        let imm = gather!(i,
            31:25 => 5,
            11:7  => 0,
        );
        Self {
            rs1: ubits!(i, 19:15) as u8,
            rs2: ubits!(i, 24:20) as u8,
            imm: sext!(imm, 12),
        }
    }
}

/// Branch Type Instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BType {
    pub rs1: u8,
    pub rs2: u8,
    pub imm: i32,
}

impl BType {
    pub fn decode(i: u32) -> Self {
        let imm = gather!(i,
            31:31 => 12,
            7:7   => 11,
            30:25 => 5,
            11:8  => 1,
        );
        Self {
            rs1: ubits!(i, 19:15) as u8,
            rs2: ubits!(i, 24:20) as u8,
            imm: sext!(imm, 13),
        }
    }
}

/// Upper Immediate Type Instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UType {
    pub rd: u8,
    pub imm: i32,
}

impl UType {
    pub fn decode(i: u32) -> Self {
        Self {
            rd: ubits!(i, 11:7) as u8,
            imm: (i & 0xFFFFF000) as i32,
        }
    }
}

/// Jump Type Instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct JType {
    pub rd: u8,
    pub imm: i32,
}

impl JType {
    pub fn decode(i: u32) -> Self {
        let imm = gather!(i,
            31:31 => 20,
            19:12 => 12,
            20:20 => 11,
            30:21 => 1,
        );
        Self {
            rd: ubits!(i, 11:7) as u8,
            imm: sext!(imm, 21),
        }
    }
}
