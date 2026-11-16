pub struct Mmm01 {
    pub current_rom_bank: u8,
}
impl Mmm01 {
    pub fn new() -> Self {
        Mmm01 {
            current_rom_bank: 1,
        }
    }
    pub fn read(&mut self, address: u16) -> u8 {
        todo!("MMM01 read");
    }
    pub fn write(&mut self, address: u16, value: u8) {
        todo!("mbc5 write");
    }
}
