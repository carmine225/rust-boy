use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::set_u16register;
use crate::timer::Timer;
impl Cpu {
    pub fn handle_interrupts(&mut self, mmu: &mut Mmu, timer: &mut Timer) -> u32 {
        let ie = mmu.read_byte(0xFFFF, timer);
        let if_reg = mmu.read_byte(0xFF0F, timer);
        let pending = ie & if_reg & 0x1F;
        if pending != 0 {
            self.halted = false;
        }
        if !self.ime || pending == 0 {
            return 0;
        }
        self.ime = false;
        let interrupt_bit = pending.trailing_zeros() as u8;
        let new_if = if_reg & !(1 << interrupt_bit);
        mmu.write_byte(0xFF0F, new_if, timer);
        let (high, low): (u8, u8);
        set_u16register!(Self, high, low, self.pc);
        self.sp = self.sp.wrapping_sub(1);
        mmu.write_byte(self.sp, high, timer);
        self.sp = self.sp.wrapping_sub(1);
        mmu.write_byte(self.sp, low, timer);
        self.pc = match interrupt_bit {
            0 => 0x0040, // VBlank
            1 => 0x0048, // LCD STAT
            2 => 0x0050, // Timer
            3 => 0x0058, // Serial
            4 => 0x0060, // Joypad
            _ => self.pc,
        };
        20
    }
}

#[cfg(test)]
mod tests {
    use crate::{cpu::Cpu, mmu::Mmu, timer::Timer};

    #[test]
    fn each_interrupt_dispatches_to_its_vector_and_pushes_pc() {
        for (interrupt_bit, vector) in [
            (0, 0x0040),
            (1, 0x0048),
            (2, 0x0050),
            (3, 0x0058),
            (4, 0x0060),
        ] {
            let mut cpu = Cpu::new();
            let mut mmu = Mmu::new();
            let mut timer = Timer::new();
            cpu.pc = 0x1234;
            cpu.sp = 0xFFFE;
            cpu.ime = true;
            mmu.write_byte(0xFFFF, 1 << interrupt_bit, &mut timer);
            mmu.write_byte(0xFF0F, 1 << interrupt_bit, &mut timer);

            assert_eq!(cpu.handle_interrupts(&mut mmu, &mut timer), 20);
            assert_eq!(cpu.pc, vector);
            assert_eq!(cpu.sp, 0xFFFC);
            assert_eq!(mmu.read_byte(0xFFFC, &mut timer), 0x34);
            assert_eq!(mmu.read_byte(0xFFFD, &mut timer), 0x12);
            assert!(!cpu.ime);
            assert_eq!(mmu.read_byte(0xFF0F, &mut timer) & (1 << interrupt_bit), 0);
        }
    }

    #[test]
    fn interrupt_priority_selects_lowest_pending_bit() {
        let mut cpu = Cpu::new();
        let mut mmu = Mmu::new();
        let mut timer = Timer::new();
        cpu.pc = 0x2000;
        cpu.sp = 0xFFFE;
        cpu.ime = true;
        mmu.write_byte(0xFFFF, 0b10101, &mut timer);
        mmu.write_byte(0xFF0F, 0b10101, &mut timer);

        cpu.handle_interrupts(&mut mmu, &mut timer);

        assert_eq!(cpu.pc, 0x0040);
        assert_eq!(mmu.read_byte(0xFF0F, &mut timer) & 0x1F, 0b10100);
    }

    #[test]
    fn pending_interrupt_wakes_halt_even_when_ime_is_disabled() {
        let mut cpu = Cpu::new();
        let mut mmu = Mmu::new();
        let mut timer = Timer::new();
        cpu.halted = true;
        mmu.write_byte(0xFFFF, 0x01, &mut timer);
        mmu.write_byte(0xFF0F, 0x01, &mut timer);

        assert_eq!(cpu.handle_interrupts(&mut mmu, &mut timer), 0);
        assert!(!cpu.halted);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn disabled_or_unrequested_interrupt_does_not_dispatch() {
        let mut cpu = Cpu::new();
        let mut mmu = Mmu::new();
        let mut timer = Timer::new();
        cpu.pc = 0x1234;
        cpu.sp = 0xFFFE;
        cpu.ime = true;
        mmu.write_byte(0xFFFF, 0x01, &mut timer);
        mmu.write_byte(0xFF0F, 0x02, &mut timer);

        assert_eq!(cpu.handle_interrupts(&mut mmu, &mut timer), 0);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
        assert!(cpu.ime);
    }
}
