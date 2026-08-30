//! Funzioni di supporto per la gestione dei flag della CPU.
//!
//! Il registro dei flag (F) contiene 4 flag significativi nel formato `ZNHC----`:
//! - **Z** (bit 7, 0x80): Flag Zero - impostato quando il risultato è 0
//! - **N** (bit 6, 0x40): Flag Subtraction - impostato su sottrazione
//! - **H** (bit 5, 0x20): Flag Half-Carry - riporto dal nibble basso
//! - **C** (bit 4, 0x10): Flag Carry - riporto dal risultato

use crate::cpu::Cpu;

const FLAG_Z_MASK: u8 = 0x80; // 1000 0000
const FLAG_N_MASK: u8 = 0x40; // 0100 0000
const FLAG_H_MASK: u8 = 0x20; // 0010 0000
const FLAG_C_MASK: u8 = 0x10; // 0001 0000

impl Cpu {
    // --- FLAG Z (ZERO) ---
    /// Legge il flag Zero (Z).
    ///
    /// Restituisce `true` se il flag Z è impostato (risultato precedente era 0).
    /// Il flag Z si trova al bit 7 (0x80) del registro F.
    pub fn get_flag_z(&self) -> bool {
        (self.f & FLAG_Z_MASK) != 0
    }

    /// Imposta il flag Zero (Z).
    ///
    /// Se `value` è `true`, il flag Z viene impostato (bit 7 = 1).
    /// Se `value` è `false`, il flag Z viene azzerato (bit 7 = 0).
    pub fn set_flag_z(&mut self, value: bool) {
        if value {
            self.f |= FLAG_Z_MASK;
        } else {
            self.f &= !FLAG_Z_MASK;
        }
    }

    // --- FLAG N (SUBTRACTION) ---
    /// Legge il flag Subtraction (N).
    ///
    /// Restituisce `true` se il flag N è impostato (ultima operazione era una sottrazione).
    /// Il flag N si trova al bit 6 (0x40) del registro F.
    pub fn get_flag_n(&self) -> bool {
        (self.f & FLAG_N_MASK) != 0
    }

    /// Imposta il flag Subtraction (N).
    ///
    /// Se `value` è `true`, il flag N viene impostato (bit 6 = 1).
    /// Se `value` è `false`, il flag N viene azzerato (bit 6 = 0).
    pub fn set_flag_n(&mut self, value: bool) {
        if value {
            self.f |= FLAG_N_MASK;
        } else {
            self.f &= !FLAG_N_MASK;
        }
    }

    // --- FLAG H (HALF-CARRY) ---
    /// Legge il flag Half-Carry (H).
    ///
    /// Restituisce `true` se il flag H è impostato (riporto dal nibble basso).
    /// Il flag H si trova al bit 5 (0x20) del registro F.
    pub fn get_flag_h(&self) -> bool {
        (self.f & FLAG_H_MASK) != 0
    }

    /// Imposta il flag Half-Carry (H).
    ///
    /// Se `value` è `true`, il flag H viene impostato (bit 5 = 1).
    /// Se `value` è `false`, il flag H viene azzerato (bit 5 = 0).
    pub fn set_flag_h(&mut self, value: bool) {
        if value {
            self.f |= FLAG_H_MASK;
        } else {
            self.f &= !FLAG_H_MASK;
        }
    }

    // --- FLAG C (CARRY) ---
    /// Legge il flag Carry (C).
    ///
    /// Restituisce `true` se il flag C è impostato (riporto dal risultato).
    /// Il flag C si trova al bit 4 (0x10) del registro F.
    pub fn get_flag_c(&self) -> bool {
        (self.f & FLAG_C_MASK) != 0
    }

    /// Imposta il flag Carry (C).
    ///
    /// Se `value` è `true`, il flag C viene impostato (bit 4 = 1).
    /// Se `value` è `false`, il flag C viene azzerato (bit 4 = 0).
    pub fn set_flag_c(&mut self, value: bool) {
        if value {
            self.f |= FLAG_C_MASK;
        } else {
            self.f &= !FLAG_C_MASK;
        }
    }
}
