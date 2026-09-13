use crate::mmu::Mmu;
use log::error;
impl Mmu {
    pub fn _rom_only(&mut self, value: u8) {
        self.current_rom_bank = value;
    }
    pub fn _rom_only_read(&mut self, address: u16) -> u8 {
        if address < self.card_rom.len() as u16 {
            self.card_rom[address as usize]
        } else {
            0xFF
        }
    }
    pub fn _rom_only_write() {
        error!("Attempted to write to ROM-only memory, which is not allowed.");
    }
}
