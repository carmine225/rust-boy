//! Implementazione delle istruzioni CPU non-prefissate del Game Boy.
//!
//! Questo modulo contiene le implementazioni di tutte le istruzioni della CPU
//! che non iniziano con il prefisso 0xCB. Sono incluse operazioni di controllo,
//! aritmetiche, logiche, di memoria e di gestione del flusso di controllo.

use crate::banking::Banking;
use crate::cpu::Cpu;
use crate::get_u16register;
use crate::mmu::Mmu;

impl Cpu {
    /// NOP (No Operation) - 4 cicli
    ///
    /// Istruzione che non esegue alcuna operazione.
    /// Utilizzata principalmente per il timing e i ritardi.
    pub fn nop(&mut self) {
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// STOP - Ferma la CPU fino a un interrupt
    ///
    /// Incrementa il PC di 1 byte e mette la CPU nello stato "stopped".
    /// La CPU rimane ferma finché non si verifica un interrupt.
    pub fn stop(&mut self) {
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
        self.stopped = true; // Aggiungi un flag per indicare che la CPU è in stato di stop
    }

    /// HALT - Sospende l'esecuzione fino a un interrupt
    ///
    /// Se IME è impostato, la CPU viene messa nello stato "halted".
    /// Se IME non è impostato, può verificarsi l'"halt bug" se ci sono interrupt pendenti.
    pub fn halt(&mut self) {
        self.cycles = self.cycles.wrapping_add(4);
        if self.ime {
            self.halted = true; // Ferma l'esecuzione finché non arriva un interrupt
        } else {
            // Gestione opzionale del celebre "Halt Bug"
            if (self.interrupt_flag & self.interrupt_enable) & 0x1F != 0 {
                self.halt_bug_triggered = true;
            } else {
                self.halted = true;
            }
        }
    }

    /// DAA - Decimal Adjust Accumulator
    ///
    /// Corregge il registro A per operazioni BCD (Binary Coded Decimal).
    /// Modifica i flag Z e H, ma mantiene N e usa C per il riporto.
    pub fn daa(&mut self) {
        let a = self.a as u16;
        let mut correction = 0;
        if !self.get_flag_n() {
            // Caso 1: ADDIZIONE
            if self.get_flag_h() || (a & 0x0F) > 0x09 {
                correction |= 0x06;
            }
            if self.get_flag_c() || a > 0x99 {
                correction |= 0x60;
                self.set_flag_c(true);
            }
            self.a = (a as u8).wrapping_add(correction);
        } else {
            if self.get_flag_h() {
                correction |= 0x06;
            }
            if self.get_flag_c() {
                correction |= 0x60;
            }
            self.a = (a as u8).wrapping_sub(correction);
        }
        self.set_flag_z(self.a == 0);
        self.set_flag_h(false);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// CPL - Complement A (NOT logico)
    ///
    /// Inverte tutti i bit del registro A (complemento a 1).
    /// Imposta i flag N e H, non modifica Z e C.
    pub fn cpl(&mut self) {
        self.a = !self.a;
        self.set_flag_n(true);
        self.set_flag_h(true);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// SCF - Set Carry Flag
    ///
    /// Imposta il flag di Carry a 1.
    /// Azzera i flag N e H.
    pub fn scf(&mut self) {
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(true);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// CCF - Complement Carry Flag
    ///
    /// Inverte il flag di Carry.
    /// Azzera i flag N e H.
    pub fn ccf(&mut self) {
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(!self.get_flag_c());
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// RLCA - Rotate Left Circular A
    ///
    /// Ruota il registro A a sinistra. Il bit 7 viene spostato al bit 0
    /// e copiato nel flag Carry. Flag Z è sempre 0.
    pub fn rlca(&mut self) {
        let mut a = self.a;
        let bit_7 = (self.a >> 7) & 0x01;
        a = (a << 1) | bit_7;
        self.a = a;
        self.set_flag_z(false);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_7 == 1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// RRCA - Rotate Right Circular A
    ///
    /// Ruota il registro A a destra. Il bit 0 viene spostato al bit 7
    /// e copiato nel flag Carry. Flag Z è sempre 0.
    pub fn rrca(&mut self) {
        let mut a = self.a;
        let bit_0 = a & 0x01;
        a = (a >> 1) | (bit_0 << 7);
        self.a = a;
        self.set_flag_z(false);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_0 == 1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// RLA - Rotate Left A (con Carry)
    ///
    /// Ruota il registro A a sinistra attraverso il flag Carry.
    /// Il flag Carry entra al bit 0, il bit 7 esce nel flag Carry.
    pub fn rla(&mut self) {
        let bit_7 = (self.a >> 7) & 0x01;
        let carry = if self.get_flag_c() { 1 } else { 0 };
        self.a = (self.a << 1) | carry;
        self.set_flag_z(false);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_7 == 1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// RRA - Rotate Right A (con Carry)
    ///
    /// Ruota il registro A a destra attraverso il flag Carry.
    /// Il flag Carry entra al bit 7, il bit 0 esce nel flag Carry.
    pub fn rra(&mut self) {
        let bit_0 = self.a & 0x01;
        let carry = if self.get_flag_c() { 1 } else { 0 };
        self.a = (self.a >> 1) | (carry << 7);
        self.set_flag_z(false);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_0 == 1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// DEC (HL), r8 - Decrementa valore in memoria (HL)
    ///
    /// Decrementa di 1 il valore puntato da HL.
    /// Modifica i flag Z, N, e H. Il flag C non viene modificato.
    pub fn dec_hl_mem(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, hl);
        let result = value.wrapping_sub(1);
        mmu.write_byte(banking, hl, result);
        self.set_flag_z(result == 0);
        self.set_flag_n(true);
        self.set_flag_h((value & 0x0F) == 0x00);
        self.cycles = self.cycles.wrapping_add(12);
    }

    /// LD (HL), n8 - Carica byte immediato in memoria (HL)
    ///
    /// Legge un byte immediato dal PC e lo scrive in memoria all'indirizzo HL.
    /// Incrementa il PC di 1.
    pub fn ld_hl_mem_imm8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, self.pc);
        mmu.write_byte(banking, hl, value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(12);
    }

    /// INC (HL), r8 - Incrementa valore in memoria (HL)
    ///
    /// Incrementa di 1 il valore puntato da HL.
    /// Modifica i flag Z, N, e H. Il flag C non viene modificato.
    pub fn inc_hl_mem(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, hl);
        let result = value.wrapping_add(1);
        mmu.write_byte(banking, hl, result);
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h((value & 0x0F) == 0x0F);
        self.cycles = self.cycles.wrapping_add(12);
    }

    /// ADD HL, r16 - Addizione 16-bit
    ///
    /// Addiziona il registro r16 a HL (16-bit).
    /// Modifica i flag N, H, e C. Il flag Z non viene modificato.
    pub fn add_hl_r16(&mut self, src: u16) {
        let hl_32 = get_u16register!(self, self.h, self.l) as u32;
        let val_32 = src as u32;
        let result_32 = hl_32 + val_32;
        self.set_flag_n(false); // Sempre false nelle addizioni
        let half_carry = ((hl_32 & 0x0FFF) + (val_32 & 0x0FFF)) > 0x0FFF;
        self.set_flag_h(half_carry);
        self.set_flag_c(result_32 > 0xFFFF);
        self.h = ((result_32 >> 8) & 0xFF) as u8;
        self.l = (result_32 & 0xFF) as u8;
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// LD (r16), A - Scrive A in memoria all'indirizzo r16
    ///
    /// Scrive il registro A in memoria all'indirizzo specificato da r16.
    pub fn ld_mem_r16_a(&mut self, banking: &mut Banking, mmu: &mut Mmu, src: u16) {
        mmu.write_byte(banking, src, self.a);
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// LD A, (r16) - Legge da memoria all'indirizzo r16 in A
    ///
    /// Legge un byte dalla memoria all'indirizzo r16 e lo carica in A.
    pub fn ld_a_mem_r16(&mut self, banking: &mut Banking, mmu: &mut Mmu, src: u16) {
        self.a = mmu.read_byte(banking, src);
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// LD (n16), SP - Scrive SP in memoria all'indirizzo n16
    ///
    /// Legge un indirizzo immediato a 16-bit dal PC e scrive SP in memoria
    /// a quell'indirizzo (byte basso e alto separatamente).
    pub fn ld_mem16_sp(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let low = mmu.read_byte(banking, self.pc) as u16;
        let high = mmu.read_byte(banking, self.pc.wrapping_add(1)) as u16;
        let addr = low | (high << 8);
        mmu.write_byte(banking, addr, self.sp as u8);
        mmu.write_byte(banking, addr.wrapping_add(1), (self.sp >> 8) as u8);
        self.pc = self.pc.wrapping_add(2);
        self.cycles = self.cycles.wrapping_add(20);
    }

    /// LD (HL+), A - Scrive A in memoria e incrementa HL
    ///
    /// Scrive il registro A all'indirizzo HL e poi incrementa HL di 1.
    /// Utile per copiare dati sequenziali in memoria.
    pub fn ld_hl_inc_a(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        mmu.write_byte(banking, hl, self.a);
        let hl = hl.wrapping_add(1);
        self.h = (hl >> 8) as u8;
        self.l = (hl & 0xFF) as u8;
        self.cycles = self.cycles.wrapping_add(8); // Aggiorna il conteggio dei cicli in base all'operazione
    }

    /// LD A, (HL+) - Legge da memoria e incrementa HL
    ///
    /// Legge un byte dall'indirizzo HL in A e poi incrementa HL di 1.
    /// Utile per leggere dati sequenziali dalla memoria.
    pub fn ld_a_hl_inc(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        self.a = mmu.read_byte(banking, hl);
        let hl = hl.wrapping_add(1);
        self.h = (hl >> 8) as u8;
        self.l = (hl & 0xFF) as u8;
        self.cycles = self.cycles.wrapping_add(8); // Aggiorna il conteggio dei cicli in base all'operazione
    }

    /// LD (HL-), A - Scrive A in memoria e decrementa HL
    ///
    /// Scrive il registro A all'indirizzo HL e poi decrementa HL di 1.
    pub fn ld_hl_dec_a(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        mmu.write_byte(banking, hl, self.a);
        let hl = hl.wrapping_sub(1);
        self.h = (hl >> 8) as u8;
        self.l = (hl & 0xFF) as u8;
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// LD A, (HL-) - Legge da memoria e decrementa HL
    ///
    /// Legge un byte dall'indirizzo HL in A e poi decrementa HL di 1.
    pub fn ld_a_hl_dec(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        self.a = mmu.read_byte(banking, hl);
        let hl = hl.wrapping_sub(1);
        self.h = (hl >> 8) as u8;
        self.l = (hl & 0xFF) as u8;
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// LD (HL), r8 - Scrive registro in memoria (HL)
    ///
    /// Scrive il valore del registro src all'indirizzo HL.
    pub fn ld_mem_hl_r8(&mut self, banking: &mut Banking, mmu: &mut Mmu, src: u8) {
        let hl = get_u16register!(self, self.h, self.l);
        mmu.write_byte(banking, hl, src);
        self.cycles = self.cycles.wrapping_add(8); // Aggiorna il conteggio dei cicli in base all'operazione
    }

    /// LD r8, (HL) - Legge memoria (HL) in registro
    ///
    /// Legge un byte dall'indirizzo HL e lo restituisce.
    pub fn ld_r8_mem_hl(&mut self, banking: &mut Banking, mmu: &mut Mmu) -> u8 {
        let hl = get_u16register!(self, self.h, self.l);
        let data = mmu.read_byte(banking, hl);
        self.cycles = self.cycles.wrapping_add(8);
        data
    }

    /// ADD A, (HL) - Addizione con valore in memoria
    ///
    /// Addiziona il valore in memoria (HL) ad A.
    /// Modifica i flag Z, N, H, e C.
    pub fn alu_add_mem_hl(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, hl);
        let result = (self.a as u16) + (value as u16);
        let final_result = result as u8;
        self.set_flag_z(final_result == 0);
        self.set_flag_n(false);
        let half_carry = ((self.a & 0x0F) + (value & 0x0F)) > 0x0F;
        self.set_flag_h(half_carry);
        self.set_flag_c(result > 0xFF);
        self.a = final_result;
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// SUB A, (HL) - Sottrazione con valore in memoria
    ///
    /// Sottrae il valore in memoria (HL) da A.
    /// Modifica i flag Z, N, H, e C.
    pub fn alu_sub_mem_hl(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, hl);
        let final_result = self.a.wrapping_sub(value);
        self.set_flag_z(final_result == 0);
        self.set_flag_n(true);
        let half_borrow = (self.a & 0x0F) < (value & 0x0F);
        self.set_flag_h(half_borrow);
        let borrow = self.a < value;
        self.set_flag_c(borrow);
        self.a = final_result;
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// CP A, (HL) - Confronta A con valore in memoria
    ///
    /// Confronta il registro A con il valore in memoria (HL) (sottrazione senza salvataggio).
    /// Modifica i flag Z, N, H, e C.
    pub fn alu_cp_mem_hl(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, hl);
        let result = (self.a as u16).wrapping_sub(value as u16);
        let final_result = result as u8;
        self.set_flag_z(final_result == 0);
        self.set_flag_n(true);
        let half_carry = (self.a & 0x0F) < (value & 0x0F);
        self.set_flag_h(half_carry);
        self.set_flag_c(self.a < value);
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// ADC A, (HL) - Addizione con Carry dal valore in memoria
    ///
    /// Addiziona il valore in memoria (HL) e il flag Carry ad A.
    /// Modifica i flag Z, N, H, e C.
    pub fn alu_adc_mem_hl(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, hl);
        let carry = if self.get_flag_c() { 1 } else { 0 };
        let final_result = self.a.wrapping_add(value).wrapping_add(carry);
        self.set_flag_z(final_result == 0);
        self.set_flag_n(false);
        let half_carry = ((self.a & 0x0F) + (value & 0x0F) + carry) > 0x0F;
        self.set_flag_h(half_carry);
        let overflow_check = (self.a as u16) + (value as u16) + (carry as u16);
        self.set_flag_c(overflow_check > 0xFF);
        self.a = final_result;
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// SBC A, (HL) - Sottrazione con Carry dal valore in memoria
    ///
    /// Sottrae il valore in memoria (HL) e il flag Carry da A.
    /// Modifica i flag Z, N, H, e C.
    pub fn alu_sbc_mem_hl(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl_addr = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, hl_addr);
        let carry = if self.get_flag_c() { 1 } else { 0 };
        let a = self.a;
        let result = (a as i32) - (value as i32) - carry;
        self.a = result as u8;
        self.set_flag_z(self.a == 0);
        self.set_flag_n(true); // Sempre true per SBC
        let half_carry = (a & 0x0F) as i32 - (value & 0x0F) as i32 - carry < 0;
        self.set_flag_h(half_carry);
        let carry_out = result < 0;
        self.set_flag_c(carry_out);
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// AND A, (HL) - AND logico con valore in memoria
    ///
    /// Esegue AND logico tra A e il valore in memoria (HL).
    /// Imposta H=true, azz era N, H, C. Modifica Z.
    pub fn alu_and_mem_hl(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, hl);
        self.a &= value;
        self.set_flag_z(self.a == 0);
        self.set_flag_n(false);
        self.set_flag_h(true);
        self.set_flag_c(false);
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// XOR A, (HL) - XOR logico con valore in memoria
    ///
    /// Esegue XOR logico tra A e il valore in memoria (HL).
    /// Azzera i flag N, H, C. Modifica Z.
    pub fn alu_xor_mem_hl(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, hl);
        self.a ^= value;
        self.set_flag_z(self.a == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// OR A, (HL) - OR logico con valore in memoria
    ///
    /// Esegue OR logico tra A e il valore in memoria (HL).
    /// Azzera i flag N, H, C. Modifica Z.
    pub fn alu_or_mem_hl(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(banking, hl);
        self.a |= value;
        self.set_flag_z(self.a == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// LD r16, n16 - Carica valore immediato a 16-bit in registro doppio
    ///
    /// Legge due byte dal PC (low, high) e restituisce il valore a 16-bit.
    /// Incrementa il PC di 2.
    pub fn ld_r16_imm16(&mut self, banking: &mut Banking, mmu: &mut Mmu) -> u16 {
        let value_low = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);
        let value_high = mmu.read_byte(banking, self.pc);
        let final_value = (value_high as u16) << 8 | (value_low as u16);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(12);
        final_value
    }

    /// INC r16 - Incrementa registro a 16-bit
    ///
    /// Incrementa il registro a 16-bit di 1.
    /// Non modifica alcun flag.
    pub fn inc_r16(&mut self, src: u16) -> u16 {
        self.cycles = self.cycles.wrapping_add(8);
        src.wrapping_add(1)
    }

    /// DEC r16 - Decrementa registro a 16-bit
    ///
    /// Decrementa il registro a 16-bit di 1.
    /// Non modifica alcun flag.
    pub fn dec_r16(&mut self, src: u16) -> u16 {
        self.cycles = self.cycles.wrapping_add(8);
        src.wrapping_sub(1)
    }

    /// INC r8 - Incrementa registro a 8-bit
    ///
    /// Incrementa il registro a 8-bit di 1.
    /// Modifica i flag Z, N, e H. Il flag C non viene modificato.
    pub fn inc_r8(&mut self, src: u8) -> u8 {
        let result = src.wrapping_add(1);
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h((src & 0x0F) == 0x0F);
        self.cycles = self.cycles.wrapping_add(4);
        result
    }

    /// DEC r8 - Decrementa registro a 8-bit
    ///
    /// Decrementa il registro a 8-bit di 1.
    /// Modifica i flag Z, N, e H. Il flag C non viene modificato.
    pub fn dec_r8(&mut self, src: u8) -> u8 {
        let result = src.wrapping_sub(1);
        self.set_flag_z(result == 0);
        self.set_flag_n(true);
        self.set_flag_h((src & 0x0F) == 0x00);
        self.cycles = self.cycles.wrapping_add(4);
        result
    }

    /// LD r8, n8 - Carica byte immediato in registro a 8-bit
    ///
    /// Legge un byte dal PC e lo restituisce.
    /// Incrementa il PC di 1.
    pub fn ld_r8_imm8(&mut self, banking: &mut Banking, mmu: &mut Mmu) -> u8 {
        let value = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
        value
    }

    /// JR cond, e8 - Salto relativo condizionato
    ///
    /// Se la condizione è vera, salta di un offset relativo (8-bit con segno).
    /// Se la condizione è falsa, continua l'esecuzione sequenziale.
    /// Incrementa il PC di 1 per leggere l'offset.
    pub fn jr_cond(&mut self, condition: bool, banking: &mut Banking, mmu: &Mmu) {
        let offset_raw = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);

        if condition {
            let offset = offset_raw as i8;
            self.pc = ((self.pc as i32) + (offset as i32)) as u16;
            self.cycles = self.cycles.wrapping_add(12);
        } else {
            self.cycles = self.cycles.wrapping_add(8);
        }
    }

    /// LD r8, r8 - Copia byte tra registri
    ///
    /// Copia il valore dal registro src nel registro di destinazione.
    /// Non modifica alcun flag.
    pub fn ld_r8_r8(&mut self, src: u8) -> u8 {
        self.cycles = self.cycles.wrapping_add(4);
        src
    }

    /// ADD A, r8 - Addizione con registro a 8-bit
    ///
    /// Addiziona il registro r8 ad A.
    /// Modifica i flag Z, N, H, e C.
    pub fn alu_add(&mut self, src: u8) {
        let result = (self.a as u16) + (src as u16);
        let final_result = result as u8;
        self.set_flag_z(final_result == 0);
        self.set_flag_n(false);
        let half_carry = ((self.a & 0x0F) + (src & 0x0F)) > 0x0F;
        self.set_flag_h(half_carry);
        self.set_flag_c(result > 0xFF);
        self.a = final_result;
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// ADC A, r8 - Addizione con Carry dal registro a 8-bit
    ///
    /// Addiziona il registro r8 e il flag Carry ad A.
    /// Modifica i flag Z, N, H, e C.
    pub fn alu_adc(&mut self, src: u8) {
        let carry = if self.get_flag_c() { 1 } else { 0 };
        let final_result = self.a.wrapping_add(src).wrapping_add(carry);
        self.set_flag_z(final_result == 0);
        self.set_flag_n(false);
        let half_carry = ((self.a & 0x0F) + (src & 0x0F) + carry) > 0x0F;
        self.set_flag_h(half_carry);
        let overflow_check = (self.a as u16) + (src as u16) + (carry as u16);
        self.set_flag_c(overflow_check > 0xFF);
        self.a = final_result;
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// SBC A, r8 - Sottrazione con Carry dal registro a 8-bit
    ///
    /// Sottrae il registro r8 e il flag Carry da A.
    /// Modifica i flag Z, N, H, e C.
    pub fn alu_sbc(&mut self, src: u8) {
        let carry = if self.get_flag_c() { 1 } else { 0 };
        let final_result = self.a.wrapping_sub(src).wrapping_sub(carry);
        self.set_flag_z(final_result == 0);
        self.set_flag_n(true);
        let half_borrow = (self.a & 0x0F) < (src & 0x0F) + carry;
        self.set_flag_h(half_borrow);
        let borrow = (self.a as u16) < (src as u16) + (carry as u16);
        self.set_flag_c(borrow);
        self.a = final_result;
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// SUB A, r8 - Sottrazione da registro a 8-bit
    ///
    /// Sottrae il registro r8 da A.
    /// Modifica i flag Z, N, H, e C.
    pub fn alu_sub(&mut self, src: u8) {
        let final_result = self.a.wrapping_sub(src);
        self.set_flag_z(final_result == 0);
        self.set_flag_n(true);
        let half_borrow = (self.a & 0x0F) < (src & 0x0F);
        self.set_flag_h(half_borrow);
        let borrow = self.a < src;
        self.set_flag_c(borrow);
        self.a = final_result;
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// AND A, r8 - AND logico con registro a 8-bit
    ///
    /// Esegue AND logico tra A e il registro r8.
    /// Imposta H=true, azzera N, H, C. Modifica Z.
    pub fn alu_and(&mut self, src: u8) {
        self.a &= src;
        self.set_flag_z(self.a == 0);
        self.set_flag_n(false);
        self.set_flag_h(true);
        self.set_flag_c(false);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// XOR A, r8 - XOR logico con registro a 8-bit
    ///
    /// Esegue XOR logico tra A e il registro r8.
    /// Azzera i flag N, H, C. Modifica Z.
    pub fn alu_xor(&mut self, src: u8) {
        self.a ^= src;
        self.set_flag_z(self.a == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// OR A, r8 - OR logico con registro a 8-bit
    ///
    /// Esegue OR logico tra A e il registro r8.
    /// Azzera i flag N, H, C. Modifica Z.
    pub fn alu_or(&mut self, src: u8) {
        self.a |= src;
        self.set_flag_z(self.a == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// CP A, r8 - Confronta A con registro a 8-bit
    ///
    /// Confronta il registro A con r8 (sottrazione senza salvataggio).
    /// Modifica i flag Z, N, H, e C.
    pub fn alu_cp(&mut self, src: u8) {
        let result = (self.a as u16).wrapping_sub(src as u16);
        let final_result = result as u8;
        self.set_flag_z(final_result == 0);
        self.set_flag_n(true);
        let half_carry = (self.a & 0x0F) < (src & 0x0F);
        self.set_flag_h(half_carry);
        self.set_flag_c(self.a < src);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// POP r16 - Estrae coppia registri dallo stack
    ///
    /// Legge due byte dallo stack e li combina in un valore a 16-bit.
    /// Incrementa SP di 2.
    pub fn pop_r16(&mut self, banking: &mut Banking, mmu: &mut Mmu) -> u16 {
        let low = mmu.read_byte(banking, self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = mmu.read_byte(banking, self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(12);
        low as u16 | ((high as u16) << 8)
    }

    /// PUSH r16 - Carica coppia registri nello stack
    ///
    /// Scrive due byte (high, low) nello stack decrement ando SP di 2.
    pub fn push_r16(&mut self, high: u8, low: u8, banking: &mut Banking, mmu: &mut Mmu) {
        self.sp = self.sp.wrapping_sub(1);
        mmu.write_byte(banking, self.sp, high);
        self.sp = self.sp.wrapping_sub(1);
        mmu.write_byte(banking, self.sp, low);
        self.cycles = self.cycles.wrapping_add(16);
    }

    /// PUSH AF - Carica A e flag F nello stack
    ///
    /// Salva A come byte alto e F (con solo i 4 bit significativi) come byte basso.
    /// Decrementa SP di 2.
    pub fn push_af(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let high = self.a;
        let low = self.f & 0xF0; // I flag sono solo i 4 bit più significativi
        self.push_r16(high, low, banking, mmu);
    }

    /// RET cond - Ritorno condizionato dalla subroutine
    ///
    /// Se la condizione è vera, esegue un ritorno incondizionato.
    /// Se la condizione è falsa, continua l'esecuzione sequenziale.
    pub fn ret_cond(&mut self, _condition: bool, banking: &mut Banking, mmu: &mut Mmu) {
        if _condition {
            self.ret_incond(banking, mmu);
            self.cycles = self.cycles.wrapping_add(4);
        } else {
            self.cycles = self.cycles.wrapping_add(8);
        }
    }

    /// RET - Ritorno incondizionato dalla subroutine
    ///
    /// Estrae l'indirizzo di ritorno dallo stack e lo carica in PC.
    pub fn ret_incond(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        self.pc = self.pop_r16(banking, mmu);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// RETI - Ritorno da interrupt
    ///
    /// Esegue un ritorno dalla routine di interrupt e abilita gli interrupt (IME = true).
    pub fn reti(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        self.ret_incond(banking, mmu);
        self.ime = true; // Abilita gli interrupt dopo il ritorno
    }

    /// JP cond, n16 - Salto condizionato a indirizzo assoluto
    ///
    /// Se la condizione è vera, salta all'indirizzo n16.
    /// Se la condizione è falsa, continua l'esecuzione sequenziale.
    pub fn jp_cond(&mut self, _condition: bool, banking: &mut Banking, mmu: &mut Mmu) {
        let addr_low = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);
        let addr_high = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);
        let addr = (addr_high as u16) << 8 | (addr_low as u16);

        if _condition {
            self.pc = addr;
            self.cycles = self.cycles.wrapping_add(16); // Cicli extra se la condizione è vera
        } else {
            self.cycles = self.cycles.wrapping_add(12); // Cicli totali se la condizione è falsa
        }
    }

    /// JP n16 - Salto incondizionato a indirizzo assoluto
    ///
    /// Carica in PC l'indirizzo immediato n16 (letto da memoria).
    pub fn jp(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let addr_low = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);
        let addr_high = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);
        let addr = (addr_high as u16) << 8 | (addr_low as u16);
        self.pc = addr;
        self.cycles = self.cycles.wrapping_add(16);
    }

    /// JP (HL) - Salto all'indirizzo in HL
    ///
    /// Carica in PC il valore del registro HL.
    pub fn jp_hl(&mut self) {
        let addr = get_u16register!(self, self.h, self.l);
        self.pc = addr;
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// CALL cond, n16 - Chiamata condizionata a subroutine
    ///
    /// Se la condizione è vera, esegue una CALL incondizionata.
    /// Se la condizione è falsa, salta l'indirizzo di 2 byte.
    pub fn call_cond(&mut self, _condition: bool, banking: &mut Banking, mmu: &mut Mmu) {
        if _condition {
            self.call(banking, mmu);
        } else {
            self.pc = self.pc.wrapping_add(2); // Salta l'indirizzo di 2 byte
            self.cycles = self.cycles.wrapping_add(12); // Cicli totali se la condizione è falsa
        }
    }

    /// CALL n16 - Chiamata a subroutine
    ///
    /// Carica l'indirizzo di ritorno (PC attuale) nello stack e salta a n16.
    /// Decrementa SP di 2.
    pub fn call(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let addr_low = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);
        let addr_high = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);
        let addr = (addr_high as u16) << 8 | (addr_low as u16);

        // Push the current PC onto the stack
        let pc_high = (self.pc >> 8) as u8;
        let pc_low = (self.pc & 0xFF) as u8;
        self.push_r16(pc_high, pc_low, banking, mmu);

        // Jump to the target address
        self.pc = addr;
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// RST n - Restart a indirizzo fisso
    ///
    /// Salva PC nello stack e salta a un indirizzo fisso (0x00, 0x08, 0x10, ecc.).
    /// Utilizzato per interrupt vectored.
    pub fn rst(&mut self, target_addr: u16, banking: &mut Banking, mmu: &mut Mmu) {
        let pc_high = (self.pc >> 8) as u8;
        let pc_low = (self.pc & 0xFF) as u8;
        self.push_r16(pc_high, pc_low, banking, mmu);
        self.pc = target_addr;
    }

    /// ADD A, n8 - Addizione con byte immediato
    ///
    /// Addiziona un byte immediato (dal PC) ad A.
    /// Incrementa PC di 1. Modifica i flag Z, N, H, e C.
    pub fn add_a_imm8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let value = mmu.read_byte(banking, self.pc);
        self.alu_add(value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// ADC A, n8 - Addizione con Carry dal byte immediato
    ///
    /// Addiziona un byte immediato e il flag Carry ad A.
    /// Incrementa PC di 1. Modifica i flag Z, N, H, e C.
    pub fn adc_a_imm8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let value = mmu.read_byte(banking, self.pc);
        self.alu_adc(value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// SUB A, n8 - Sottrazione con byte immediato
    ///
    /// Sottrae un byte immediato da A.
    /// Incrementa PC di 1. Modifica i flag Z, N, H, e C.
    pub fn sub_a_imm8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let value = mmu.read_byte(banking, self.pc);
        self.alu_sub(value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// SBC A, n8 - Sottrazione con Carry dal byte immediato
    ///
    /// Sottrae un byte immediato e il flag Carry da A.
    /// Incrementa PC di 1. Modifica i flag Z, N, H, e C.
    pub fn sbc_a_imm8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let value = mmu.read_byte(banking, self.pc);
        self.alu_sbc(value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// ADD SP, e8 - Addizione a Stack Pointer con offset 8-bit con segno
    ///
    /// Addiziona un offset relativo (8-bit con segno) a SP.
    /// Azzera Z e N. Modifica H e C basati sull'operazione a 8-bit.
    /// Incrementa PC di 1.
    pub fn add_sp_e8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let raw_offset = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);

        let offset = raw_offset as i8 as i32;
        let sp = self.sp as i32;
        let result = sp.wrapping_add(offset);

        // I flag Z ed N sono sempre false
        self.set_flag_z(false);
        self.set_flag_n(false);

        // H e C si calcolano sui primi 4 e 8 bit dell'operazione a 8-bit (raw_offset)
        let sp_low = self.sp as u32;
        let byte_val = raw_offset as u32;
        self.set_flag_h(((sp_low & 0x0F) + (byte_val & 0x0F)) > 0x0F);
        self.set_flag_c(((sp_low & 0xFF) + (byte_val & 0xFF)) > 0xFF);

        self.sp = result as u16;
        self.cycles = self.cycles.wrapping_add(16);
    }

    /// AND A, n8 - AND logico con byte immediato
    ///
    /// Esegue AND logico tra A e un byte immediato.
    /// Incrementa PC di 1. Imposta H=true, azzera N, H, C. Modifica Z.
    pub fn and_a_imm8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let value = mmu.read_byte(banking, self.pc);
        self.alu_and(value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// XOR A, n8 - XOR logico con byte immediato
    ///
    /// Esegue XOR logico tra A e un byte immediato.
    /// Incrementa PC di 1. Azzera i flag N, H, C. Modifica Z.
    pub fn xor_a_imm8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let value = mmu.read_byte(banking, self.pc);
        self.alu_xor(value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// OR A, n8 - OR logico con byte immediato
    ///
    /// Esegue OR logico tra A e un byte immediato.
    /// Incrementa PC di 1. Azzera i flag N, H, C. Modifica Z.
    pub fn or_a_imm8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let value = mmu.read_byte(banking, self.pc);
        self.alu_or(value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// CP A, n8 - Confronta A con byte immediato
    ///
    /// Confronta A con un byte immediato (sottrazione senza salvataggio).
    /// Incrementa PC di 1. Modifica i flag Z, N, H, e C.
    pub fn cp_a_imm8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let value = mmu.read_byte(banking, self.pc);
        self.alu_cp(value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// LDH (n8), A - Scrive A in memoria alta (0xFF00+n8)
    ///
    /// Scrive A all'indirizzo 0xFF00 + offset (n8).
    /// Usa la memoria alta I/O (0xFF00-0xFFFF).
    /// Incrementa PC di 1.
    pub fn ldh_mem8_a(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let offset = mmu.read_byte(banking, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        mmu.write_byte(banking, 0xFF00 | offset, self.a);
        self.cycles = self.cycles.wrapping_add(12);
    }

    /// LDH A, (n8) - Legge da memoria alta (0xFF00+n8) in A
    ///
    /// Legge un byte da 0xFF00 + offset (n8) in A.
    /// Usa la memoria alta I/O (0xFF00-0xFFFF).
    /// Incrementa PC di 1.
    pub fn ldh_a_mem8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let offset = mmu.read_byte(banking, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        self.a = mmu.read_byte(banking, 0xFF00 | offset);
        self.cycles = self.cycles.wrapping_add(12);
    }

    /// LD (C), A - Scrive A in memoria alta (0xFF00+C)
    ///
    /// Scrive A all'indirizzo 0xFF00 + C.
    /// Versione dinamica di LDH usando il registro C.
    pub fn ld_mem_c_a(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let addr = 0xFF00 | (self.c as u16);
        mmu.write_byte(banking, addr, self.a);
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// LD A, (C) - Legge da memoria alta (0xFF00+C) in A
    ///
    /// Legge un byte da 0xFF00 + C in A.
    /// Versione dinamica di LDH usando il registro C.
    pub fn ld_a_mem_c(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let addr = 0xFF00 | (self.c as u16);
        self.a = mmu.read_byte(banking, addr);
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// LD (n16), A - Scrive A in memoria assoluta (n16)
    ///
    /// Legge un indirizzo a 16-bit dal PC e scrive A a quell'indirizzo.
    /// Incrementa PC di 2.
    pub fn ld_mem16_a(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let low = mmu.read_byte(banking, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let high = mmu.read_byte(banking, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let addr = low | (high << 8);
        mmu.write_byte(banking, addr, self.a);
        self.cycles = self.cycles.wrapping_add(16);
    }

    /// LD A, (n16) - Legge da memoria assoluta (n16) in A
    ///
    /// Legge un indirizzo a 16-bit dal PC e carica il valore da quell'indirizzo in A.
    /// Incrementa PC di 2.
    pub fn ld_a_mem16(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let low = mmu.read_byte(banking, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let high = mmu.read_byte(banking, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let addr = low | (high << 8);
        self.a = mmu.read_byte(banking, addr);
        self.cycles = self.cycles.wrapping_add(16);
    }

    /// LD HL, SP+e8 - Carica HL = SP + offset 8-bit con segno
    ///
    /// Calcola SP + offset (8-bit con segno) e carica il risultato in HL.
    /// Azzera Z e N. Modifica H e C basati sull'operazione a 8-bit.
    /// Incrementa PC di 1.
    pub fn ld_hl_sp_e8(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let raw_offset = mmu.read_byte(banking, self.pc);
        self.pc = self.pc.wrapping_add(1);

        let offset = raw_offset as i8 as i32;
        let sp = self.sp as i32;
        let result = sp.wrapping_add(offset);

        self.set_flag_z(false);
        self.set_flag_n(false);

        let sp_low = self.sp as u32;
        let byte_val = raw_offset as u32;
        self.set_flag_h(((sp_low & 0x0F) + (byte_val & 0x0F)) > 0x0F);
        self.set_flag_c(((sp_low & 0xFF) + (byte_val & 0xFF)) > 0xFF);

        let final_res = result as u16;
        self.h = (final_res >> 8) as u8;
        self.l = (final_res & 0xFF) as u8;

        self.cycles = self.cycles.wrapping_add(12);
    }

    /// LD SP, HL - Carica Stack Pointer da HL
    ///
    /// Copia il valore di HL in SP.
    pub fn ld_sp_hl(&mut self) {
        let hl = get_u16register!(self, self.h, self.l);
        self.sp = hl;
        self.cycles = self.cycles.wrapping_add(8);
    }

    /// DI - Disabilita Interrupt
    ///
    /// Azzera il flag IME (Interrupt Master Enable).
    /// Gli interrupt rimangono disabilitati finché non viene eseguita EI.
    pub fn di(&mut self) {
        self.ime = false;
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// EI - Abilita Interrupt
    ///
    /// Imposta il flag IME (Interrupt Master Enable).
    /// Gli interrupt vengono abilitati dopo l'istruzione successiva.
    pub fn ei(&mut self) {
        self.ime = true;
        self.cycles = self.cycles.wrapping_add(4);
    }

    /// Dispatcher per istruzioni prefissate con 0xCB
    ///
    /// Legge il prossimo byte come opcode prefissato e lo elabora.
    /// Le istruzioni prefissate includono rotazioni, shift, bit operations, ecc.
    pub fn cb(&mut self, banking: &mut Banking, mmu: &mut Mmu) {
        let opcode = mmu.read_byte(banking, self.pc);
        self.cb_prefixed(opcode, banking, mmu);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
}
