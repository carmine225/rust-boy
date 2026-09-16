use crate::mmu::Mmu;
use std::fs;
use std::path::PathBuf;

impl Mmu {
    pub fn load_bios(&mut self) {
        let data = fs::read(self.bios_path.clone()).expect("Failed to read BIOS file");
        self.bios = data;
    }

    pub fn set_bios_path(&mut self, bios_path: PathBuf) {
        self.bios_path = bios_path.join("bios.gb");
    }
}
