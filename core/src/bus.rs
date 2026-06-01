pub trait Bus {
    fn read(&self, addr: u32) -> u32;
    fn write(&mut self, addr: u32, data: u32);
}
