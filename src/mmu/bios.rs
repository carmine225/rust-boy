use crate::mmu::Mmu;
use std::fs;
use std::path::PathBuf;

impl Mmu {
    pub fn load_bios(&mut self, bios_path: PathBuf) {
        let data = fs::read(bios_path).expect("Failed to read BIOS file");
        self.bios = data;
    }

    pub fn get_bios_path(&mut self) {
        fs::create_dir_all("bios").unwrap();
        self.bios_path = PathBuf::from("bios");
    }
}
