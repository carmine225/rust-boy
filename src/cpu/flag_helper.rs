use crate::cpu::Cpu;

const FLAG_Z_MASK: u8 = 0x80; // 1000 0000
const FLAG_N_MASK: u8 = 0x40; // 0100 0000
const FLAG_H_MASK: u8 = 0x20; // 0010 0000
const FLAG_C_MASK: u8 = 0x10; // 0001 0000
///helper functions for the flags of the CPU
impl Cpu {
    // --- FLAG Z (ZERO) ---
    pub fn get_flag_z(&self) -> bool {
        (self.f & FLAG_Z_MASK) != 0
    }
    pub fn set_flag_z(&mut self, value: bool) {
        if value {
            self.f |= FLAG_Z_MASK;
        } else {
            self.f &= !FLAG_Z_MASK;
        }
    }

    // --- FLAG N (SUBTRACTION) ---
    pub fn get_flag_n(&self) -> bool {
        (self.f & FLAG_N_MASK) != 0
    }
    pub fn set_flag_n(&mut self, value: bool) {
        if value {
            self.f |= FLAG_N_MASK;
        } else {
            self.f &= !FLAG_N_MASK;
        }
    }

    // --- FLAG H (HALF-CARRY) ---
    pub fn get_flag_h(&self) -> bool {
        (self.f & FLAG_H_MASK) != 0
    }
    pub fn set_flag_h(&mut self, value: bool) {
        if value {
            self.f |= FLAG_H_MASK;
        } else {
            self.f &= !FLAG_H_MASK;
        }
    }

    // --- FLAG C (CARRY) ---
    pub fn get_flag_c(&self) -> bool {
        (self.f & FLAG_C_MASK) != 0
    }
    pub fn set_flag_c(&mut self, value: bool) {
        if value {
            self.f |= FLAG_C_MASK;
        } else {
            self.f &= !FLAG_C_MASK;
        }
    }
}
