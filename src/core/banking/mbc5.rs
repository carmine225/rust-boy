use crate::banking::Banking;

pub struct Mbc5 {
    pub current_rom_bank: u16,
    pub current_ram_bank: u8,
    ram_enabled: bool,
}
impl Mbc5 {
    pub fn new() -> Self {
        Mbc5 {
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: false,
        }
    }
    pub fn read(&self, banking: &mut Banking, address: u16) -> u8 {
        match address {
            // Banco ROM 0 fisso
            0x0000..=0x3FFF => banking.card_rom[address as usize],

            // Banco ROM Switchabile (0..=511)
            0x4000..=0x7FFF => {
                let offset =
                    (self.current_rom_bank as usize * 0x4000) + ((address - 0x4000) as usize);
                banking.card_rom.get(offset).copied().unwrap_or(0xFF)
            }

            // RAM Esterna (0..=15 banchi da 8 KiB)
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }
                let offset =
                    (self.current_ram_bank as usize * 0x2000) + ((address - 0xA000) as usize);
                banking.card_ram.get(offset).copied().unwrap_or(0xFF)
            }

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, banking: &mut Banking, address: u16, value: u8) {
        match address {
            // Abilitazione RAM (0x0A abilita, qualsiasi altro valore disabilita)
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }

            // ROM Bank Select: 8 bit inferiori (Bit 0-7)
            0x2000..=0x2FFF => {
                let current = self.current_rom_bank as usize;
                // Mantiene il 9° bit superiore e aggiorna gli 8 bit inferiori
                self.current_rom_bank = ((current & 0x0100) | (value as usize)) as u16;
            }

            // ROM Bank Select: 9° bit (Bit 8 / MSB)
            0x3000..=0x3FFF => {
                let current = self.current_rom_bank as usize;
                let bit_8 = ((value & 0x01) as usize) << 8;
                // Mantiene gli 8 bit inferiori e aggiorna il 9° bit
                self.current_rom_bank = ((current & 0x00FF) | bit_8) as u16;
            }

            // RAM Bank Select (4 bit: 0..=15)
            0x4000..=0x5FFF => {
                self.current_ram_bank = value & 0x0F;
            }

            // Scrittura in RAM Esterna
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return;
                }
                let offset =
                    (self.current_ram_bank as usize * 0x2000) + ((address - 0xA000) as usize);
                if offset < banking.card_ram.len() {
                    banking.card_ram[offset] = value;
                }
            }

            _ => {}
        }
    }
}
