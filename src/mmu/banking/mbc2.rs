use crate::mmu::Mmu;

impl Mmu {
    pub fn _mbc2(&mut self, value: u8) {
        self.mbc_type = value;
        self.current_rom_bank = 1;
        self.current_ram_bank = 0;
        self.ram_enabled = false;
        self.card_ram = vec![0; 512];
    }
    pub fn _mbc2_read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.card_rom[address as usize],
            0x4000..=0x7FFF => {
                let offset =
                    ((self.current_rom_bank as u16 * 0x4000) + (address - 0x4000)) as usize;
                self.card_rom[offset]
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    0xFF
                } else {
                    let i = (address as usize) & 0x01FF;
                    self.card_ram[i] | 0xF0
                }
            }
            _ => 0xFF,
        }
    }
    pub fn _mbc2_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => {
                if (address & 0x0100) == 0 {
                    // Bit 8 è 0: Abilitazione RAM
                    // Come MBC1, abilitata solo se i 4 bit bassi sono 0x0A
                    self.ram_enabled = (value & 0x0F) == 0x0A;
                } else {
                    // Bit 8 è 1: Selezione Banco ROM (4 bit)
                    let mut bank = (value & 0x0F) as usize;
                    if bank == 0 {
                        bank = 1; // La regola del banco 0 vale anche qui
                    }
                    self.current_rom_bank = bank as u8;
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let idx = (address as usize) & 0x01FF;
                    // Si salvano solo i 4 bit meno significativi
                    self.card_ram[idx] = value & 0x0F;
                }
            }
            _ => {}
        }
    }
}
