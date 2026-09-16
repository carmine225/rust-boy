pub struct Mbc7 {
    pub current_rom_bank: u8,
}
impl Mbc7 {
    pub fn new() -> Self {
        Mbc7 {
            current_rom_bank: 1,
        }
    }
    pub fn read(&mut self, address: u16) -> u8 {
        todo!("MBC7 read");
    }
    pub fn write(&mut self, address: u16, value: u8) {
        todo!("mbc7 write");
    }
}
