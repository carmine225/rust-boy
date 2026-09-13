use crate::mmu::Mmu;
mod huc1;
mod huc3;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod mbc6;
mod mbc7;
mod mmm01;
mod rom_only;
mod tama5;

impl Mmu {
    pub fn manager(&mut self, banking_mode: u8) {
        match banking_mode {
            // ROM ONLY
            0x00 | 0x08 | 0x09 => self._rom_only(banking_mode),

            // MBC1
            0x01..=0x03 => self._mbc1(),

            // MBC2
            0x05 | 0x06 => self._mbc2(),

            // MBC3
            0x0F..=0x13 => self._mbc3(),

            // MBC5
            0x19..=0x1E => self._mbc5(),

            // MBC6
            0x20 => self._mbc6(),

            // MBC7
            0x22 => self._mbc7(),

            // Chip Speciali / Esotici
            0xFE => self._huc3(),
            0xFF => self._huc1(),
            0xEA => self._tama5(),
            0x0B..=0x0D => self._mmm01(),

            _ => {}
        }
    }
    pub fn ram_manager(&mut self, ram_size: u8) {
        match ram_size {
            0 => {
                self.current_ram_bank = 0;
                self.card_ram = vec![0; 0x2000];
            }
            1 => {
                self.current_ram_bank = 1;
                self.card_ram = vec![0; 0x4000];
            }
            2 => {
                self.current_ram_bank = 2;
                self.card_ram = vec![0; 0x6000];
            }
            3 => {
                self.current_ram_bank = 3;
                self.card_ram = vec![0; 0x8000];
            }
            _ => {}
        }
    }
    pub fn handle_mbc_write(&mut self, address: u16, value: u8) {
        todo!("handle mbc write");
    }
}
