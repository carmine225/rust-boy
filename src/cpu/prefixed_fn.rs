use crate::cpu::Cpu;
use crate::get_u16register;
use crate::mmu::Mmu;

impl Cpu {
    // ==========================================
    // 1. RLC
    // ==========================================
    pub fn rlc_r8(&mut self, val: u8) -> u8 {
        let bit_7 = (val >> 7) & 0x01;
        let res = (val << 1) | bit_7;
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_7 == 1);

        res
    }

    pub fn rlc_hl_mem(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        let res = self.rlc_r8(val);
        mmu.write_byte(hl, res);
        self.cycles = self.cycles.wrapping_add(8); // 16 cicli totali
    }

    // ==========================================
    // 2. RRC
    // ==========================================
    pub fn rrc_r8(&mut self, val: u8) -> u8 {
        let bit_0 = val & 0x01;
        let res = (val >> 1) | (bit_0 << 7);
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_0 == 1);
        res
    }

    pub fn rrc_hl_mem(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        let res = self.rrc_r8(val);
        mmu.write_byte(hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 3. RL
    // ==========================================
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

    pub fn rl_hl_mem(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        let res = self.rl_r8(val);
        mmu.write_byte(hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 4. RR
    // ==========================================
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

    pub fn rr_hl_mem(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        let res = self.rr_r8(val);
        mmu.write_byte(hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 5. SLA
    // ==========================================
    pub fn sla_r8(&mut self, val: u8) -> u8 {
        let bit_7 = (val >> 7) & 0x01;
        let res = val << 1;
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_7 == 1);
        res
    }

    pub fn sla_hl_mem(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        let res = self.sla_r8(val);
        mmu.write_byte(hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 6. SRA
    // ==========================================
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

    pub fn sra_hl_mem(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        let res = self.sra_r8(val);
        mmu.write_byte(hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 7. SWAP
    // ==========================================
    pub fn swap_r8(&mut self, val: u8) -> u8 {
        let res = (val >> 4) | (val << 4);
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
        res
    }

    pub fn swap_hl_mem(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        let res = self.swap_r8(val);
        mmu.write_byte(hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 8. SRL
    // ==========================================
    pub fn srl_r8(&mut self, val: u8) -> u8 {
        let bit_0 = val & 0x01;
        let res = val >> 1;
        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(bit_0 == 1);
        res
    }

    pub fn srl_hl_mem(&mut self, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        let res = self.srl_r8(val);
        mmu.write_byte(hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 9. BIT
    // ==========================================
    pub fn bit_b_r8(&mut self, bit: u8, val: u8) {
        let is_bit_set = (val & (1 << bit)) != 0;
        self.set_flag_z(!is_bit_set);
        self.set_flag_n(false);
        self.set_flag_h(true);
    }

    pub fn bit_b_hl_mem(&mut self, bit: u8, mmu: &Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        self.bit_b_r8(bit, val);
        self.cycles = self.cycles.wrapping_add(4); // 12 cicli totali con cb()
    }

    // ==========================================
    // 10. RES
    // ==========================================
    pub fn res_b_r8(&mut self, bit: u8, val: u8) -> u8 {
        val & !(1 << bit)
    }

    pub fn res_b_hl_mem(&mut self, bit: u8, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        let res = self.res_b_r8(bit, val);
        mmu.write_byte(hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }

    // ==========================================
    // 11. SET
    // ==========================================
    pub fn set_b_r8(&mut self, bit: u8, val: u8) -> u8 {
        val | (1 << bit)
    }

    pub fn set_b_hl_mem(&mut self, bit: u8, mmu: &mut Mmu) {
        let hl = get_u16register!(self, self.h, self.l);
        let val = mmu.read_byte(hl);
        let res = self.set_b_r8(bit, val);
        mmu.write_byte(hl, res);
        self.cycles = self.cycles.wrapping_add(8);
    }
}
