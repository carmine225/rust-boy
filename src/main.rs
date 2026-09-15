mod banking;
mod cpu; // Registra il file cpu.rs
mod log;
mod macros; // Registra il file macro.rs
mod mmu; // Registra il file cartridge/mod.rs
use std::path::PathBuf;
mod rtc;

fn main() {
    let mut cpu = cpu::Cpu::new();
    let mut mmu = mmu::Mmu::new();
    let mut banking = banking::Banking::new();
    mmu.get_game_path();
    let path_log = log::log_path();
    let path_log = path_log.join("rust-boy.log");
    log::init_logger(&path_log);
    mmu.load_bios(PathBuf::from("bios/bios.gb"));
    mmu.load_game(&mut banking, PathBuf::from("games"));
    cpu.step(&mut banking, &mut mmu); // Passa banking, MMU e PC iniziale
}
