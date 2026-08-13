mod cpu; // Registra il file cpu.rs
mod macros; // Registra il file macro.rs
mod mmu;

fn main() {
    let mut cpu = cpu::Cpu::new();
    let mut mmu = mmu::Mmu::new();

    cpu.step(&mut mmu); // Passa una reference al MMU e il PC iniziale
}
