use crate::{cpu, mmu::Mmu};

impl Mmu {
    pub fn set_bios(&mut self, enable: bool, cpu: &mut cpu::Cpu) {
        if !enable {
            cpu.bios_off();
            self.bios_enabe = false;
        }
    }
}
