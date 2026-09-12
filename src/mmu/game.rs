use crate::mmu::Mmu;
use std::fs;
use std::path::PathBuf;

impl Mmu {
    pub fn load_game(&mut self, game_path: PathBuf) {
        self.card_rom = fs::read(game_path).expect("Failed to read game file");
    }

    pub fn get_game_path(&mut self) {
        fs::create_dir_all("games").unwrap();
        self.game_path = PathBuf::from("games");
    }
}
