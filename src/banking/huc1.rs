pub struct Huc1 {
    pub current_rom_bank: u8,
    pub current_ram_bank: u8,
    ram_enabled: bool,
}

impl Huc1 {
    pub fn _huc1(&mut self, value: u8) {
        todo!("HUC1 mapper not implemented");
    }
    pub fn _huc1_write(&mut self, address: u16, value: u8) {
        todo!("huc1 write");
    }
}
