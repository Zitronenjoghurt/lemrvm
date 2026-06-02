use crate::bus::Bus;
use crate::instruction::Instruction::{self, *};
use crate::privilege::PrivilegeMode;
use crate::ubits;

// Machine-level CSRs
pub const MSTATUS: usize = 0x300;
pub const MTVEC: usize = 0x305;
pub const MEPC: usize = 0x341;
pub const MCAUSE: usize = 0x342;
pub const MTVAL: usize = 0x343;

// mstatus bit positions
const MSTATUS_MPP_SHIFT: u32 = 11;
const MSTATUS_MPP_MASK: u32 = 0x3 << MSTATUS_MPP_SHIFT;

// Trap causes
const CAUSE_ILLEGAL_INSTRUCTION: u32 = 2;
const CAUSE_BREAKPOINT: u32 = 3;
const CAUSE_ECALL_U: u32 = 8;
const CAUSE_ECALL_S: u32 = 9;
const CAUSE_ECALL_M: u32 = 11;

/// A RISC-V Hardware Thread
#[derive(Debug)]
pub struct Hart {
    /// Since x0 is hardwired to zero, we can assign pc to index 0
    regs: [u32; 32],
    /// Control and Status Registers
    csrs: [u32; 4096],
    mode: PrivilegeMode,
}

impl Default for Hart {
    fn default() -> Self {
        Self {
            regs: [0; 32],
            csrs: [0; 4096],
            mode: PrivilegeMode::Machine,
        }
    }
}

// Base functionality
impl Hart {
    pub fn step(&mut self, bus: &mut impl Bus) {
        let i = self.fetch(bus);
        if let Some(instr) = Instruction::decode(i) {
            self.execute(bus, instr);
        } else {
            self.trap(CAUSE_ILLEGAL_INSTRUCTION, i);
        }
    }

    fn fetch(&mut self, bus: &mut impl Bus) -> u32 {
        let addr = self.regs[0];
        let value = bus.read32(addr);
        self.regs[0] += 4;
        value
    }

    fn trap(&mut self, cause: u32, tval: u32) {
        self.set_csr(MEPC, self.pc().wrapping_sub(4));
        self.set_csr(MCAUSE, cause);
        self.set_csr(MTVAL, tval);

        let mstatus =
            (self.csr(MSTATUS) & !MSTATUS_MPP_MASK) | ((self.mode as u32) << MSTATUS_MPP_SHIFT);
        self.set_csr(MSTATUS, mstatus);

        self.mode = PrivilegeMode::Machine;
        self.set_pc(self.csr(MTVEC) & !0x3); // ToDo: Vectored mode? (assuming direct mode rn)
    }

