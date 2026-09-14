use crate::mmu::Mmu;
impl Mmu {
    pub fn _rom_only(&mut self, value: u8) {
        self.mbc_type = value;
        self.current_rom_bank = 1;
        self.current_ram_bank = 0;
        self.ram_enabled = value == 0x08 || value == 0x09;
    }
    pub fn _rom_only_read(&mut self, address: u16) -> u8 {
        if address < self.card_rom.len() as u16 {
            self.card_rom[address as usize]
        } else {
            0xFF
        }
    }
    pub fn _rom_only_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => {}
            0xA000..=0xBFFF => {
                if address < self.card_rom.len() as u16 {
                    self.card_ram[address as usize] = value;
                }
            }
            _ => {}
        }
    }
}
