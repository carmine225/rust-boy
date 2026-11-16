use crate::ui::Emu;
use crate::ui::GameEntry;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path;

impl Emu {
    pub fn add_game(&mut self, rom_path: Option<path::PathBuf>) {
        match rom_path {
            Some(path) => {
                self.games.push(GameEntry {
                    title: Self::get_rom_title(&path),
                    subtitle: path.display().to_string(),
                    rom_path: path,
                });
            }
            None => {}
        }
    }
    fn get_rom_title<P: AsRef<path::Path>>(path: P) -> String {
        // Usiamo un blocco o closures/helper per poter sfruttare la comoda sintassi '?'
        // e ritornare un fallback in caso di errore.
        let read_title = || -> Option<String> {
            let mut file = File::open(path).ok()?;

            // Il flag CGB è all'indirizzo 0x0143 e identifica la lunghezza del titolo.
            file.seek(SeekFrom::Start(0x0143)).ok()?;
            let mut cgb_flag = [0u8; 1];
            file.read_exact(&mut cgb_flag).ok()?;
            let title_length = if cgb_flag[0] & 0x80 != 0 { 11 } else { 16 };

            file.seek(SeekFrom::Start(0x0134)).ok()?;
            let mut buffer = [0u8; 16];
            file.read_exact(&mut buffer[..]).ok()?;

            let title = String::from_utf8_lossy(&buffer[..title_length]);
            let cleaned = title
                .trim_matches(|c: char| c == '\0' || c.is_whitespace())
                .to_string();

            if cleaned.is_empty() {
                None
            } else {
                Some(cleaned)
            }
        };

        // Se qualcosa va storto o il titolo è vuoto, restituisce "Unknown" (o String::new())
        read_title().unwrap_or_else(|| String::from("Unknown ROM"))
    }
    pub fn add_folder(&mut self, folder_path: Option<path::PathBuf>) {
        match folder_path {
            Some(path) => {
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if entry_path.is_file() {
                            self.add_game(Some(entry_path));
                        }
                    }
                }
            }
            None => {}
        }
    }
}
