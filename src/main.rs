mod cpu; // Registra il file cpu.rs
mod log;
mod macros; // Registra il file macro.rs
mod mmu; // Registra il file cartridge/mod.rs
use std::path::PathBuf;

fn main() {
    let mut cpu = cpu::Cpu::new();
    let mut mmu = mmu::Mmu::new();
    mmu.get_game_path();
    let path_log = log::log_path();
    let path_log = path_log.join("rust-boy.log");
    log::init_logger(&path_log);
    mmu.load_bios(PathBuf::from("bios/bios.gb"));
    mmu.load_game(PathBuf::from("games"));
    cpu.step(&mut mmu); // Passa una reference al MMU e il PC iniziale
}
