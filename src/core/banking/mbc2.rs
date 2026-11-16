use crate::banking::Banking;
pub struct Mbc2 {
    pub current_rom_bank: u8,
    ram_enabled: bool,
}
impl Mbc2 {
    pub fn new() -> Self {
        Mbc2 {
            current_rom_bank: 1,
            ram_enabled: false,
        }
    }
    pub fn read(&mut self, banking: &mut Banking, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => banking
                .card_rom
                .get(address as usize)
                .copied()
                .unwrap_or(0xFF),
            0x4000..=0x7FFF => {
                let offset =
                    ((self.current_rom_bank as u16 * 0x4000) + (address - 0x4000)) as usize;
                banking.card_rom.get(offset).copied().unwrap_or(0xFF)
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    0xFF
                } else {
                    let i = (address as usize) & 0x01FF;
                    banking.card_ram.get(i).copied().unwrap_or(0x0F) | 0xF0
                }
            }
            _ => 0xFF,
        }
    }
    pub fn write(&mut self, banking: &mut Banking, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                if (address & 0x0100) == 0 {
                    self.ram_enabled = (value & 0x0F) == 0x0A;
                }
            }
            0x2000..=0x3FFF => {
                if (address & 0x0100) != 0 {
                    let mut bank = (value & 0x0F) as usize;
                    if bank == 0 {
                        bank = 1;
                    }
                    self.current_rom_bank = bank as u8;
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let idx = (address as usize) & 0x01FF;
                    if let Some(byte) = banking.card_ram.get_mut(idx) {
                        *byte = value & 0x0F;
                    }
                }
            }
            _ => {}
        }
    }
}
