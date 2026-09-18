mod banking;
mod cpu; // Registra il file cpu.rs
mod log;
mod macros; // Registra il file macro.rs
mod mmu; // Registra il file cartridge/mod.rs
mod system;
mod timer;
use std::fs;
use std::io;
use std::path::PathBuf;
mod rtc;

fn main() {
    let mut cpu = cpu::Cpu::new();
    let mut mmu = mmu::Mmu::new();

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

    println!("Inserisci un input per avviare {}", game_file.display());
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Impossibile leggere l'input");

    mmu.set_bios_path(bios_path);
    mmu.set_game_path(game_file.clone());
    mmu.set_save_path(save_path);
    let log_file_path = log_path.join("rust-boy.log");
    log::init_logger(&log_file_path);
    mmu.load_bios();
    mmu.load_game();
    let mut timer = timer::Timer::new();
    cpu.step(&mut mmu, &mut timer); // Passa MMU, timer e PC iniziale
}
