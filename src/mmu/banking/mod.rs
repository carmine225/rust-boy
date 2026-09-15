use crate::mmu::Mmu;
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

impl Mmu {
    pub fn manager(&mut self, banking_mode: u8) {
        match banking_mode {
            // ROM ONLY
            0x00 | 0x08 | 0x09 => self._rom_only(banking_mode),

            // MBC1
            0x01..=0x03 => self._mbc1(banking_mode),

            // MBC2
            0x05 | 0x06 => self._mbc2(banking_mode),

            // MBC3
            0x0F..=0x13 => self._mbc3(banking_mode),

            // MBC5
            0x19..=0x1E => self._mbc5(banking_mode),

            // MBC6
            0x20 => self._mbc6(banking_mode),

            // MBC7
            0x22 => self._mbc7(banking_mode),

            // Chip Speciali / Esotici
            0xFE => self._huc3(banking_mode),
            0xFF => self._huc1(banking_mode),
            0xEE => self._m161(banking_mode),
            0xEA => self._tama5(banking_mode),
            0x0B..=0x0D => self._mmm01(banking_mode),
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
            0x00 | 0x08 | 0x09 => self._rom_only_write(address, value),

            // MBC1
            0x01..=0x03 => self._mbc1_write(address, value),

            // MBC2
            0x05 | 0x06 => self._mbc2_write(address, value),

            // MBC3
            0x0F..=0x13 => self._mbc3_write(address, value),

            // MBC5
            0x19..=0x1E => self._mbc5_write(address, value),

            // MBC6
            0x20 => self._mbc6_write(address, value),

            // MBC7
            0x22 => self._mbc7_write(address, value),

            // Chip Speciali / Esotici
            0xFE => self._huc3_write(address, value),
            0xFF => self._huc1_write(address, value),
            0xEA => self._tama5_write(address, value),
            0xEE => self._m161_write(address, value),
            0x0B..=0x0D => self._mmm01_write(address, value),

            _ => {}
        }
    }
}
