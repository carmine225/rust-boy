pub struct Mbc5 {
    pub current_rom_bank: u8,
}
impl Mbc5 {
    pub fn new() -> Self {
        Mbc5 {
            current_rom_bank: 1,
        }
    }
    pub fn read(&mut self, address: u16) -> u8 {
        todo!("MBC5 mapper not implemented");
    }
    pub fn write(&mut self, address: u16, value: u8) {
        todo!("mbc5 write");
    }
}
