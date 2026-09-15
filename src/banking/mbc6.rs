pub struct Mbc6 {
    pub current_rom_bank: u8,
}
impl Mbc6 {
    pub fn _mbc6(&mut self, value: u8) {
        todo!("MBC6 mapper not implemented");
    }
    pub fn _mbc6_write(&mut self, address: u16, value: u8) {
        todo!("mbc6 write");
    }
}
