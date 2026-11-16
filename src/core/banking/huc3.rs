pub struct Huc3 {
    pub current_rom_bank: u8,
    pub current_ram_bank: u8,
    ram_enabled: bool,
}

impl Huc3 {
    pub fn new() -> Self {
        Huc3 {
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: false,
        }
    }
    pub fn read(&mut self, address: u16) -> u8 {
        todo!("HUC3 read");
    }
    pub fn write(&mut self, address: u16, value: u8) {
        todo!("HUC3 write");
    }
}
