mod cpu; // Registra il file cpu.rs
mod log;
mod macros; // Registra il file macro.rs
mod mmu;

fn main() {
    let mut cpu = cpu::Cpu::new();
    let mut mmu = mmu::Mmu::new();
    let path = log::log_path();
    let path = path.join("rust-boy.log");
    log::init_logger(&path);
    cpu.step(&mut mmu); // Passa una reference al MMU e il PC iniziale
}
