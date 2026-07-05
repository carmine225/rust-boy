mod macros; // Registra il file macro.rs
mod cpu;   // Registra il file cpu.rs
mod mmu;

fn main() {
    let mut cpu = cpu::Cpu::new();
    let mut mmu = mmu::Mmu::new();   
    
    cpu.step(&mut mmu, 0x0000); // Passa una reference al MMU e il PC iniziale
}
