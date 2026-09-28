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
