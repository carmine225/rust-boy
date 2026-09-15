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
    pub fn read(banking: &mut Banking, address: u16) -> u8 {
        if address < banking.card_rom.len() as u16 {
            banking.card_rom[address as usize]
        } else {
            0xFF
        }
    }
    pub fn write(banking: &mut Banking, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => {}
            0xA000..=0xBFFF => {
                if address < banking.card_rom.len() as u16 {
                    banking.card_ram[address as usize] = value;
                }
            }
            _ => {}
        }
    }
}
