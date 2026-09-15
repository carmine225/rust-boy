//mapper for specific games
pub struct Tama5 {
    pub current_rom_bank: u8,
}
impl Tama5 {
    pub fn _tama5(&mut self, value: u8) {
        todo!("TAMA5 mapper not implemented");
    }
    pub fn _tama5_write(&mut self, address: u16, value: u8) {
        todo!("huc1 write");
    }
}
