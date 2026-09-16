use crate::banking::{huc1::Huc1, rom_only::RomOnly};

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
    pub fn current_read(&mut self, banking: &mut Banking, address: u16) -> u8 {
        match self {
            Mapper::Huc1(m) => m.read(address),
            Mapper::Huc3(m) => m.read(address),
            Mapper::M161(m) => m.read(address),
            Mapper::Mbc1(m) => m.read(banking, address),
            Mapper::Mbc2(m) => m.read(banking, address),
            Mapper::Mbc3(m) => m.read(banking, address),
            Mapper::Mbc5(m) => m.read(address),
            Mapper::Mbc6(m) => m.read(address),
            Mapper::Mbc7(m) => m.read(address),
            Mapper::Mmm01(m) => m.read(address),
            Mapper::Tama5(m) => m.read(address),
            Mapper::RomOnly(_) => RomOnly::read(banking, address),
            Mapper::WisdomTree(m) => m.read(address),
        }
    }
    pub fn current_write(&mut self, banking: &mut Banking, address: u16, value: u8) {
        match self {
            Mapper::Huc1(m) => m.write(address, value),
            Mapper::Huc3(m) => m.write(address, value),
            Mapper::M161(m) => m.write(address, value),
            Mapper::Mbc1(m) => m.write(banking, address, value),
            Mapper::Mbc2(m) => m.write(banking, address, value),
            Mapper::Mbc3(m) => m.write(banking, address, value),
            Mapper::Mbc5(m) => m.write(address, value),
            Mapper::Mbc6(m) => m.write(address, value),
            Mapper::Mbc7(m) => m.write(address, value),
            Mapper::Mmm01(m) => m.write(address, value),
            Mapper::Tama5(m) => m.write(address, value),
            Mapper::RomOnly(_) => RomOnly::write(banking, address, value),
            Mapper::WisdomTree(m) => m.write(address, value),
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
}
impl Banking {
    pub fn new() -> Self {
        Banking {
            mapper: Mapper::RomOnly(rom_only::RomOnly::new(0x00)),
            mbc_type: 0,
            banking_mode: 0,
            card_rom: Vec::new(),
            card_ram: Vec::new(),
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
                self.mapper = Mapper::Mbc2(mbc2::Mbc2::new())
            }

            // MBC3
            0x0F..=0x13 => {
                self.current_ram_bank = 0;
                self.ram_enabled = false;
                self.mapper = Mapper::Mbc3(mbc3::Mbc3::new())
            }

            // MBC5
            0x19..=0x1E => self.mapper = Mapper::Mbc5(mbc5::Mbc5::new()),

            // MBC6
            0x20 => self.mapper = Mapper::Mbc6(mbc6::Mbc6::new()),

            // MBC7
            0x22 => self.mapper = Mapper::Mbc7(mbc7::Mbc7::new()),

            // Chip Speciali / Esotici
            0xFE => self.mapper = Mapper::Huc3(huc3::Huc3::new()),
            0xFF => self.mapper = Mapper::Huc1(huc1::Huc1::new()),
            0xEE => self.mapper = Mapper::M161(m161::M161::new()),
            0xEA => self.mapper = Mapper::Tama5(tama5::Tama5::new()),
            0x0B..=0x0D => self.mapper = Mapper::Mmm01(mmm01::Mmm01::new()),
            _ => {}
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let replacement = Banking::new().mapper;
        let mut mapper = std::mem::replace(&mut self.mapper, replacement);
        mapper.current_write(self, address, value);
        self.mapper = mapper;
    }
    pub fn read(&mut self, address: u16) -> u8 {
        let replacement = Banking::new().mapper;
        let mut mapper = std::mem::replace(&mut self.mapper, replacement);
        let readed = mapper.current_read(self, address);
        self.mapper = mapper;
        readed
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
}
