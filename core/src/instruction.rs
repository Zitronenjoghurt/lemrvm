use crate::{gather, ibits, sext, ubits};

#[inline]
pub fn decode_signature(i: u32) -> (u32, u32, u32) {
    (ubits!(i, 6:0), ubits!(i, 14:12), ubits!(i, 31:25))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instruction {
    Illegal(u32),
    // RV32I
    // R-Type
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
    // I-Type
    Addi(IType),
    Xori(IType),
    Ori(IType),
    Andi(IType),
    Slli(IType),
    Srli(IType),
    Srai(IType),
    Slti(IType),
    Sltiu(IType),
    Lb(IType),
    Lh(IType),
    Lw(IType),
    Lbu(IType),
    Lhu(IType),
    Sb(SType),
    Sh(SType),
    Sw(SType),
    Ecall(IType),
    Ebreak(IType),
    Uret(IType),
    Sret(IType),
    Mret(IType),
    Wfi(IType),
    Csrrw(IType),
    Csrrs(IType),
    Csrrc(IType),
    Csrrwi(IType),
    Csrrci(IType),
    Csrrsi(IType),
    Fence(IType),
    // S-Type
    // B-Type
    Beq(BType),
    Bne(BType),
    Blt(BType),
    Bge(BType),
    Bltu(BType),
    Bgeu(BType),
    // U-Type
    Lui(UType),
    Auipc(UType),
    // J-Type
    Jal(JType),
    Jalr(IType),
}

impl Instruction {
    pub fn decode(i: u32) -> Option<Self> {
        if i == 0 || i == 0xFFFFFFF {
            return Some(Self::Illegal(i));
        }

        let (op, f3, f7) = decode_signature(i);

        let instr = match op {
            0x03 => match f3 {
                0x0 => Self::Lb(IType::decode(i)),
                0x1 => Self::Lh(IType::decode(i)),
                0x2 => Self::Lw(IType::decode(i)),
                0x4 => Self::Lbu(IType::decode(i)),
                0x5 => Self::Lhu(IType::decode(i)),
                _ => return None,
            },
            0x0F => Self::Fence(IType::decode(i)),
            0x13 => match f3 {
                0x0 => Self::Addi(IType::decode(i)),
                0x1 => Self::Slli(IType::decode(i)), // ToDo: Check if imm condition is real (imm[11:5] == 0x00?)
                0x2 => Self::Slti(IType::decode(i)),
                0x3 => Self::Sltiu(IType::decode(i)),
                0x4 => Self::Xori(IType::decode(i)),
                0x5 => {
                    let it = IType::decode(i);
                    let c = ibits!(it.imm, 11:5);
                    if c == 0x00 {
                        Self::Srli(it)
                    } else if c == 0x20 {
                        Self::Srai(it)
                    } else {
                        return None;
                    }
                }
                0x6 => Self::Ori(IType::decode(i)),
                0x7 => Self::Andi(IType::decode(i)),
                _ => return None,
            },
            0x17 => Self::Auipc(UType::decode(i)),
            0x23 => match f3 {
                0x0 => Self::Sb(SType::decode(i)),
                0x1 => Self::Sh(SType::decode(i)),
                0x2 => Self::Sw(SType::decode(i)),
                _ => return None,
            },
            0x33 => match (f3, f7) {
                (0x0, 0x00) => Self::Add(RType::decode(i)),
                (0x0, 0x20) => Self::Sub(RType::decode(i)),
                (0x1, 0x00) => Self::Sll(RType::decode(i)),
                (0x2, 0x00) => Self::Slt(RType::decode(i)),
                (0x3, 0x00) => Self::Sltu(RType::decode(i)),
                (0x4, 0x00) => Self::Xor(RType::decode(i)),
                (0x5, 0x00) => Self::Srl(RType::decode(i)),
                (0x5, 0x20) => Self::Sra(RType::decode(i)),
                (0x6, 0x00) => Self::Or(RType::decode(i)),
                (0x7, 0x00) => Self::And(RType::decode(i)),
                _ => return None,
            },
            0x37 => Self::Lui(UType::decode(i)),
            0x63 => match f3 {
                0x0 => Self::Beq(BType::decode(i)),
                0x1 => Self::Bne(BType::decode(i)),
                0x4 => Self::Blt(BType::decode(i)),
                0x5 => Self::Bge(BType::decode(i)),
                0x6 => Self::Bltu(BType::decode(i)),
                0x7 => Self::Bgeu(BType::decode(i)),
                _ => return None,
            },
            0x67 => match f3 {
                0x0 => Self::Jalr(IType::decode(i)),
                _ => {
                    return None;
                }
            },
            0x6F => Self::Jal(JType::decode(i)),
            0x73 => match f3 {
                0x0 => {
                    let it = IType::decode(i);
                    let c = it.imm as u32;
                    match c {
                        0x000 => Self::Ecall(it),
                        0x001 => Self::Ebreak(it),
                        0x002 => Self::Uret(it),
                        0x102 => Self::Sret(it),
                        0x105 => Self::Wfi(it),
                        0x302 => Self::Mret(it),
                        _ => return None,
                    }
                }
                0x1 => Self::Csrrw(IType::decode(i)),
                0x2 => Self::Csrrs(IType::decode(i)),
                0x3 => Self::Csrrc(IType::decode(i)),
                0x5 => Self::Csrrwi(IType::decode(i)),
                0x6 => Self::Csrrci(IType::decode(i)),
                0x7 => Self::Csrrsi(IType::decode(i)),
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
    pub rd: usize,
    pub rs1: usize,
    pub imm: i32,
}

impl IType {
    pub fn decode(i: u32) -> Self {
        Self {
            rd: ubits!(i, 11:7) as usize,
            rs1: ubits!(i, 19:15) as usize,
            imm: ibits!(i, 31:20),
        }
    }
}

/// Store Type Instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SType {
    pub rs1: usize,
    pub rs2: usize,
    pub imm: i32,
}

impl SType {
    pub fn decode(i: u32) -> Self {
        let imm = gather!(i,
            31:25 => 5,
            11:7  => 0,
        );
        Self {
            rs1: ubits!(i, 19:15) as usize,
            rs2: ubits!(i, 24:20) as usize,
            imm: sext!(imm, 12),
        }
    }
}

/// Branch Type Instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BType {
    pub rs1: usize,
    pub rs2: usize,
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
            rs1: ubits!(i, 19:15) as usize,
            rs2: ubits!(i, 24:20) as usize,
            imm: sext!(imm, 13),
        }
    }
}

/// Upper Immediate Type Instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UType {
    pub rd: usize,
    pub imm: i32,
}

impl UType {
    pub fn decode(i: u32) -> Self {
        Self {
            rd: ubits!(i, 11:7) as usize,
            imm: (i & 0xFFFFF000) as i32,
        }
    }
}

/// Jump Type Instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct JType {
    pub rd: usize,
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
            rd: ubits!(i, 11:7) as usize,
            imm: sext!(imm, 21),
        }
    }
}
