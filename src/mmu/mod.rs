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
    bios_enable: bool,
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
            bios_enable: false,
            bios_path: PathBuf::new(),
            save_path: PathBuf::new(),
        }
    }

    pub fn read_byte(&mut self, address: u16, timer: &mut timer::Timer) -> u8 {
        match address {
            // Mappatura BIOS: 0x0000 - 0x00FF se attivo
            0x0000..=0x00FF if self.bios_enable => {
                if let Some(&byte) = self.bios.get(address as usize) {
                    byte
                } else {
                    self.banking.read(address)
                }
            }

            // ROM e RAM della cartuccia (0x0000 - 0x7FFF | 0xA000 - 0xBFFF)
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.banking.read(address),

            // VRAM (0x8000 - 0x9FFF)
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],

            // WRAM (0xC000 - 0xDFFF)
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],

            // Echo RAM (0xE000 - 0xFDFF) -> Alias diretto di C000 - DDFF
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],

            // OAM (0xFE00 - 0xFE9F)
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],

            // Area non utilizzata
            0xFEA0..=0xFEFF => 0xFF,

            // Timer (0xFF04 - 0xFF07)
            0xFF04..=0xFF07 => timer.read_byte(address),

            // IF Register (0xFF0F) - I 3 bit superiori sono inutilizzati e ritornano 1 su DMG
            0xFF0F => self.io_registers[0x0F] | 0xE0,

            // Registro disattivazione BIOS (0xFF50)
            0xFF50 => {
                if self.bios_enable {
                    0x00
                } else {
                    0xFF
                }
            }

            // Registri I/O generici (0xFF00 - 0xFF7F)
            0xFF00..=0xFF7F => self.io_registers[(address - 0xFF00) as usize],

            // HRAM (0xFF80 - 0xFFFE)
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],

            // IE Register (0xFFFF)
            0xFFFF => self.ie_register,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8, timer: &mut timer::Timer) {
        match address {
            // Cartuccia / Banking
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.banking.write(address, value),

            // VRAM
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,

            // WRAM
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,

            // Echo RAM (scrittura specchiata su WRAM)
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,

            // OAM
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,

            // Area non utilizzata
            0xFEA0..=0xFEFF => {}

            // Timer
            0xFF04..=0xFF07 => timer.write_byte(address, value),

            // DMA OAM: trasferisce una pagina di 160 byte in OAM
            0xFF46 => {
                self.io_registers[0x46] = value;
                let source = (value as u16) << 8;
                for offset in 0..160 {
                    let byte = self.read_byte(source + offset as u16, timer);
                    self.oam[offset] = byte;
                }
            }

            // Disattivazione permanente BIOS se viene scritto un valore != 0
            0xFF50 => {
                if value != 0 {
                    self.bios_enable = false;
                }
            }

            // Registri I/O generici
            0xFF00..=0xFF7F => self.io_registers[(address - 0xFF00) as usize] = value,

            // HRAM
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,

            // IE Register
            0xFFFF => self.ie_register = value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Mmu;
    use crate::timer::Timer;

    #[test]
    fn memory_regions_and_echo_ram_are_mapped_correctly() {
        let mut mmu = Mmu::new();
        let mut timer = Timer::new();

        mmu.write_byte(0x8000, 0x12, &mut timer);
        mmu.write_byte(0x9FFF, 0x34, &mut timer);
        mmu.write_byte(0xC000, 0x56, &mut timer);
        mmu.write_byte(0xDDFF, 0x78, &mut timer);
        mmu.write_byte(0xFE00, 0x9A, &mut timer);
        mmu.write_byte(0xFE9F, 0xBC, &mut timer);
        mmu.write_byte(0xFF80, 0xDE, &mut timer);
        mmu.write_byte(0xFFFE, 0xF0, &mut timer);

        assert_eq!(mmu.read_byte(0x8000, &mut timer), 0x12);
        assert_eq!(mmu.read_byte(0x9FFF, &mut timer), 0x34);
        assert_eq!(mmu.read_byte(0xC000, &mut timer), 0x56);
        assert_eq!(mmu.read_byte(0xE000, &mut timer), 0x56);
        assert_eq!(mmu.read_byte(0xDDFF, &mut timer), 0x78);
        assert_eq!(mmu.read_byte(0xFDFF, &mut timer), 0x78);
        assert_eq!(mmu.read_byte(0xFE00, &mut timer), 0x9A);
        assert_eq!(mmu.read_byte(0xFE9F, &mut timer), 0xBC);
        assert_eq!(mmu.read_byte(0xFF80, &mut timer), 0xDE);
        assert_eq!(mmu.read_byte(0xFFFE, &mut timer), 0xF0);
        assert_eq!(mmu.read_byte(0xFEA0, &mut timer), 0xFF);
        mmu.write_byte(0xFEA0, 0x11, &mut timer);
        assert_eq!(mmu.read_byte(0xFEA0, &mut timer), 0xFF);
    }

    #[test]
    fn io_interrupt_and_timer_registers_are_routed_correctly() {
        let mut mmu = Mmu::new();
        let mut timer = Timer::new();

        mmu.write_byte(0xFF00, 0x21, &mut timer);
        mmu.write_byte(0xFF7F, 0x43, &mut timer);
        mmu.write_byte(0xFFFF, 0x1F, &mut timer);
        mmu.write_byte(0xFF0F, 0x05, &mut timer);
        mmu.write_byte(0xFF05, 0x67, &mut timer);

        assert_eq!(mmu.read_byte(0xFF00, &mut timer), 0x21);
        assert_eq!(mmu.read_byte(0xFF7F, &mut timer), 0x43);
        assert_eq!(mmu.read_byte(0xFFFF, &mut timer), 0x1F);
        assert_eq!(mmu.read_byte(0xFF0F, &mut timer), 0xE5);
        assert_eq!(mmu.read_byte(0xFF05, &mut timer), 0x67);
    }

    #[test]
    fn dma_copies_one_page_into_oam() {
        let mut mmu = Mmu::new();
        let mut timer = Timer::new();

        for offset in 0u16..160 {
            mmu.write_byte(0xC000 + offset, offset as u8, &mut timer);
        }
        mmu.write_byte(0xFF46, 0xC0, &mut timer);

        assert_eq!(mmu.read_byte(0xFF46, &mut timer), 0xC0);
        for offset in 0u16..160 {
            assert_eq!(mmu.read_byte(0xFE00 + offset, &mut timer), offset as u8);
        }
    }
}
