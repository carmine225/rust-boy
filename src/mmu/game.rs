use crate::mmu::Mmu;
use std::fs;

impl Mmu {
    pub fn load_game(&mut self) {
        self.banking.card_rom = fs::read(&self.game_path).expect("Failed to read game file");
        let cartridge_type = self.banking.card_rom[0x147];
        let ram_size_code = self.banking.card_rom[0x148];
        self.banking.manager(cartridge_type);
        self.banking.ram_manager(ram_size_code);
    }

    pub fn set_game_path(&mut self, game_path: std::path::PathBuf) {
        self.game_path = game_path;
    }
}
