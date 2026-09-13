use crate::mmu::Mmu;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

impl Mmu {
    // Carica il file .sav all'avvio
    pub fn load_save_file(&mut self, save_path: &str) {
        if let Ok(mut file) = fs::File::open(save_path) {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                self.card_ram = buffer;
            }
        }
    }

    // Salva il vettore card_ram su file .sav
    pub fn save_to_disk(&self, save_path: &str) {
        if !self.card_ram.is_empty() {
            if let Ok(mut file) = fs::File::create(save_path) {
                let _ = file.write_all(&self.card_ram);
            }
        }
    }
    pub fn get_save_path(&mut self) {
        fs::create_dir_all("saves").unwrap();
        self.save_path = PathBuf::from("saves");
    }
}
