use crate::bus::Bus;
use crate::instruction::Instruction;

/// A RISC-V Hardware Thread
#[derive(Debug, Default)]
pub struct Hart {
    /// Since x0 is hardwired to zero, we can assign pc to index 0
    regs: [u32; 32],
}

// Base functionality
impl Hart {
    pub fn step(&mut self, bus: &mut impl Bus) {
        let i = self.fetch(bus);
        if let Some(instr) = Instruction::decode(i) {
            self.execute(bus, instr);
        } else {
            panic!(
                "unimplemented instruction: {:#010X} at pc {:#010X}",
                i,
                self.pc()
            )
        }
    }

    fn fetch(&mut self, bus: &mut impl Bus) -> u32 {
        let addr = self.regs[0];
        let value = bus.read(addr);
        self.regs[0] += 4;
        value
    }

    fn execute(&mut self, bus: &mut impl Bus, instr: Instruction) {
        match instr {
            Instruction::Add(r) => {
                self.set_reg(r.rd, self.reg(r.rs1).wrapping_add(self.reg(r.rs2)));
            }
            Instruction::Sub(r) => {
                self.set_reg(r.rd, self.reg(r.rs1).wrapping_sub(self.reg(r.rs2)));
            }
            Instruction::Xor(r) => self.set_reg(r.rd, self.reg(r.rs1) ^ self.reg(r.rs2)),
            Instruction::Or(r) => self.set_reg(r.rd, self.reg(r.rs1) | self.reg(r.rs2)),
            Instruction::And(r) => self.set_reg(r.rd, self.reg(r.rs1) & self.reg(r.rs2)),
            Instruction::Sll(r) => self.set_reg(r.rd, self.reg(r.rs1) << (self.reg(r.rs2) & 0x1F)),
            Instruction::Srl(r) => self.set_reg(r.rd, self.reg(r.rs1) >> (self.reg(r.rs2) & 0x1F)),
            Instruction::Sra(r) => self.set_reg(
                r.rd,
                ((self.reg(r.rs1) as i32) >> (self.reg(r.rs2) & 0x1F)) as u32,
            ),
            Instruction::Slt(r) => self.set_reg(
                r.rd,
                ((self.reg(r.rs1) as i32) < (self.reg(r.rs2) as i32)) as u32,
            ),
            Instruction::Sltu(r) => self.set_reg(r.rd, (self.reg(r.rs1) < self.reg(r.rs2)) as u32),
        }
    }
}

// Registers
impl Hart {
    pub fn pc(&self) -> u32 {
        self.regs[0]
    }

    pub fn set_pc(&mut self, pc: u32) {
        self.regs[0] = pc;
    }

    /// Will panic if out of bounds
    pub fn reg(&self, index: usize) -> u32 {
        if index == 0 { 0 } else { self.regs[index] }
    }

    /// Will panic if out of bounds
    pub fn set_reg(&mut self, index: usize, value: u32) {
        if index != 0 {
            self.regs[index] = value;
        }
    }
}