    fn execute(&mut self, bus: &mut impl Bus, instr: Instruction) {
        match instr {
            Illegal(i) => {
                self.trap(CAUSE_ILLEGAL_INSTRUCTION, i);
            }
            Add(r) => {
                self.set_reg(r.rd, self.reg(r.rs1).wrapping_add(self.reg(r.rs2)));
            }
            Sub(r) => {
                self.set_reg(r.rd, self.reg(r.rs1).wrapping_sub(self.reg(r.rs2)));
            }
            Xor(r) => self.set_reg(r.rd, self.reg(r.rs1) ^ self.reg(r.rs2)),
            Or(r) => self.set_reg(r.rd, self.reg(r.rs1) | self.reg(r.rs2)),
            And(r) => self.set_reg(r.rd, self.reg(r.rs1) & self.reg(r.rs2)),
            Sll(r) => self.set_reg(r.rd, self.reg(r.rs1) << (self.reg(r.rs2) & 0x1F)),
            Srl(r) => self.set_reg(r.rd, self.reg(r.rs1) >> (self.reg(r.rs2) & 0x1F)),
            Sra(r) => self.set_reg(
                r.rd,
                ((self.reg(r.rs1) as i32) >> (self.reg(r.rs2) & 0x1F)) as u32,
            ),
            Slt(r) => self.set_reg(
                r.rd,
                ((self.reg(r.rs1) as i32) < (self.reg(r.rs2) as i32)) as u32,
            ),
            Sltu(r) => self.set_reg(r.rd, (self.reg(r.rs1) < self.reg(r.rs2)) as u32),
            Addi(i) => {
                self.set_reg(i.rd, (self.reg(i.rs1) as i32).wrapping_add(i.imm) as u32);
            }
            Xori(i) => self.set_reg(i.rd, (self.reg(i.rs1) as i32 ^ i.imm) as u32),
            Ori(i) => self.set_reg(i.rd, (self.reg(i.rs1) as i32 | i.imm) as u32),
            Andi(i) => self.set_reg(i.rd, (self.reg(i.rs1) as i32 & i.imm) as u32),
            Slli(i) => self.set_reg(i.rd, self.reg(i.rs1) << ubits!(i.imm, 4:0)),
            Srli(i) => self.set_reg(i.rd, self.reg(i.rs1) >> ubits!(i.imm, 4:0)),
            Srai(i) => {
                self.set_reg(
                    i.rd,
                    ((self.reg(i.rs1) as i32) >> ubits!(i.imm, 4:0)) as u32,
                );
            }
            Slti(i) => {
                self.set_reg(i.rd, ((self.reg(i.rs1) as i32) < i.imm) as u32);
            }
            Sltiu(i) => {
                self.set_reg(i.rd, (self.reg(i.rs1) < (i.imm as u32)) as u32);
            }
            Lb(i) => {
                let a = (self.reg(i.rs1) as i32).wrapping_add(i.imm) as u32;
                self.set_reg(i.rd, bus.read8(a) as i8 as u32);
            }
            Lh(i) => {
                let a = (self.reg(i.rs1) as i32).wrapping_add(i.imm) as u32;
                self.set_reg(i.rd, bus.read16(a) as i16 as u32);
            }
            Lw(i) => {
                let a = (self.reg(i.rs1) as i32).wrapping_add(i.imm) as u32;
                self.set_reg(i.rd, bus.read32(a));
            }
            Lbu(i) => {
                let a = (self.reg(i.rs1) as i32).wrapping_add(i.imm) as u32;
                self.set_reg(i.rd, bus.read8(a) as u32);
            }
            Lhu(i) => {
                let a = (self.reg(i.rs1) as i32).wrapping_add(i.imm) as u32;
                self.set_reg(i.rd, bus.read16(a) as u32);
            }
            Sb(s) => {
                let a = (self.reg(s.rs1) as i32).wrapping_add(s.imm) as u32;
                bus.write8(a, self.reg(s.rs2) as u8);
            }
            Sh(s) => {
                let a = (self.reg(s.rs1) as i32).wrapping_add(s.imm) as u32;
                bus.write16(a, self.reg(s.rs2) as u16);
            }
            Sw(s) => {
                let a = (self.reg(s.rs1) as i32).wrapping_add(s.imm) as u32;
                bus.write32(a, self.reg(s.rs2));
            }
            Ecall(_) => {
                let cause = match self.mode {
                    PrivilegeMode::User => CAUSE_ECALL_U,
                    PrivilegeMode::Supervisor => CAUSE_ECALL_S,
                    PrivilegeMode::Machine => CAUSE_ECALL_M,
                };
                self.trap(cause, 0);
            }
            Ebreak(_) => {
                self.trap(CAUSE_BREAKPOINT, self.pc().wrapping_sub(4));
            }
            Uret(_) => {}
            Sret(_) => {
                // restore privilege from sstatus.SPP, jump to sepc
                // TODO: implement if needed for supervisor tests
            }
            Mret(_) => {
                let mstatus = self.csr(MSTATUS);
                let mpp = (mstatus & MSTATUS_MPP_MASK) >> MSTATUS_MPP_SHIFT;
                self.mode = PrivilegeMode::from_bits(mpp);

                let mstatus = mstatus & !MSTATUS_MPP_MASK;
                self.set_csr(MSTATUS, mstatus);

                self.set_pc(self.csr(MEPC));
            }
            Wfi(_) => {}
            Csrrw(i) => {
                let addr = (i.imm as u32 & 0xFFF) as usize;
                let old = self.csr(addr);
                self.set_csr(addr, self.reg(i.rs1));
                self.set_reg(i.rd, old);
            }
            Csrrs(i) => {
                let addr = (i.imm as u32 & 0xFFF) as usize;
                let old = self.csr(addr);
                if i.rs1 != 0 {
                    self.set_csr(addr, old | self.reg(i.rs1));
                }
                self.set_reg(i.rd, old);
            }
            Csrrc(i) => {
                let addr = (i.imm as u32 & 0xFFF) as usize;
                let old = self.csr(addr);
                if i.rs1 != 0 {
                    self.set_csr(addr, old & !self.reg(i.rs1));
                }
                self.set_reg(i.rd, old);
            }
            Csrrwi(i) => {
                let addr = (i.imm as u32 & 0xFFF) as usize;
                let old = self.csr(addr);
                self.set_csr(addr, i.rs1 as u32);
                self.set_reg(i.rd, old);
            }
            Csrrsi(i) => {
                let addr = (i.imm as u32 & 0xFFF) as usize;
                let old = self.csr(addr);
                if i.rs1 != 0 {
                    self.set_csr(addr, old | i.rs1 as u32);
                }
                self.set_reg(i.rd, old);
            }
            Csrrci(i) => {
                let addr = (i.imm as u32 & 0xFFF) as usize;
                let old = self.csr(addr);
                if i.rs1 != 0 {
                    self.set_csr(addr, old & !(i.rs1 as u32));
                }
                self.set_reg(i.rd, old);
            }
            Fence(_) => {}
            Beq(b) => {
                if self.reg(b.rs1) == self.reg(b.rs2) {
                    self.set_pc((self.pc() as i32).wrapping_sub(4).wrapping_add(b.imm) as u32);
                }
            }
            Bne(b) => {
                if self.reg(b.rs1) != self.reg(b.rs2) {
                    self.set_pc((self.pc() as i32).wrapping_sub(4).wrapping_add(b.imm) as u32);
                }
            }
            Blt(b) => {
                if (self.reg(b.rs1) as i32) < (self.reg(b.rs2) as i32) {
                    self.set_pc((self.pc() as i32).wrapping_sub(4).wrapping_add(b.imm) as u32);
                }
            }
            Bge(b) => {
                if (self.reg(b.rs1) as i32) >= (self.reg(b.rs2) as i32) {
                    self.set_pc((self.pc() as i32).wrapping_sub(4).wrapping_add(b.imm) as u32);
                }
            }
            Bltu(b) => {
                if self.reg(b.rs1) < (self.reg(b.rs2)) {
                    self.set_pc(self.pc().wrapping_sub(4).wrapping_add(b.imm as u32));
                }
            }
            Bgeu(b) => {
                if self.reg(b.rs1) >= (self.reg(b.rs2)) {
                    self.set_pc(self.pc().wrapping_sub(4).wrapping_add(b.imm as u32));
                }
            }
            Lui(u) => self.set_reg(u.rd, u.imm as u32),
            Auipc(a) => self.set_reg(a.rd, self.pc().wrapping_sub(4).wrapping_add(a.imm as u32)),
            Jal(j) => {
                self.set_reg(j.rd, self.pc());
                self.set_pc(self.pc().wrapping_sub(4).wrapping_add(j.imm as u32));
            }
            Jalr(i) => {
                let t = ((self.reg(i.rs1) as i32).wrapping_add(i.imm) as u32) & !1;
                self.set_reg(i.rd, self.pc());
                self.set_pc(t);
            }
        }
    }
}

// Registers
impl Hart {
    #[inline]
    pub fn pc(&self) -> u32 {
        self.regs[0]
    }

    #[inline]
    pub fn set_pc(&mut self, pc: u32) {
        self.regs[0] = pc;
    }

    /// Will panic if out of bounds
    #[inline]
    pub fn reg(&self, index: usize) -> u32 {
        if index == 0 { 0 } else { self.regs[index] }
    }

    /// Will panic if out of bounds
    #[inline]
    pub fn set_reg(&mut self, index: usize, value: u32) {
        if index != 0 {
            self.regs[index] = value;
        }
    }

    /// Will panic if out of bounds
    #[inline]
    pub fn csr(&self, csr: usize) -> u32 {
        self.csrs[csr]
    }

    /// Will panic if out of bounds
    #[inline]
    pub fn set_csr(&mut self, csr: usize, value: u32) {
        self.csrs[csr] = value;
    }
}
