use lemrvm_core::bus::Bus;
use lemrvm_core::hart::Hart;
use std::collections::HashMap;
use std::path::Path;

struct TestBus {
    mem: HashMap<u32, u8>,
}

impl TestBus {
    fn new() -> Self {
        Self {
            mem: HashMap::new(),
        }
    }

    fn load(&mut self, addr: u32, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            self.mem.insert(addr + i as u32, byte);
        }
    }
}

impl Bus for TestBus {
    fn read(&self, addr: u32) -> u32 {
        let bytes = [
            *self.mem.get(&addr).unwrap_or(&0),
            *self.mem.get(&(addr + 1)).unwrap_or(&0),
            *self.mem.get(&(addr + 2)).unwrap_or(&0),
            *self.mem.get(&(addr + 3)).unwrap_or(&0),
        ];
        u32::from_le_bytes(bytes)
    }

    fn write(&mut self, addr: u32, value: u32) {
        for (i, byte) in value.to_le_bytes().iter().enumerate() {
            self.mem.insert(addr + i as u32, *byte);
        }
    }
}

fn riscv_test(path: &Path) -> datatest_stable::Result<()> {
    let bytes = std::fs::read(path)?;
    let elf = goblin::elf::Elf::parse(&bytes)?;

    let mut bus = TestBus::new();
    for phdr in &elf.program_headers {
        if phdr.p_type == goblin::elf::program_header::PT_LOAD {
            let src = &bytes[phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize];
            let addr = phdr.p_paddr as u32;
            bus.load(addr, src);
        }
    }

    let mut hart = Hart::default();
    hart.set_pc(elf.entry as u32);

    loop {
        let instr = bus.read(hart.pc());
        if instr == 0x00000073 {
            break;
        }
        hart.step(&mut bus);
    }

    let gp = hart.reg(3);
    assert_eq!(gp, 1, "failed at test case {}", gp >> 1);

    Ok(())
}

datatest_stable::harness! {
    { test = riscv_test, root = "data/bins/riscv" },
}
