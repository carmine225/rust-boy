use crate::mmu::Mmu;

impl Mmu {
    pub fn _mbc1(&mut self) {
        self.current_rom_bank = 1;
    }
}
