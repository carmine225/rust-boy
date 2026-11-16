use crate::{cpu, mmu::Mmu};

impl Mmu {
    pub fn set_bios(&mut self, enable: bool, cpu: &mut cpu::Cpu) {
        self.bios_enable = enable;
        if enable {
            cpu.pc = 0x0000;
        } else {
            *cpu = cpu.bios_off();
        }
    }
}
