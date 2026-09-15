use crate::banking::Banking;
use crate::mmu::Mmu;
use std::fs;
use std::path::PathBuf;

impl Mmu {
    pub fn load_game(&mut self, banking: &mut Banking, game_path: PathBuf) {
        banking.card_rom = fs::read(game_path).expect("Failed to read game file");
        banking.manager(banking.card_rom[0x147]);
        banking.ram_manager(banking.card_ram[0x148]);
    }

    pub fn get_game_path(&mut self) {
        fs::create_dir_all("games").unwrap();
        self.game_path = PathBuf::from("games");
    }
}
