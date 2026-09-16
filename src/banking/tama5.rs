//mapper for specific games
pub struct Tama5 {
    pub current_rom_bank: u8,
}
impl Tama5 {
    pub fn new() -> Self {
        Tama5 {
            current_rom_bank: 1,
        }
    }
    pub fn read(&mut self, address: u16) -> u8 {
        todo!("tama5 read");
    }
    pub fn write(&mut self, address: u16, value: u8) {
        todo!("tama5 write");
    }
}
