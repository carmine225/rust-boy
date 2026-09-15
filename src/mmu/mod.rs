use crate::banking::Banking;
use std::path::PathBuf;
mod bios;
mod game;
mod save;

pub struct Mmu {
    wram: [u8; 8192],
    hram: [u8; 127],
    io_registers: [u8; 128],
    ie_register: u8,
    vram: [u8; 8192],
    oam: [u8; 160],
    banking: Banking,
    game_path: PathBuf,
    bios: Vec<u8>,
    bios_path: PathBuf,
    save_path: PathBuf,
}

impl Mmu {
    pub fn new() -> Self {
        Mmu {
            wram: [0; 8192],
            hram: [0; 127],
            io_registers: [0; 128],
            ie_register: 0,
            vram: [0; 8192],
            oam: [0; 160],
            banking: Banking::new(),
            game_path: PathBuf::new(),
            bios: Vec::new(),
            bios_path: PathBuf::new(),
            save_path: PathBuf::new(),
        }
    }

    pub fn read_byte(&self, banking: &mut Banking, address: u16) -> u8 {
        match address {
            // ROM Bank 00 (0x0000 - 0x3FFF) -> Primi 16 KiB fissi
            0x0000..=0x3FFF => {
                let idx = address as usize;
                if idx < banking.card_rom.len() {
                    banking.card_rom[idx]
                } else {
                    0xFF
                }
            }

            // ROM Bank 01..N (0x4000 - 0x7FFF) -> Calcolato col banco attivo
            0x4000..=0x7FFF => {
                let bank = banking.mapper.current_rom_bank();
                let offset = ((bank as usize * 0x4000) + ((address - 0x4000) as usize)) as usize;
                if offset < banking.card_rom.len() {
                    banking.card_rom[offset]
                } else {
                    0xFF
                }
            }

            // VRAM (0x8000 - 0x9FFF)
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],

            // External RAM Cartuccia (0xA000 - 0xBFFF) -> Richiede RAM abilitata
            0xA000..=0xBFFF => {
                if !banking.ram_enabled || banking.card_ram.is_empty() {
                    return 0xFF;
                }
                let offset =
                    (banking.current_ram_bank as usize * 0x2000) + (address - 0xA000) as usize;
                if offset < banking.card_ram.len() {
                    banking.card_ram[offset]
                } else {
                    0xFF
                }
            }

            // WRAM (0xC000 - 0xDFFF)
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],

            // Echo RAM (0xE000 - 0xFDFF)
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],

            // OAM (0xFE00 - 0xFE9F)
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],

            // Area non utilizzata
            0xFEA0..=0xFEFF => 0xFF,

            // Registri I/O, HRAM, IE
            0xFF00..=0xFF7F => self.io_registers[(address - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.ie_register,
        }
    }

    pub fn write_byte(&mut self, banking: &mut Banking, address: u16, value: u8) {
        match address {
            // Scrittura in ROM -> Inoltrata al gestore del modulo banking  || External RAM Cartuccia (0xA000 - 0xBFFF)
            0x0000..=0x7FFF | 0xA000..=0xBFFF => banking.handle_mbc_write(address, value),

            // VRAM
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,

            // WRAM
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,

            // Echo RAM
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,

            // OAM
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,

            // Area non utilizzata
            0xFEA0..=0xFEFF => {}

            // Registri I/O, HRAM, IE
            0xFF00..=0xFF7F => self.io_registers[(address - 0xFF00) as usize] = value,
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.ie_register = value,
        }
    }
}
