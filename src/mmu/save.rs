use crate::mmu::Mmu;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

impl Mmu {
    // Carica il file .sav all'avvio
    pub fn load_save_file(&mut self) {
        if let Ok(mut file) = fs::File::open(self.save_path.clone()) {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                let copy_len = buffer.len().min(self.banking.card_ram.len());
                self.banking.card_ram[..copy_len].copy_from_slice(&buffer[..copy_len]);
            }
        }
    }

    // Salva il vettore card_ram su file .sav
    pub fn save_to_disk(&self, save_path: &str) {
        if !self.banking.card_ram.is_empty() {
            if let Ok(mut file) = fs::File::create(save_path) {
                let _ = file.write_all(&self.banking.card_ram);
            }
        }
    }
    pub fn set_save_path(&mut self, save_path: PathBuf) {
        self.save_path = save_path;
    }
}
