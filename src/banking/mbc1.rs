use crate::banking::Banking;

pub struct Mbc1 {
    pub current_rom_bank: u8,
}
impl Mbc1 {
    pub fn new() -> Self {
        Mbc1 {
            current_rom_bank: 1,
        }
    }
    pub fn read(&mut self, banking: &mut Banking, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => {
                let bank = if banking.banking_mode == 1 {
                    (self.current_rom_bank & 0x60) as usize
                } else {
                    0
                };
                banking
                    .card_rom
                    .get(bank * 0x4000 + address as usize)
                    .copied()
                    .unwrap_or(0xFF)
            }
            0x4000..=0x7fff => {
                let bank = if banking.banking_mode == 1 {
                    self.current_rom_bank & 0x1F
                } else {
                    self.current_rom_bank
                };
                let bank = if bank == 0 { 1 } else { bank } as usize;
                banking
                    .card_rom
                    .get(bank * 0x4000 + (address - 0x4000) as usize)
                    .copied()
                    .unwrap_or(0xFF)
            }
            0xA000..=0xBFFF => {
                // Se la RAM non è abilitata o non è presente sulla cartuccia, restituisce 0xFF
                if !banking.ram_enabled || banking.card_ram.is_empty() {
                    0xFF
                } else {
                    let offset =
                        banking.current_ram_bank as usize * 0x2000 + (address - 0xA000) as usize;
                    if offset < banking.card_ram.len() {
                        banking.card_ram[offset]
                    } else {
                        0xFF
                    }
                }
            }
            _ => 0xff,
        }
    }
    pub fn write(&mut self, banking: &mut Banking, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => {
                banking.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3fff => {
                let mut bank = (value & 0x1F) as usize;
                if bank == 0 {
                    bank = 1;
                }
                let high_bits = if banking.banking_mode == 0 {
                    self.current_rom_bank as usize & 0x60
                } else {
                    0
                };
                self.current_rom_bank = (high_bits | bank) as u8;
            }
            0x4000..=0x5FFF => {
                let bits = (value & 0x03) as usize;
                if banking.banking_mode == 0 {
                    self.current_rom_bank =
                        ((self.current_rom_bank as usize & 0x1F) | (bits << 5)) as u8;
                    banking.current_ram_bank = 0;
                } else {
                    self.current_rom_bank =
                        ((self.current_rom_bank as usize & 0x1F) | (bits << 5)) as u8;
                    banking.current_ram_bank = bits as u8;
                }
            }
            0x6000..=0x7FFF => {
                // Si prende solo il primo bit (0 oppure 1)
                banking.banking_mode = value & 0x01;

                // Nota hardware: in Mode 0, la RAM torna sempre a puntare al Banco 0
                if banking.banking_mode == 0 {
                    banking.current_ram_bank = 0;
                }
            }
            0xA000..=0xBFFF => {
                if banking.ram_enabled && !banking.card_ram.is_empty() {
                    let offset =
                        banking.current_ram_bank as usize * 0x2000 + (address - 0xA000) as usize;
                    if let Some(byte) = banking.card_ram.get_mut(offset) {
                        *byte = value;
                    }
                }
            }
            _ => {}
        }
    }
}
