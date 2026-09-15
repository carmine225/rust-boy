use crate::rtc::Rtc;
mod huc1;
mod huc3;
mod m161;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod mbc6;
mod mbc7;
mod mmm01;
mod rom_only;
mod tama5;
mod wisdom_tree;

pub enum Mapper {
    Huc1(huc1::Huc1),
    Huc3(huc3::Huc3),
    M161(m161::M161),
    Mbc1(mbc1::Mbc1),
    Mbc2(mbc2::Mbc2),
    Mbc3(mbc3::Mbc3),
    Mbc5(mbc5::Mbc5),
    Mbc6(mbc6::Mbc6),
    Mbc7(mbc7::Mbc7),
    Mmm01(mmm01::Mmm01),
    RomOnly(rom_only::RomOnly),
    Tama5(tama5::Tama5),
    WisdomTree(wisdom_tree::WisdomTree),
}
impl Mapper {
    pub fn current_rom_bank(&self) -> u8 {
        match self {
            Mapper::Huc1(m) => m.current_rom_bank as u8,
            Mapper::Huc3(m) => m.current_rom_bank as u8,
            Mapper::M161(m) => m.current_rom_bank as u8,
            Mapper::Mbc1(m) => m.current_rom_bank as u8,
            Mapper::Mbc2(m) => m.current_rom_bank as u8,
            Mapper::Mbc3(m) => m.current_rom_bank as u8,
            Mapper::Mbc5(m) => m.current_rom_bank as u8,
            Mapper::Mbc6(m) => m.current_rom_bank as u8,
            Mapper::Mbc7(m) => m.current_rom_bank as u8,
            Mapper::Mmm01(m) => m.current_rom_bank as u8,
            Mapper::RomOnly(m) => m.current_rom_bank as u8,
            Mapper::Tama5(m) => m.current_rom_bank as u8,
            Mapper::WisdomTree(m) => m.current_rom_bank as u8,
        }
    }
}

pub struct Banking {
    pub mapper: Mapper,
    pub current_ram_bank: u8,
    mbc_type: u8,
    banking_mode: u8,
    pub card_rom: Vec<u8>,
    pub card_ram: Vec<u8>,
    pub ram_enabled: bool,
    rtc: Rtc,
}
impl Banking {
    pub fn new() -> Self {
        Banking {
            mapper: Mapper::RomOnly(rom_only::RomOnly::new(0x00)),
            mbc_type: 0,
            banking_mode: 0,
            card_rom: Vec::new(),
            card_ram: Vec::new(),
            rtc: Rtc::new(),
            current_ram_bank: 0,
            ram_enabled: false,
        }
    }
    pub fn manager(&mut self, banking_mode: u8) {
        self.mbc_type = banking_mode;
        match banking_mode {
            // ROM ONLY
            0x00 | 0x08 | 0x09 => {
                self.mapper = Mapper::RomOnly(rom_only::RomOnly::new(banking_mode))
            }

            // MBC1
            0x01..=0x03 => {
                self.banking_mode = 0;
                self.current_ram_bank = 0;
                self.mapper = Mapper::Mbc1(mbc1::Mbc1::new())
            }

            // MBC2
            0x05 | 0x06 => {
                self.card_ram = vec![0; 512];
                self.mapper = Mapper::Mbc2(mbc2::Mbc2::new(banking_mode))
            }

            // MBC3
            0x0F..=0x13 => {
                self.current_ram_bank = 0;
                self.ram_enabled = false;
                self.mapper = Mapper::Mbc3(mbc3::Mbc3::new())
            }

            // MBC5
            0x19..=0x1E => {}

            // MBC6
            0x20 => {}

            // MBC7
            0x22 => {}

            // Chip Speciali / Esotici
            0xFE => {}
            0xFF => {}
            0xEE => {}
            0xEA => {}
            0x0B..=0x0D => {}
            _ => {}
        }
    }
    pub fn ram_manager(&mut self, ram_size_code: u8) {
        // Il banco attivo parte sempre da 0 all'inizializzazione
        self.current_ram_bank = 0;
        let size_in_bytes = match ram_size_code {
            0x00 => 0,       // Nessuna RAM
            0x01 => 0x800,   // 2 KiB (Inusuale, usata in pochissime ROM)
            0x02 => 0x2000,  // 8 KiB (1 banco)
            0x03 => 0x8000,  // 32 KiB (4 banchi da 8 KiB)
            0x04 => 0x20000, // 128 KiB (16 banchi da 8 KiB)
            0x05 => 0x10000, // 64 KiB (8 banchi da 8 KiB)
            _ => 0,
        };
        if (0x00..=0x05).contains(&size_in_bytes) {
            self.ram_enabled = true;
        }
        self.card_ram = vec![0; size_in_bytes];
    }
    pub fn handle_mbc_write(&mut self, address: u16, value: u8) {
        match self.mbc_type {
            // ROM ONLY
            0x00 | 0x08 | 0x09 => {}

            // MBC1
            0x01..=0x03 => {}

            // MBC2
            0x05 | 0x06 => {}

            // MBC3
            0x0F..=0x13 => {}

            // MBC5
            0x19..=0x1E => {}

            // MBC6
            0x20 => {}

            // MBC7
            0x22 => {}

            //huc3
            0xFE => {}

            //huc1
            0xFF => {}

            //tama5
            0xEA => {}

            //m161
            0xEE => {}

            //mmm01
            0x0B..=0x0D => {}

            _ => {}
        }
    }
}
