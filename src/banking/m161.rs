pub struct M161 {
    pub current_rom_bank: u8,
    pub current_ram_bank: u8,
    ram_enabled: bool,
}
impl M161 {
    pub fn new() -> Self {
        M161 {
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: false,
        }
    }
    pub fn read(&mut self, address: u16) -> u8 {
        todo!("m161 read")
    }
    pub fn write(&mut self, address: u16, value: u8) {
        todo!("m161 write");
    }
}
