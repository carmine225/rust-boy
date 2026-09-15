pub struct Huc3 {
    pub current_rom_bank: u8,
    pub current_ram_bank: u8,
    ram_enabled: bool,
}

impl Huc3 {
    pub fn _huc3(&mut self, value: u8) {
        todo!("HUC3 mapper not implemented");
    }
    pub fn _huc3_write(&mut self, address: u16, value: u8) {
        todo!("huc3 write");
    }
}
