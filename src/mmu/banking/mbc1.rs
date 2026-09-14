use crate::mmu::Mmu;

impl Mmu {
    pub fn _mbc1(&mut self, value: u8) {
        self.mbc_type = value;
        self.current_rom_bank = 1;
        self.current_ram_bank = 0;
        self.ram_enabled = false;
        self.banking_mode = 0; // Se hai la variabile nello struct Mmu
    }
    pub fn _mbc1_read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => self.card_rom[address as usize],
            0x4000..=0x7fff => {
                self.card_rom
                    [((self.current_rom_bank as u16 * 0x4000) + (address - 0x4000)) as usize]
            }
            0xA000..=0xBFFF => {
                // Se la RAM non è abilitata o non è presente sulla cartuccia, restituisce 0xFF
                if !self.ram_enabled || self.card_ram.is_empty() {
                    0xFF
                } else {
                    let offset =
                        ((self.current_ram_bank as u16 * 0x4000) + (address - 0x4000)) as usize;
                    if offset < self.card_ram.len() {
                        self.card_ram[offset]
                    } else {
                        0xFF
                    }
                }
            }
            _ => 0xff,
        }
    }
    pub fn _mbc1_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3fff => {
                let mut bank = (value & 0x1F) as usize;
                if bank == 0 {
                    bank = 1;
                }
                self.current_rom_bank = ((self.current_rom_bank as usize & 0x60) | bank) as u8;
            }
            0x4000..=0x5FFF => {
                let bits = (value & 0x03) as usize;
                if self.banking_mode == 0 {
                    self.current_rom_bank =
                        ((self.current_rom_bank as usize & 0x1F) | (bits << 5)) as u8;
                    self.current_ram_bank = 0;
                } else {
                    self.current_ram_bank = bits as u8;
                }
            }
            0x6000..=0x7FFF => {
                // Si prende solo il primo bit (0 oppure 1)
                self.banking_mode = value & 0x01;

                // Nota hardware: in Mode 0, la RAM torna sempre a puntare al Banco 0
                if self.banking_mode == 0 {
                    self.current_ram_bank = 0;
                }
            }
            _ => {}
        }
    }
}
