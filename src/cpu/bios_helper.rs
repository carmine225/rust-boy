use crate::cpu::Cpu;

impl Cpu {
    pub fn bios_off(&mut self) -> Self {
        Cpu {
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
            cycles: 0,
            stopped: false,
            halted: false,
            halt_bug_triggered: false,
            ime: false,
            interrupt_enable: 0x00,
            interrupt_flag: 0xE1,
        }
    }
}
