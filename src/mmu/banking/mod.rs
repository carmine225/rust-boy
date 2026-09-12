use crate::mmu::Mmu;
mod mbc1;
mod mbc2;

impl Mmu {
    pub fn manager(&mut self, banking_mode: u8) {
        match banking_mode {
            1 | 2 | 3 => self._mbc1(),
            5 | 6 => self._mbc2(),
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
}
