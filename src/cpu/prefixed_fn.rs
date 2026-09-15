//! Implementazione delle istruzioni CPU prefissate con 0xCB del Game Boy.
//!
//! Questo modulo contiene tutte le istruzioni prefissate che includono:
//! - Rotazioni circolari (RLC, RRC)
//! - Rotazioni con carry (RL, RR)
//! - Shift aritmetici e logici (SLA, SRA, SRL)
//! - Manipolazione di nibble (SWAP)
//! - Operazioni su bit singoli (BIT, RES, SET)

use crate::banking::Banking;
use crate::cpu::Cpu;
use crate::get_u16register;
use crate::mmu::Mmu;

impl Cpu {
    // ==========================================
    // 1. RLC
    // ==========================================
    /// RLC r8 - Rotate Left Circular su registro a 8-bit
    ///
    /// Ruota il valore a sinistra. Il bit 7 viene spostato al bit 0
    /// e copiato nel flag Carry. Modifica i flag Z, N, H, e C.
    pub fn rlc_r8(&mut self, val: u8) -> u8 {
        let bit_7 = (val >> 7) & 0x01;
        let res = (val << 1) | bit_7;
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_7 == 1);

        res
    }

    /// RLC (HL) - Rotate Left Circular su memoria (HL)
    ///
    /// Ruota il valore in memoria a sinistra. Il bit 7 va al bit 0
    /// e nel flag Carry. Modifica i flag Z, N, H, e C.
    pub fn rlc_hl_mem(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        let res = self.rlc_r8(val);
        mmu.write_byte(banking, hl, res);
        self.cycles = self.cycles.wrapping_add(8); // 16 cicli totali
    }

    // ==========================================
    // 2. RRC
    // ==========================================
    /// RRC r8 - Rotate Right Circular su registro a 8-bit
    ///
    /// Ruota il valore a destra. Il bit 0 viene spostato al bit 7
    /// e copiato nel flag Carry. Modifica i flag Z, N, H, e C.
    pub fn rrc_r8(&mut self, val: u8) -> u8 {
        let bit_0 = val & 0x01;
        let res = (val >> 1) | (bit_0 << 7);
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_0 == 1);
        res
    }

    /// RRC (HL) - Rotate Right Circular su memoria (HL)
    ///
    /// Ruota il valore in memoria a destra. Il bit 0 va al bit 7
    /// e nel flag Carry. Modifica i flag Z, N, H, e C.
    pub fn rrc_hl_mem(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        let res = self.rrc_r8(val);
        mmu.write_byte(banking, hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 3. RL
    // ==========================================
    /// RL r8 - Rotate Left su registro a 8-bit (con Carry)
    ///
    /// Ruota il valore a sinistra attraverso il flag Carry.
    /// Il flag Carry entra al bit 0, il bit 7 esce nel flag Carry.
    /// Modifica i flag Z, N, H, e C.
    pub fn rl_r8(&mut self, val: u8) -> u8 {
        let bit_7 = (val >> 7) & 0x01;
        let c = if self.get_flag_c() { 1 } else { 0 };
        let res = (val << 1) | c;
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_7 == 1);
        res
    }

    /// RL (HL) - Rotate Left su memoria (HL) (con Carry)
    ///
    /// Ruota il valore in memoria a sinistra attraverso il flag Carry.
    /// Il flag Carry entra al bit 0, il bit 7 esce nel flag Carry.
    pub fn rl_hl_mem(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        let res = self.rl_r8(val);
        mmu.write_byte(banking, hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 4. RR
    // ==========================================
    /// RR r8 - Rotate Right su registro a 8-bit (con Carry)
    ///
    /// Ruota il valore a destra attraverso il flag Carry.
    /// Il flag Carry entra al bit 7, il bit 0 esce nel flag Carry.
    /// Modifica i flag Z, N, H, e C.
    pub fn rr_r8(&mut self, val: u8) -> u8 {
        let bit_0 = val & 0x01;
        let c = if self.get_flag_c() { 1 } else { 0 };
        let res = (val >> 1) | (c << 7);
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_0 == 1);
        res
    }

    /// RR (HL) - Rotate Right su memoria (HL) (con Carry)
    ///
    /// Ruota il valore in memoria a destra attraverso il flag Carry.
    /// Il flag Carry entra al bit 7, il bit 0 esce nel flag Carry.
    pub fn rr_hl_mem(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        let res = self.rr_r8(val);
        mmu.write_byte(banking, hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 5. SLA
    // ==========================================
    /// SLA r8 - Shift Left Arithmetic su registro a 8-bit
    ///
    /// Sposta il valore a sinistra di 1 posizione. Bit 7 esce nel Carry.
    /// Bit 0 viene azzerato. Modifica i flag Z, N, H, e C.
    pub fn sla_r8(&mut self, val: u8) -> u8 {
        let bit_7 = (val >> 7) & 0x01;
        let res = val << 1;
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_7 == 1);
        res
    }

    /// SLA (HL) - Shift Left Arithmetic su memoria (HL)
    ///
    /// Sposta il valore in memoria a sinistra di 1 posizione.
    /// Bit 7 esce nel Carry, bit 0 viene azzerato.
    pub fn sla_hl_mem(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        let res = self.sla_r8(val);
        mmu.write_byte(banking, hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 6. SRA
    // ==========================================
    /// SRA r8 - Shift Right Arithmetic su registro a 8-bit
    ///
    /// Sposta il valore a destra di 1 posizione, mantenendo il bit di segno (bit 7).
    /// Bit 0 esce nel Carry. Modifica i flag Z, N, H, e C.
    pub fn sra_r8(&mut self, val: u8) -> u8 {
        let bit_0 = val & 0x01;
        let bit_7 = val & 0x80;
        let res = (val >> 1) | bit_7;
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_0 == 1);
        res
    }

    /// SRA (HL) - Shift Right Arithmetic su memoria (HL)
    ///
    /// Sposta il valore in memoria a destra, mantenendo il bit 7.
    /// Bit 0 esce nel Carry.
    pub fn sra_hl_mem(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        let res = self.sra_r8(val);
        mmu.write_byte(banking, hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 7. SWAP
    // ==========================================
    /// SWAP r8 - Scambia nibble di un registro a 8-bit
    ///
    /// Scambia il nibble alto e basso (4 bit) del valore.
    /// Azzera i flag N, H, C. Modifica Z.
    pub fn swap_r8(&mut self, val: u8) -> u8 {
        let res = (val >> 4) | (val << 4);
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
        res
    }

    /// SWAP (HL) - Scambia nibble in memoria (HL)
    ///
    /// Scambia il nibble alto e basso del valore in memoria.
    /// Azzera i flag N, H, C. Modifica Z.
    pub fn swap_hl_mem(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        let res = self.swap_r8(val);
        mmu.write_byte(banking, hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 8. SRL
    // ==========================================
    /// SRL r8 - Shift Right Logical su registro a 8-bit
    ///
    /// Sposta il valore a destra di 1 posizione. Bit 7 viene azzerato.
    /// Bit 0 esce nel Carry. Modifica i flag Z, N, H, e C.
    pub fn srl_r8(&mut self, val: u8) -> u8 {
        let bit_0 = val & 0x01;
        let res = val >> 1;
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_0 == 1);
        res
    }

    /// SRL (HL) - Shift Right Logical su memoria (HL)
    ///
    /// Sposta il valore in memoria a destra di 1 posizione.
    /// Bit 7 viene azzerato, bit 0 esce nel Carry.
    pub fn srl_hl_mem(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        let res = self.srl_r8(val);
        mmu.write_byte(banking, hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 9. BIT
    // ==========================================
    /// BIT b, r8 - Testa bit in un registro a 8-bit
    ///
    /// Verifica se il bit b è impostato nel valore.
    /// Azzera N, imposta H. Modifica Z in base al bit testato.
    /// Non modifica il valore.
    pub fn bit_b_r8(&mut self, bit: u8, val: u8) {
        let is_bit_set = (val & (1 << bit)) != 0;
        self.set_flag_z(!is_bit_set);
        self.set_flag_n(false);
        self.set_flag_h(true);
    }

    /// BIT b, (HL) - Testa bit in memoria (HL)
    ///
    /// Verifica se il bit b è impostato nel valore in memoria.
    /// Azzera N, imposta H. Modifica Z in base al bit testato.
    pub fn bit_b_hl_mem(&mut self, bit: u8, banking: &mut Banking, mmu: &Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        self.bit_b_r8(bit, val);
        self.cycles = self.cycles.wrapping_add(4); // 12 cicli totali con cb()
    }

    // ==========================================
    // 10. RES
    // ==========================================
    /// RES b, r8 - Azzera (Reset) bit in un registro a 8-bit
    ///
    /// Imposta il bit b a 0 nel valore.
    /// Restituisce il valore modificato.
    pub fn res_b_r8(&mut self, bit: u8, val: u8) -> u8 {
        val & !(1 << bit)
    }

    /// RES b, (HL) - Azzera (Reset) bit in memoria (HL)
    ///
    /// Imposta il bit b a 0 nel valore in memoria.
    pub fn res_b_hl_mem(&mut self, bit: u8, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        let res = self.res_b_r8(bit, val);
        mmu.write_byte(banking, hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 11. SET
    // ==========================================
    /// SET b, r8 - Setta bit in un registro a 8-bit
    ///
    /// Imposta il bit b a 1 nel valore.
    /// Restituisce il valore modificato.
    pub fn set_b_r8(&mut self, bit: u8, val: u8) -> u8 {
        val | (1 << bit)
    }

    /// SET b, (HL) - Setta bit in memoria (HL)
    ///
    /// Imposta il bit b a 1 nel valore in memoria.
    pub fn set_b_hl_mem(&mut self, bit: u8, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(banking, hl);
        let res = self.set_b_r8(bit, val);
        mmu.write_byte(banking, hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }
}
