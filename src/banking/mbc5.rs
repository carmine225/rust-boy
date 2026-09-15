pub struct Mbc5 {
    pub current_rom_bank: u8,
}
impl Mbc5 {
    pub fn _mbc5(&mut self, value: u8) {
        todo!("MBC5 mapper not implemented");
    }
    pub fn _mbc5_write(&mut self, address: u16, value: u8) {
        todo!("mbc5 write");
    }
}
