use crate::{banking::Banking, timer};
use std::path::PathBuf;
mod bios;
mod game;
mod helper;
mod save;

pub struct Mmu {
    wram: [u8; 8192],
    hram: [u8; 127],
    pub io_registers: [u8; 128],
    ie_register: u8,
    vram: [u8; 8192],
    oam: [u8; 160],
    banking: Banking,
    game_path: PathBuf,
    bios: Vec<u8>,
    bios_enabe: bool,
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
            bios_enabe: false,
            bios_path: PathBuf::new(),
            save_path: PathBuf::new(),
        }
    }

    pub fn read_byte(&mut self, address: u16, timer: &mut timer::Timer) -> u8 {
        match address {
            // ROM Bank 00 (0x0000 - 0x3FFF) -> Primi 16 KiB fissi
            // ROM e RAM della cartuccia: il mapper deve controllare entrambe
            // le finestre ROM, inclusa la banca fissa di MBC1 in mode 1.
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.banking.read(address),

            // VRAM (0x8000 - 0x9FFF)
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],

            // WRAM (0xC000 - 0xDFFF)
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],

            // Echo RAM (0xE000 - 0xFDFF)
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],

            // OAM (0xFE00 - 0xFE9F)
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],

            // Area non utilizzata
            0xFEA0..=0xFEFF => 0xFF,

            // Registri I/O, HRAM, IE
            // Registri I/O, HRAM, IE
            0xFF04..=0xFF07 => timer.read_byte(address),
            0xFF0F => self.io_registers[0x0F] | 0xE0,
            0xFF50 => {
                if self.bios_enabe {
                    0x00
                } else {
                    0xFF
                }
            }
            0xFF00..=0xFF7F => self.io_registers[(address - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.ie_register,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8, timer: &mut timer::Timer) {
        match address {
            // Scrittura in ROM -> Inoltrata al gestore del modulo banking  || External RAM Cartuccia (0xA000 - 0xBFFF)
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.banking.write(address, value),

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
            // Registri I/O, HRAM, IE
            0xFF04..=0xFF07 => timer.write_byte(address, value), // Intercetta le scritture sul Timer
            0xFF00..=0xFF7F => self.io_registers[(address - 0xFF00) as usize] = value,
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.ie_register = value,
        }
    }
}
