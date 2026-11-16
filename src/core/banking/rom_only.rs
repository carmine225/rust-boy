use crate::banking::Banking;

pub struct RomOnly {
    pub current_rom_bank: u8,
    pub current_ram_bank: u8,
    ram_enabled: bool,
}
impl RomOnly {
    pub fn new(value: u8) -> Self {
        RomOnly {
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: value == 0x08 || value == 0x09,
        }
    }
    pub fn read(&self, banking: &mut Banking, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => banking.card_rom.get(address as usize).copied().unwrap_or(0xFF),
            0xA000..=0xBFFF if self.ram_enabled => banking
                .card_ram
                .get((address - 0xA000) as usize)
                .copied()
                .unwrap_or(0xFF),
            _ => 0xFF,
        }
    }
    pub fn write(&self, banking: &mut Banking, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => {}
            0xA000..=0xBFFF if self.ram_enabled => {
                if let Some(byte) = banking.card_ram.get_mut((address - 0xA000) as usize) {
                    *byte = value;
                }
            }
            _ => {}
        }
    }
}
