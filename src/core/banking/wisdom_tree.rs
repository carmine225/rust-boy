pub struct WisdomTree {
    pub current_rom_bank: u8,
}
impl WisdomTree {
    pub fn new() -> Self {
        WisdomTree {
            current_rom_bank: 1,
        }
    }
    pub fn read(&mut self, address: u16) -> u8 {
        todo!("wisdom tree mapper read");
    }
    pub fn write(&mut self, address: u16, value: u8) {
        todo!("wisdom tree mapper write");
    }
}
