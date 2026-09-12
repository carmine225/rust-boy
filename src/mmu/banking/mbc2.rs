use crate::mmu::Mmu;

impl Mmu {
    pub fn _mbc2(&mut self) {
        self.current_rom_bank = 2;
        self.current_ram_bank = 255;
    }
}
