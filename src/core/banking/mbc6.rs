pub struct Mbc6 {
    pub current_rom_bank: u8,
}
impl Mbc6 {
    pub fn new() -> Self {
        Mbc6 {
            current_rom_bank: 1,
        }
    }
    pub fn read(&mut self, address: u16) -> u8 {
        todo!("MBC6 read");
    }
    pub fn write(&mut self, address: u16, value: u8) {
        todo!("mbc6 write");
    }
}
