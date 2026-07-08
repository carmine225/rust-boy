use crate::cpu::Cpu;
use crate::get_u16register;
use crate::mmu::Mmu;

impl Cpu {
    pub fn nop(&mut self) {
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }
    pub fn stop(&mut self) {
        self.pc = self.pc.wrapping_add(2);
        self.cycles = self.cycles.wrapping_add(4);
        self.stopped = true; // Aggiungi un flag per indicare che la CPU è in stato di stop
    }
    pub fn halt(&mut self) {
        self.pc = self.pc.wrapping_add(1);
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
    pub fn daa(&mut self) {
        let a = self.a as u16;
        let mut correction = 0;

        if !self.get_flag_n() {
            // Caso 1: ADDIZIONE
            if self.get_flag_h() || (a & 0x0F) > 0x09 {
                correction |= 0x06;
            }
            if self.get_flag_c() || a > 0x9F {
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
    }
    pub fn cpl(&mut self) {
        self.a = !self.a;
        self.set_flag_n(true);
        self.set_flag_h(true);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }
    pub fn scf(&mut self) {
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(true);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }
    pub fn ccf(&mut self) {
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(!self.get_flag_c());
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }
    pub fn rlca(&mut self) {
        let mut a = self.a;
        let bit_7 = (self.a >> 7) & 0x01;
        a = (a << 1) | bit_7;
        self.a = a;
        self.set_flag_z(false);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_7 == 1);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }
    pub fn rrca(&mut self) {
        let mut a = self.a;
        let bit_0 = a & 0x01;
        a = (a >> 1) | (bit_0 << 7);
        self.a = a;
        self.set_flag_z(false);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_0 == 1);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }
    pub fn rla(&mut self) {
        let bit_7 = (self.a >> 7) & 0x01;
        let carry = if self.get_flag_c() { 1 } else { 0 };
        self.a = (self.a << 1) | carry;
        self.set_flag_z(false);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_7 == 1);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }
    pub fn rra(&mut self) {
        let bit_0 = self.a & 0x01;
        let carry = if self.get_flag_c() { 1 } else { 0 };
        self.a = (self.a >> 1) | (carry << 7);
        self.set_flag_z(false);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_0 == 1);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
    }
    pub fn dec_hl_mem(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(hl);
        let result = value.wrapping_sub(1);
        mmu.write_byte(hl, result);
        self.set_flag_z(result == 0);
        self.set_flag_n(true);
        self.set_flag_h((value & 0x0F) == 0x00);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(12);
    }
    pub fn ld_hl_mem_imm8(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = self.fetch_byte(mmu, self.pc + 1);
        mmu.write_byte(hl, value);
        self.pc = self.pc.wrapping_add(2);
        self.cycles = self.cycles.wrapping_add(12);
    }
    pub fn inc_hl_mem(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(hl);
        let result = value.wrapping_add(1);
        mmu.write_byte(hl, result);
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h((value & 0x0F) == 0x0F);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(12);
    }
    pub fn add_hl_r16(&mut self, value: u16) {
        let hl_32 = get_u16register!(self, self.h, self.l) as u32;
        let val_32 = value as u32;
        let result_32 = hl_32 + val_32;
        self.set_flag_n(false); // Sempre false nelle addizioni
        let half_carry = ((hl_32 & 0x0FFF) + (val_32 & 0x0FFF)) > 0x0FFF;
        self.set_flag_h(half_carry);
        self.set_flag_c(result_32 > 0xFFFF);
        self.h = ((result_32 >> 8) & 0xFF) as u8;
        self.l = (result_32 & 0xFF) as u8;
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn ld_mem_r16_a(&mut self, mmu: &mut Mmu, value: u16) {
        mmu.write_byte(value, self.a);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn ld_a_mem_r16(&mut self, mmu: &mut Mmu, value: u16) {
        self.a = mmu.read_byte(value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn ld_mem16_sp(&mut self, mmu: &mut Mmu) {
        let low = mmu.read_byte(self.pc + 1) as u16;
        let high = mmu.read_byte(self.pc + 2) as u16;
        let addr = low | (high << 8);
        mmu.write_byte(addr, self.sp as u8);
        mmu.write_byte(addr.wrapping_add(1), (self.sp >> 8) as u8);
        self.pc = self.pc.wrapping_add(3);
        self.cycles = self.cycles.wrapping_add(20);
    }
    pub fn ld_hl_inc_a(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        mmu.write_byte(hl, self.a);
        let hl = hl.wrapping_add(1);
        self.h = (hl >> 8) as u8;
        self.l = (hl & 0xFF) as u8;
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8); // Aggiorna il conteggio dei cicli in base all'operazione
    }
    pub fn ld_a_hl_inc(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        self.a = mmu.read_byte(hl);
        let hl = hl.wrapping_add(1);
        self.h = (hl >> 8) as u8;
        self.l = (hl & 0xFF) as u8;
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8); // Aggiorna il conteggio dei cicli in base all'operazione
    }
    pub fn ld_hl_dec_a(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        mmu.write_byte(hl, self.a);
        let hl = hl.wrapping_sub(1);
        self.h = (hl >> 8) as u8;
        self.l = (hl & 0xFF) as u8;
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn ld_a_hl_dec(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        self.a = mmu.read_byte(hl);
        let hl = hl.wrapping_sub(1);
        self.h = (hl >> 8) as u8;
        self.l = (hl & 0xFF) as u8;
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn ld_mem_hl_r8(&mut self, mmu: &mut Mmu, value: u8) {
        let hl = get_u16register!(self, self.h, self.l);
        mmu.write_byte(hl, value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8); // Aggiorna il conteggio dei cicli in base all'operazione
    }
    pub fn ld_r8_mem_hl(&mut self, mmu: &mut Mmu) -> u8 {
        let hl = get_u16register!(self, self.h, self.l);
        let data = mmu.read_byte(hl);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
        data
    }
    pub fn alu_add_mem_hl(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(hl);
        let result = (self.a as u16) + (value as u16);
        let final_result = result as u8;
        self.set_flag_z(final_result == 0);
        self.set_flag_n(false);
        let half_carry = ((self.a & 0x0F) + (value & 0x0F)) > 0x0F;
        self.set_flag_h(half_carry);
        self.set_flag_c(result > 0xFF);
        self.a = final_result;
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn alu_sub_mem_hl(&mut self, _mmu: &mut Mmu) {
        todo!("Implement alu_sub_mem_hl")
    }
    pub fn alu_cp_mem_hl(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(hl);
        let result = (self.a as u16).wrapping_sub(value as u16);
        let final_result = result as u8;
        self.set_flag_z(final_result == 0);
        self.set_flag_n(true);
        let half_carry = (self.a & 0x0F) < (value & 0x0F);
        self.set_flag_h(half_carry);
        self.set_flag_c(self.a < value);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn alu_adc_mem_hl(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(hl);
        let carry = if self.get_flag_c() { 1 } else { 0 };
        let final_result = self.a.wrapping_add(value).wrapping_add(carry);
        self.set_flag_z(final_result == 0);
        self.set_flag_n(false);
        let half_carry = ((self.a & 0x0F) + (value & 0x0F) + carry) > 0x0F;
        self.set_flag_h(half_carry);
        let overflow_check = (self.a as u16) + (value as u16) + (carry as u16);
        self.set_flag_c(overflow_check > 0xFF);
        self.a = final_result;
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn alu_sbc_mem_hl(&mut self, mmu: &mut Mmu) {
        let hl_addr = ((self.h as u16) << 8) | (self.l as u16);
        let value = mmu.read_byte(hl_addr);
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
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn alu_and_mem_hl(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(hl);
        self.a &= value;
        self.set_flag_z(self.a == 0);
        self.set_flag_n(false);
        self.set_flag_h(true);
        self.set_flag_c(false);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn alu_xor_mem_hl(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(hl);
        self.a ^= value;
        self.set_flag_z(self.a == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn alu_or_mem_hl(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let value = mmu.read_byte(hl);
        self.a |= value;
        self.set_flag_z(self.a == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn ld_r16_imm16(&mut self, value: &mut u16, mmu: &mut Mmu) {
        self.pc = self.pc.wrapping_add(1);
        let value_low = mmu.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let value_high = mmu.read_byte(self.pc);
        let final_value = (value_high as u16) << 8 | (value_low as u16);
        *value = final_value;
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(12);
    }
    pub fn inc_r16(&mut self, value: &mut u16) {
        *value = value.wrapping_add(1);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn dec_r16(&mut self, value: &mut u16) {
        *value = value.wrapping_sub(1);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn inc_r8(&mut self, value: &mut u8) -> u8 {
        let result = value.wrapping_add(1);
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h((*value & 0x0F) == 0x0F);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
        result
    }
    pub fn dec_r8(&mut self, value: &mut u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.set_flag_z(result == 0);
        self.set_flag_n(true);
        self.set_flag_h((*value & 0x0F) == 0x00);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
        result
    }
    pub fn ld_r8_imm8(&mut self, value: &mut u8, mmu: &mut Mmu) {
        self.pc = self.pc.wrapping_add(1);
        *value = mmu.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(8);
    }
    pub fn jr_cond(&mut self, condition: bool, mmu: &Mmu) {
        self.pc = self.pc.wrapping_add(1);
        let offset_raw = mmu.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);

        if condition {
            let offset = offset_raw as i8;
            self.pc = ((self.pc as i32) + (offset as i32)) as u16;
            self.cycles = self.cycles.wrapping_add(12);
        } else {
            self.cycles = self.cycles.wrapping_add(8);
        }
    }
    pub fn ld_r8_r8(&mut self, src: u8) -> u8 {
        self.pc = self.pc.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(4);
        src
    }

    pub fn alu_op(&mut self, _op: &str, _reg: &str) {
        todo!("Implement alu_op")
    }

    pub fn pop_r16(&mut self, _reg: Reg16) {
        todo!("Implement pop_r16")
    }

    pub fn push_r16(&mut self, _reg: Reg16) {
        todo!("Implement push_r16")
    }

    pub fn ret_cond(&mut self, _condition: bool) {
        todo!("Implement ret_cond")
    }

    pub fn ret_inconditional(&mut self) {
        todo!("Implement ret_inconditional")
    }

    pub fn reti(&mut self) {
        todo!("Implement reti")
    }

    pub fn jp_cond(&mut self, _condition: bool) {
        todo!("Implement jp_cond")
    }

    pub fn jp_inconditional(&mut self) {
        todo!("Implement jp_inconditional")
    }

    pub fn jp_hl(&mut self) {
        todo!("Implement jp_hl")
    }

    pub fn call_cond(&mut self, _condition: bool) {
        todo!("Implement call_cond")
    }

    pub fn call_inconditional(&mut self) {
        todo!("Implement call_inconditional")
    }

    pub fn rst(&mut self, _target: u16) {
        todo!("Implement rst")
    }

    pub fn add_a_imm8(&mut self) {
        todo!("Implement add_a_imm8")
    }

    pub fn adc_a_imm8(&mut self) {
        todo!("Implement adc_a_imm8")
    }

    pub fn sub_a_imm8(&mut self) {
        todo!("Implement sub_a_imm8")
    }

    pub fn sbc_a_imm8(&mut self) {
        todo!("Implement sbc_a_imm8")
    }

    pub fn and_a_imm8(&mut self) {
        todo!("Implement and_a_imm8")
    }

    pub fn xor_a_imm8(&mut self) {
        todo!("Implement xor_a_imm8")
    }

    pub fn or_a_imm8(&mut self) {
        todo!("Implement or_a_imm8")
    }

    pub fn cp_a_imm8(&mut self) {
        todo!("Implement cp_a_imm8")
    }

    pub fn ldh_mem8_a(&mut self) {
        todo!("Implement ldh_mem8_a")
    }

    pub fn ldh_a_mem8(&mut self) {
        todo!("Implement ldh_a_mem8")
    }

    pub fn ld_mem_c_a(&mut self) {
        todo!("Implement ld_mem_c_a")
    }

    pub fn ld_a_mem_c(&mut self) {
        todo!("Implement ld_a_mem_c")
    }

    pub fn ld_mem16_a(&mut self) {
        todo!("Implement ld_mem16_a")
    }

    pub fn ld_a_mem16(&mut self) {
        todo!("Implement ld_a_mem16")
    }

    pub fn add_sp_e8(&mut self) {
        todo!("Implement add_sp_e8")
    }

    pub fn ld_hl_sp_e8(&mut self) {
        todo!("Implement ld_hl_sp_e8")
    }

    pub fn ld_sp_hl(&mut self) {
        todo!("Implement ld_sp_hl")
    }

    pub fn di(&mut self) {
        todo!("Implement di")
    }

    pub fn ei(&mut self) {
        todo!("Implement ei")
    }
}
