mod banking;
mod cpu;
mod log;
mod macros;
mod mmu;
mod rtc;
mod system;
mod timer;

use std::fs;
use std::io;
use std::path::PathBuf;
use system::System;

fn main() {
    let app_path = PathBuf::from(env!("CARGO_PKG_NAME"));
    let bios_path = app_path.join("bios");
    let game_path = app_path.join("game");
    let save_path = app_path.join("save");
    let log_path = app_path.join("log");

    for path in [&bios_path, &game_path, &save_path, &log_path] {
        fs::create_dir_all(path).expect("Impossibile creare la cartella dell'app");
    }

    let game_file = fs::read_dir(&game_path)
        .expect("Impossibile leggere la cartella dei giochi")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("gb"))
        })
        .expect("Nessun file .gb trovato nella cartella rust-boy/game");

    println!("Premi Invio per avviare {}", game_file.display());
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Impossibile leggere l'input");

    // Inizializza la MMU e carica i file
    let mut mmu = mmu::Mmu::new();
    mmu.set_bios_path(bios_path);
    mmu.set_game_path(game_file.clone());
    mmu.set_save_path(save_path);

    let log_file_path = log_path.join("rust-boy.log");
    log::init_logger(&log_file_path);

    mmu.load_bios();
    mmu.load_game();

    // Crea il coordinator System passando la MMU inizializzata
    let mut system = System::new(mmu);

    // Se non stai usando una Boot ROM reale, imposta il PC allo stato post-boot DMG (0x0100)
    system.cpu.pc = 0x0100;

    println!("Avvio del loop di emulazione...");

    // Loop principale: esegue l'emulazione frame per frame
    loop {
        system.step_frame();

        // TODO (P0.7): Inserire qui l'aggiornamento della PPU e del rendering grafico
    }
}
