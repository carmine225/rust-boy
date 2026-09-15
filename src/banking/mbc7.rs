pub struct Mbc7 {
    pub current_rom_bank: u8,
}
impl Mbc7 {
    pub fn _mbc7(&mut self, value: u8) {
        todo!("MBC7 mapper not implemented")
    }
    pub fn _mbc7_write(&mut self, address: u16, value: u8) {
        todo!("mbc7 write");
    }
}
