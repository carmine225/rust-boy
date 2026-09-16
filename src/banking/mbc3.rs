use crate::banking::Banking;
use crate::rtc::Rtc;
pub struct Mbc3 {
    pub current_rom_bank: u8,
    ram_rtc_select: u8,
    latched_rtc: [u8; 5],
    latch_state: u8,
    rct: Rtc,
}
impl Mbc3 {
    pub fn new() -> Self {
        Mbc3 {
            current_rom_bank: 1,
            ram_rtc_select: 0,
            latch_state: 0xFF,
            latched_rtc: [0; 5],
            rct: Rtc::new(),
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
                let bank = if self.current_rom_bank == 0 {
                    1
                } else {
                    self.current_rom_bank
                };
                let offset = (bank as usize * 0x4000) + (address - 0x4000) as usize;
                banking.card_rom.get(offset).copied().unwrap_or(0xFF)
            }

            0xA000..=0xBFFF => {
                if !banking.ram_enabled {
                    return 0xFF;
                }

                match self.ram_rtc_select {
                    // Banco RAM standard
                    0x00..=0x07 => {
                        let offset =
                            (self.ram_rtc_select as usize * 0x2000) + ((address - 0xA000) as usize);
                        banking.card_ram.get(offset).copied().unwrap_or(0xFF)
                    }
                    // Registro RTC latched
                    0x08..=0x0C => {
                        let idx = (self.ram_rtc_select - 0x08) as usize;
                        self.latched_rtc[idx]
                    }
                    _ => 0xFF,
                }
            }
            _ => 0xFF,
        }
    }
    pub fn write(&mut self, banking: &mut Banking, address: u16, value: u8) {
        match address {
            // Abilitazione RAM e RTC
            0x0000..=0x1FFF => {
                banking.ram_enabled = (value & 0x0F) == 0x0A;
            }

            // Selezione ROM Bank (7 bit)
            0x2000..=0x3FFF => {
                let mut bank = (value & 0x7F) as usize;
                if bank == 0 {
                    bank = 1;
                }
                self.current_rom_bank = bank as u8;
            }

            // Selezione Banco RAM (0x00..=0x03) o Registro RTC (0x08..=0x0C)
            0x4000..=0x5FFF => {
                self.ram_rtc_select = value;
            }

            // Latch RTC: rileva la transizione 0x00 -> 0x01
            0x6000..=0x7FFF => {
                if self.latch_state == 0x00 && value == 0x01 {
                    // Chiama il modulo rtc per popolare i 5 byte dall'orario di sistema
                    self.latched_rtc = self.rct.get_mbc3_registers();
                }
                self.latch_state = value;
            }

            // Scrittura RAM o Registri RTC
            0xA000..=0xBFFF => {
                if !banking.ram_enabled {
                    return;
                }

                match self.ram_rtc_select {
                    0x00..=0x07 => {
                        let offset =
                            (self.ram_rtc_select as usize * 0x2000) + ((address - 0xA000) as usize);
                        if offset < banking.card_ram.len() {
                            banking.card_ram[offset] = value;
                        }
                    }
                    0x08..=0x0C => {
                        let idx = (self.ram_rtc_select - 0x08) as usize;
                        self.rct.set_mbc3_register(idx, value);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
