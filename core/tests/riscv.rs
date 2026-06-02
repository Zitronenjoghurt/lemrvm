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
    fn read8(&self, addr: u32) -> u8 {
        self.mem.get(&addr).copied().unwrap_or(0)
    }

    fn read16(&self, addr: u32) -> u16 {
        (self.read8(addr) as u16) | ((self.read8(addr + 1) as u16) << 8)
    }

    fn read32(&self, addr: u32) -> u32 {
        (self.read16(addr) as u32) | ((self.read16(addr + 2) as u32) << 16)
    }

    fn write8(&mut self, addr: u32, val: u8) {
        self.mem.insert(addr, val);
    }

    fn write16(&mut self, addr: u32, val: u16) {
        self.write8(addr, val as u8);
        self.write8(addr + 1, (val >> 8) as u8);
    }

    fn write32(&mut self, addr: u32, val: u32) {
        self.write16(addr, val as u16);
        self.write16(addr + 2, (val >> 16) as u16);
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

    let tohost_addr = elf
        .syms
        .iter()
        .find(|sym| elf.strtab.get_at(sym.st_name) == Some("tohost"))
        .map(|sym| sym.st_value as u32)
        .expect("tohost symbol not found");

    let mut hart = Hart::default();
    hart.set_pc(elf.entry as u32);

    loop {
        hart.step(&mut bus);
        let tohost = bus.read32(tohost_addr);
        if tohost != 0 {
            break;
        }
    }
    assert_eq!(
        bus.read32(tohost_addr),
        1,
        "failed at test case {}",
        bus.read32(tohost_addr) >> 1
    );

    let gp = hart.reg(3);
    assert_eq!(gp, 1, "failed at test case {}", gp >> 1);

    Ok(())
}

datatest_stable::harness! {
    { test = riscv_test, root = "data/bins/riscv" },
}
