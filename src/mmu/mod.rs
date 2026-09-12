use std::path::PathBuf;
pub mod banking;
pub mod bios;
pub mod game;

pub struct Mmu {
    wram: [u8; 8192],
    hram: [u8; 127],
    io_registers: [u8; 128],
    ie_register: u8,
    vram: [u8; 8192],
    oam: [u8; 160],
    current_rom_bank: u8,
    current_ram_bank: u8,
    card_rom: Vec<u8>,
    card_ram: Vec<u8>,
    game_path: PathBuf,
    bios: Vec<u8>,
    bios_path: PathBuf,
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
            current_rom_bank: 1,
            current_ram_bank: 255,
            card_rom: Vec::new(),
            card_ram: Vec::new(),
            game_path: PathBuf::new(),
            bios: Vec::new(),
            bios_path: PathBuf::new(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // ROM Cartuccia (0x0000 - 0x7FFF)
            0x0000..=0x7FFF => {
                let idx = address as usize;
                if idx < self.card_rom.len() {
                    self.card_rom[idx]
                } else {
                    0xFF
                }
            }
            // VRAM (0x8000 - 0x9FFF)
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],

            // External RAM Cartuccia (0xA000 - 0xBFFF)
            0xA000..=0xBFFF => {
                let idx = (address - 0xA000) as usize;
                if idx < self.card_ram.len() {
                    self.card_ram[idx]
                } else {
                    0xFF
                }
            }
            // WRAM (0xC000 - 0xDFFF)
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],

            // Echo RAM (0xE000 - 0xFDFF) -> specchio di WRAM
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],

            // OAM (0xFE00 - 0xFE9F)
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],

            // Area non utilizzata (0xFEA0 - 0xFEFF)
            0xFEA0..=0xFEFF => 0x00,

            // Registri I/O (0xFF00 - 0xFF7F)
            0xFF00..=0xFF7F => self.io_registers[(address - 0xFF00) as usize],

            // HRAM (0xFF80 - 0xFFFE)
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],

            // Interrupt Enable (0xFFFF)
            0xFFFF => self.ie_register,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // Scrittura in ROM -> usata dai chip MBC per cambiare banchi
            0x0000..=0x7FFF => {}
            // VRAM (0x8000 - 0x9FFF)
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,

            // External RAM Cartuccia (0xA000 - 0xBFFF)
            0xA000..=0xBFFF => {
                let idx = (address - 0xA000) as usize;
                if idx < self.card_ram.len() {
                    self.card_ram[idx] = value;
                }
            }
            // WRAM (0xC000 - 0xDFFF)
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,

            // Echo RAM (0xE000 - 0xFDFF) -> scrive direttamente in WRAM
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,

            // OAM (0xFE00 - 0xFE9F)
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,

            // Area non utilizzata (0xFEA0 - 0xFEFF)
            0xFEA0..=0xFEFF => {}

            // Registri I/O (0xFF00 - 0xFF7F)
            0xFF00..=0xFF7F => self.io_registers[(address - 0xFF00) as usize] = value,

            // HRAM (0xFF80 - 0xFFFE)
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,

            // Interrupt Enable (0xFFFF)
            0xFFFF => self.ie_register = value,
        }
    }
}
