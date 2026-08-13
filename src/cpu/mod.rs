// central processing unit

use crate::get_u16register;
use crate::mmu::Mmu;
use crate::set_u16register;
mod prefixed_fn;
mod unprefixed_fn;

const FLAG_Z_MASK: u8 = 0x80; // 1000 0000
const FLAG_N_MASK: u8 = 0x40; // 0100 0000
const FLAG_H_MASK: u8 = 0x20; // 0010 0000
const FLAG_C_MASK: u8 = 0x10; // 0001 0000

pub struct Cpu {
    a: u8, // Accumulator
    f: u8, // Flags
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16, // Stack Pointer
    pc: u16, // Program Counter
    cycles: u64,
    stopped: bool,
    halted: bool,
    halt_bug_triggered: bool,
    ime: bool, //Interrupt Master Enable
    interrupt_enable: u8,
    interrupt_flag: u8,
}
impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0xFFFE, // Stack Pointer starts at the end of memory
            pc: 0x0100, // Program Counter starts at the beginning of the cartridge
            cycles: 0,
            stopped: false,
            halted: false, // indicates whether the CPU is halted (waiting for an interrupt)
            halt_bug_triggered: false, // indicates whether the halt bug has been triggered
            ime: false,    // Interrupt Master Enable flag
            interrupt_enable: 0,
            interrupt_flag: 0,
        }
    }
    // --- HELPER PER IL FLAG Z (ZERO) ---
    fn get_flag_z(&self) -> bool {
        (self.f & FLAG_Z_MASK) != 0
    }
    fn set_flag_z(&mut self, value: bool) {
        if value {
            self.f |= FLAG_Z_MASK; // Imposta il bit a 1
        } else {
            self.f &= !FLAG_Z_MASK; // Azzera il bit (0)
        }
    }

    // --- HELPER PER IL FLAG N (SUBTRACTION) ---
    fn get_flag_n(&self) -> bool {
        (self.f & FLAG_N_MASK) != 0
    }
    fn set_flag_n(&mut self, value: bool) {
        if value {
            self.f |= FLAG_N_MASK;
        } else {
            self.f &= !FLAG_N_MASK;
        }
    }

    // --- HELPER PER IL FLAG H (HALF-CARRY) ---
    fn get_flag_h(&self) -> bool {
        (self.f & FLAG_H_MASK) != 0
    }
    fn set_flag_h(&mut self, value: bool) {
        if value {
            self.f |= FLAG_H_MASK;
        } else {
            self.f &= !FLAG_H_MASK;
        }
    }

    // --- HELPER PER IL FLAG C (CARRY) ---
    fn get_flag_c(&self) -> bool {
        (self.f & FLAG_C_MASK) != 0
    }
    fn set_flag_c(&mut self, value: bool) {
        if value {
            self.f |= FLAG_C_MASK;
        } else {
            self.f &= !FLAG_C_MASK;
        }
    }

    pub fn step(&mut self, mmu: &mut Mmu) {
        let opcode = self.fetch_byte(mmu, self.pc);
        match opcode {
            // ==========================================
            // ISTRUZIONI DI CONTROLLO E SPECIALI
            // ==========================================
            0x00 => self.nop(),
            0x10 => self.stop(),
            0x76 => self.halt(),
            0x27 => self.daa(),
            0x2F => self.cpl(),
            0x37 => self.scf(),
            0x3F => self.ccf(),

            // ==========================================
            // ROTAZIONI VELOCI DELL'ACCUMULATORE
            // ==========================================
            0x07 => self.rlca(),
            0x0F => self.rrca(),
            0x17 => self.rla(),
            0x1F => self.rra(),

            // ==========================================
            // CARICAMENTI IMMEDIATI A 16 BIT (LD r16, n16)
            // ==========================================
            0x01 => {
                let bc = self.ld_r16_imm16(mmu);
                set_u16register!(self, self.b, self.c, bc);
            }
            0x11 => {
                let de = self.ld_r16_imm16(mmu);
                set_u16register!(self, self.d, self.e, de);
            }
            0x21 => {
                let hl = self.ld_r16_imm16(mmu);
                set_u16register!(self, self.h, self.l, hl);
            }
            0x31 => self.sp = self.ld_r16_imm16(mmu),

            // ==========================================
            // INCREMENTI E DECREMENTI A 16 BIT
            // ==========================================
            0x03 => {
                let mut bc = get_u16register!(self, self.b, self.c);
                bc = self.inc_r16(bc);
                set_u16register!(self, self.b, self.c, bc);
            }
            0x13 => {
                let mut de = get_u16register!(self, self.d, self.e);
                de = self.inc_r16(de);
                set_u16register!(self, self.d, self.e, de);
            }
            0x23 => {
                let mut hl = get_u16register!(self, self.h, self.l);
                hl = self.inc_r16(hl);
                set_u16register!(self, self.h, self.l, hl);
            }
            0x33 => self.sp = self.inc_r16(self.sp),

            0x0B => {
                let mut bc = get_u16register!(self, self.b, self.c);
                bc = self.dec_r16(bc);
                set_u16register!(self, self.b, self.c, bc);
            }
            0x1B => {
                let mut de = get_u16register!(self, self.d, self.e);
                de = self.dec_r16(de);
                set_u16register!(self, self.d, self.e, de);
            }
            0x2B => {
                let mut hl = get_u16register!(self, self.h, self.l);
                hl = self.dec_r16(hl);
                set_u16register!(self, self.h, self.l, hl);
            }
            0x3B => self.sp = self.dec_r16(self.sp),

            // ==========================================
            // INCREMENTI E DECREMENTI A 8 BIT
            // ==========================================
            0x04 => self.b = self.inc_r8(self.b),
            0x0C => self.c = self.inc_r8(self.c),
            0x14 => self.d = self.inc_r8(self.d),
            0x1C => self.e = self.inc_r8(self.e),
            0x24 => self.h = self.inc_r8(self.h),
            0x2C => self.l = self.inc_r8(self.l),
            0x34 => self.inc_hl_mem(mmu), // Speciale: incrementa la memoria puntata da (HL)
            0x3C => self.a = self.inc_r8(self.a),

            0x05 => self.b = self.dec_r8(self.b),
            0x0D => self.c = self.dec_r8(self.c),
            0x15 => self.d = self.dec_r8(self.d),
            0x1D => self.e = self.dec_r8(self.e),
            0x25 => self.h = self.dec_r8(self.h),
            0x2D => self.l = self.dec_r8(self.l),
            0x35 => self.dec_hl_mem(mmu), // Speciale: decrementa la memoria puntata da (HL)
            0x3D => self.a = self.dec_r8(self.a),

            // ==========================================
            // CARICAMENTI IMMEDIATI A 8 BIT (LD r8, n8)
            // ==========================================
            0x06 => self.b = self.ld_r8_imm8(mmu),
            0x0E => self.c = self.ld_r8_imm8(mmu),
            0x16 => self.d = self.ld_r8_imm8(mmu),
            0x1E => self.e = self.ld_r8_imm8(mmu),
            0x26 => self.h = self.ld_r8_imm8(mmu),
            0x2E => self.l = self.ld_r8_imm8(mmu),
            0x36 => self.ld_hl_mem_imm8(mmu), // LD (HL), n8
            0x3E => self.a = self.ld_r8_imm8(mmu),

            // ==========================================
            // OPERAZIONI ARITMETICHE SU HL (ADD HL, r16)
            // ==========================================
            0x09 => {
                let bc = get_u16register!(self, self.b, self.c);
                self.add_hl_r16(bc);
            }
            0x19 => {
                let de = get_u16register!(self, self.d, self.e);
                self.add_hl_r16(de);
            }
            0x29 => {
                let hl = get_u16register!(self, self.h, self.l);
                self.add_hl_r16(hl);
            }
            0x39 => {
                let sp = self.sp;
                self.add_hl_r16(sp);
            }

            // ==========================================
            // CARICAMENTI INDIRETTI (MEMORIA/ACCUMULATORE)
            // ==========================================
            0x02 => {
                let bc = get_u16register!(self, self.b, self.c);
                self.ld_mem_r16_a(mmu, bc)
            } // LD (BC), A
            0x12 => {
                let de = get_u16register!(self, self.d, self.e);
                self.ld_mem_r16_a(mmu, de)
            } // LD (DE), A
            0x0A => {
                let bc = get_u16register!(self, self.b, self.c);
                self.ld_a_mem_r16(mmu, bc)
            } // LD A, (BC)
            0x1A => {
                let de = get_u16register!(self, self.d, self.e);
                self.ld_a_mem_r16(mmu, de)
            } // LD A, (DE)
            0x08 => self.ld_mem16_sp(mmu), // LD (n16), SP

            // Auto-incremento / decremento HL
            0x22 => self.ld_hl_inc_a(mmu), // LD (HL+), A
            0x2A => self.ld_a_hl_inc(mmu), // LD A, (HL+)
            0x32 => self.ld_hl_dec_a(mmu), // LD (HL-), A
            0x3A => self.ld_a_hl_dec(mmu), // LD A, (HL-)

            // ==========================================
            // SALTI RELATIVI CONDIZIONATI (JR)
            // ==========================================
            0x18 => self.jr_cond(true, mmu),
            0x20 => self.jr_cond(!self.get_flag_z(), mmu), // JR NZ, e8
            0x28 => self.jr_cond(self.get_flag_z(), mmu),  // JR Z, e8
            0x30 => self.jr_cond(!self.get_flag_c(), mmu), // JR NC, e8
            0x38 => self.jr_cond(self.get_flag_c(), mmu),  // JR C, e8

            // ==========================================
            // BLOCCO LD R8, R8 (0x40 - 0x7F)
            // Esplicitati riga per riga per massima efficienza
            // ==========================================
            // Destinazione B
            0x40 => self.b = self.ld_r8_r8(self.b),
            0x41 => self.b = self.ld_r8_r8(self.c),
            0x42 => self.b = self.ld_r8_r8(self.d),
            0x43 => self.b = self.ld_r8_r8(self.e),
            0x44 => self.b = self.ld_r8_r8(self.h),
            0x45 => self.b = self.ld_r8_r8(self.l),
            0x46 => self.b = self.ld_r8_mem_hl(mmu), // LD B, (HL)
            0x47 => self.b = self.ld_r8_r8(self.a),

            // Destinazione C
            0x48 => self.c = self.ld_r8_r8(self.b),
            0x49 => self.c = self.ld_r8_r8(self.c),
            0x4A => self.c = self.ld_r8_r8(self.d),
            0x4B => self.c = self.ld_r8_r8(self.e),
            0x4C => self.c = self.ld_r8_r8(self.h),
            0x4D => self.c = self.ld_r8_r8(self.l),
            0x4E => self.c = self.ld_r8_mem_hl(mmu), // LD C, (HL)
            0x4F => self.c = self.ld_r8_r8(self.a),

            // Destinazione D
            0x50 => self.d = self.ld_r8_r8(self.b),
            0x51 => self.d = self.ld_r8_r8(self.c),
            0x52 => self.d = self.ld_r8_r8(self.d),
            0x53 => self.d = self.ld_r8_r8(self.e),
            0x54 => self.d = self.ld_r8_r8(self.h),
            0x55 => self.d = self.ld_r8_r8(self.l),
            0x56 => self.d = self.ld_r8_mem_hl(mmu), // LD D, (HL)
            0x57 => self.d = self.ld_r8_r8(self.a),

            // Destinazione E
            0x58 => self.e = self.ld_r8_r8(self.b),
            0x59 => self.e = self.ld_r8_r8(self.c),
            0x5A => self.e = self.ld_r8_r8(self.d),
            0x5B => self.e = self.ld_r8_r8(self.e),
            0x5C => self.e = self.ld_r8_r8(self.h),
            0x5D => self.e = self.ld_r8_r8(self.l),
            0x5E => self.e = self.ld_r8_mem_hl(mmu), // LD E, (HL)
            0x5F => self.e = self.ld_r8_r8(self.a),

            // Destinazione H
            0x60 => self.h = self.ld_r8_r8(self.b),
            0x61 => self.h = self.ld_r8_r8(self.c),
            0x62 => self.h = self.ld_r8_r8(self.d),
            0x63 => self.h = self.ld_r8_r8(self.e),
            0x64 => self.h = self.ld_r8_r8(self.h),
            0x65 => self.h = self.ld_r8_r8(self.l),
            0x66 => self.h = self.ld_r8_mem_hl(mmu), // LD H, (HL)
            0x67 => self.h = self.ld_r8_r8(self.a),

            // Destinazione L
            0x68 => self.l = self.ld_r8_r8(self.b),
            0x69 => self.l = self.ld_r8_r8(self.c),
            0x6A => self.l = self.ld_r8_r8(self.d),
            0x6B => self.l = self.ld_r8_r8(self.e),
            0x6C => self.l = self.ld_r8_r8(self.h),
            0x6D => self.l = self.ld_r8_r8(self.l),
            0x6E => self.l = self.ld_r8_mem_hl(mmu), // LD L, (HL)
            0x6F => self.l = self.ld_r8_r8(self.a),

            // Scrittura in memoria da registro (LD (HL), r8)
            // Nota: 0x76 è HALT ed è già gestito in alto, quindi non viene mappato qui!
            0x70 => self.ld_mem_hl_r8(mmu, self.b),
            0x71 => self.ld_mem_hl_r8(mmu, self.c),
            0x72 => self.ld_mem_hl_r8(mmu, self.d),
            0x73 => self.ld_mem_hl_r8(mmu, self.e),
            0x74 => self.ld_mem_hl_r8(mmu, self.h),
            0x75 => self.ld_mem_hl_r8(mmu, self.l),
            0x77 => self.ld_mem_hl_r8(mmu, self.a),

            // Destinazione A
            0x78 => self.a = self.ld_r8_r8(self.b),
            0x79 => self.a = self.ld_r8_r8(self.c),
            0x7A => self.a = self.ld_r8_r8(self.d),
            0x7B => self.a = self.ld_r8_r8(self.e),
            0x7C => self.a = self.ld_r8_r8(self.h),
            0x7D => self.a = self.ld_r8_r8(self.l),
            0x7E => self.a = self.ld_r8_mem_hl(mmu), // LD A, (HL)
            0x7F => self.a = self.ld_r8_r8(self.a),

            // ==========================================
            // BLOCCO ALU REGISTRI (0x80 - 0xBF)
            // Ogni riga corrisponde a un'operazione specifica
            // ==========================================
            // ADD A, r8
            0x80 => self.alu_add(self.b),
            0x81 => self.alu_add(self.c),
            0x82 => self.alu_add(self.d),
            0x83 => self.alu_add(self.e),
            0x84 => self.alu_add(self.h),
            0x85 => self.alu_add(self.l),
            0x86 => self.alu_add_mem_hl(mmu),
            0x87 => self.alu_add(self.a),

            // ADC A, r8
            0x88 => self.alu_adc(self.b),
            0x89 => self.alu_adc(self.c),
            0x8A => self.alu_adc(self.d),
            0x8B => self.alu_adc(self.e),
            0x8C => self.alu_adc(self.h),
            0x8D => self.alu_adc(self.l),
            0x8E => self.alu_adc_mem_hl(mmu),
            0x8F => self.alu_adc(self.a),

            // SUB A, r8
            0x90 => self.alu_sub(self.b),
            0x91 => self.alu_sub(self.c),
            0x92 => self.alu_sub(self.d),
            0x93 => self.alu_sub(self.e),
            0x94 => self.alu_sub(self.h),
            0x95 => self.alu_sub(self.l),
            0x96 => self.alu_sub_mem_hl(mmu),
            0x97 => self.alu_sub(self.a),

            // SBC A, r8
            0x98 => self.alu_sbc(self.b),
            0x99 => self.alu_sbc(self.c),
            0x9A => self.alu_sbc(self.d),
            0x9B => self.alu_sbc(self.e),
            0x9C => self.alu_sbc(self.h),
            0x9D => self.alu_sbc(self.l),
            0x9E => self.alu_sbc_mem_hl(mmu),
            0x9F => self.alu_sbc(self.a),

            // AND A, r8
            0xA0 => self.alu_and(self.b),
            0xA1 => self.alu_and(self.c),
            0xA2 => self.alu_and(self.d),
            0xA3 => self.alu_and(self.e),
            0xA4 => self.alu_and(self.h),
            0xA5 => self.alu_and(self.l),
            0xA6 => self.alu_and_mem_hl(mmu),
            0xA7 => self.alu_and(self.a),

            // XOR A, r8
            0xA8 => self.alu_xor(self.b),
            0xA9 => self.alu_xor(self.c),
            0xAA => self.alu_xor(self.d),
            0xAB => self.alu_xor(self.e),
            0xAC => self.alu_xor(self.h),
            0xAD => self.alu_xor(self.l),
            0xAE => self.alu_xor_mem_hl(mmu),
            0xAF => self.alu_xor(self.a),

            // OR A, r8
            0xB0 => self.alu_or(self.b),
            0xB1 => self.alu_or(self.c),
            0xB2 => self.alu_or(self.d),
            0xB3 => self.alu_or(self.e),
            0xB4 => self.alu_or(self.h),
            0xB5 => self.alu_or(self.l),
            0xB6 => self.alu_or_mem_hl(mmu),
            0xB7 => self.alu_or(self.a),

            // CP A, r8
            0xB8 => self.alu_cp(self.b),
            0xB9 => self.alu_cp(self.c),
            0xBA => self.alu_cp(self.d),
            0xBB => self.alu_cp(self.e),
            0xBC => self.alu_cp(self.h),
            0xBD => self.alu_cp(self.l),
            0xBE => self.alu_cp_mem_hl(mmu),
            0xBF => self.alu_cp(self.a),

            // ==========================================
            // GESTIONE STACK, CALL, RET, JP
            // ==========================================
            // POP & PUSH
            0xC1 => {
                let bc = self.pop_r16(mmu);
                set_u16register!(self, self.b, self.c, bc);
            }
            0xD1 => {
                let de = self.pop_r16(mmu);
                set_u16register!(self, self.d, self.e, de);
            }
            0xE1 => {
                let hl = self.pop_r16(mmu);
                set_u16register!(self, self.h, self.l, hl);
            }
            0xF1 => {
                let af = self.pop_r16(mmu);
                self.a = ((af >> 8) & 0xFF) as u8;
                self.f = (af & 0xF0) as u8;
            }

            0xC5 => self.push_r16(self.b, self.c, mmu),
            0xD5 => self.push_r16(self.d, self.e, mmu),
            0xE5 => self.push_r16(self.h, self.l, mmu),
            0xF5 => self.push_af(mmu),

            // RET Condizionati ed Incondizionati
            0xC0 => self.ret_cond(!self.get_flag_z()),
            0xC8 => self.ret_cond(self.get_flag_z()),
            0xD0 => self.ret_cond(!self.get_flag_c()),
            0xD8 => self.ret_cond(self.get_flag_c()),
            0xC9 => self.ret_inconditional(),
            0xD9 => self.reti(),

            // JP Condizionati ed Incondizionati
            0xC2 => self.jp_cond(!self.get_flag_z()),
            0xCA => self.jp_cond(self.get_flag_z()),
            0xD2 => self.jp_cond(!self.get_flag_c()),
            0xDA => self.jp_cond(self.get_flag_c()),
            0xC3 => self.jp(),
            0xE9 => self.jp_hl(),

            // CALL Condizionati ed Incondizionati
            0xC4 => self.call_cond(!self.get_flag_z()),
            0xCC => self.call_cond(self.get_flag_z()),
            0xD4 => self.call_cond(!self.get_flag_c()),
            0xDC => self.call_cond(self.get_flag_c()),
            0xCD => self.call(),

            // RESTART (RST)
            0xC7 => self.rst(0x00),
            0xCF => self.rst(0x08),
            0xD7 => self.rst(0x10),
            0xDF => self.rst(0x18),
            0xE7 => self.rst(0x20),
            0xEF => self.rst(0x28),
            0xF7 => self.rst(0x30),
            0xFF => self.rst(0x38),

            // ==========================================
            // OPERAZIONI ALU IMMEDIATE (Valori a 8 bit)
            // ==========================================
            0xC6 => self.add_a_imm8(),
            0xCE => self.adc_a_imm8(),
            0xD6 => self.sub_a_imm8(),
            0xDE => self.sbc_a_imm8(),
            0xE6 => self.and_a_imm8(),
            0xEE => self.xor_a_imm8(),
            0xF6 => self.or_a_imm8(),
            0xFE => self.cp_a_imm8(),

            // ==========================================
            // CARICAMENTI SPECIALI / RAM ALTA (LDH)
            // ==========================================
            0xE0 => self.ldh_mem8_a(), // LDH (n8), A
            0xF0 => self.ldh_a_mem8(), // LDH A, (n8)
            0xE2 => self.ld_mem_c_a(), // LD (C), A
            0xF2 => self.ld_a_mem_c(), // LD A, (C)
            0xEA => self.ld_mem16_a(), // LD (n16), A
            0xFA => self.ld_a_mem16(), // LD A, (n16)

            // Manipolazioni SP
            0xE8 => self.add_sp_e8(),
            0xF8 => self.ld_hl_sp_e8(),
            0xF9 => self.ld_sp_hl(),

            // Interrupts
            0xF3 => self.di(),
            0xFB => self.ei(),

            // ==========================================
            // PREFISSO SPECIALE 0xCB
            // ==========================================
            0xCB => self.cb(mmu),

            _ => panic!("Opcode non valido o non implementato: {:#04X}", opcode),
        }
    }

    fn cb_prefixed(&mut self, cb_opcode: u8, mmu: &mut Mmu) {
        match cb_opcode {
            // 0x00..0x07: RLC
            0x00 => self.b = self.rlc_r8(self.b),
            0x01 => self.c = self.rlc_r8(self.c),
            0x02 => self.d = self.rlc_r8(self.d),
            0x03 => self.e = self.rlc_r8(self.e),
            0x04 => self.h = self.rlc_r8(self.h),
            0x05 => self.l = self.rlc_r8(self.l),
            0x06 => self.rlc_hl_mem(mmu),
            0x07 => self.a = self.rlc_r8(self.a),

            // 0x08..0x0F: RRC
            0x08 => self.b = self.rrc_r8(self.b),
            0x09 => self.c = self.rrc_r8(self.c),
            0x0A => self.d = self.rrc_r8(self.d),
            0x0B => self.e = self.rrc_r8(self.e),
            0x0C => self.h = self.rrc_r8(self.h),
            0x0D => self.l = self.rrc_r8(self.l),
            0x0E => self.rrc_hl_mem(mmu),
            0x0F => self.a = self.rrc_r8(self.a),

            // 0x10..0x17: RL
            0x10 => self.b = self.rl_r8(self.b),
            0x11 => self.c = self.rl_r8(self.c),
            0x12 => self.d = self.rl_r8(self.d),
            0x13 => self.e = self.rl_r8(self.e),
            0x14 => self.h = self.rl_r8(self.h),
            0x15 => self.l = self.rl_r8(self.l),
            0x16 => self.rl_hl_mem(mmu),
            0x17 => self.a = self.rl_r8(self.a),

            // 0x18..0x1F: RR
            0x18 => self.b = self.rr_r8(self.b),
            0x19 => self.c = self.rr_r8(self.c),
            0x1A => self.d = self.rr_r8(self.d),
            0x1B => self.e = self.rr_r8(self.e),
            0x1C => self.h = self.rr_r8(self.h),
            0x1D => self.l = self.rr_r8(self.l),
            0x1E => self.rr_hl_mem(mmu),
            0x1F => self.a = self.rr_r8(self.a),

            // 0x20..0x27: SLA
            0x20 => self.b = self.sla_r8(self.b),
            0x21 => self.c = self.sla_r8(self.c),
            0x22 => self.d = self.sla_r8(self.d),
            0x23 => self.e = self.sla_r8(self.e),
            0x24 => self.h = self.sla_r8(self.h),
            0x25 => self.l = self.sla_r8(self.l),
            0x26 => self.sla_hl_mem(mmu),
            0x27 => self.a = self.sla_r8(self.a),

            // 0x28..0x2F: SRA
            0x28 => self.b = self.sra_r8(self.b),
            0x29 => self.c = self.sra_r8(self.c),
            0x2A => self.d = self.sra_r8(self.d),
            0x2B => self.e = self.sra_r8(self.e),
            0x2C => self.h = self.sra_r8(self.h),
            0x2D => self.l = self.sra_r8(self.l),
            0x2E => self.sra_hl_mem(mmu),
            0x2F => self.a = self.sra_r8(self.a),

            // 0x30..0x37: SWAP
            0x30 => self.b = self.swap_r8(self.b),
            0x31 => self.c = self.swap_r8(self.c),
            0x32 => self.d = self.swap_r8(self.d),
            0x33 => self.e = self.swap_r8(self.e),
            0x34 => self.h = self.swap_r8(self.h),
            0x35 => self.l = self.swap_r8(self.l),
            0x36 => self.swap_hl_mem(mmu),
            0x37 => self.a = self.swap_r8(self.a),

            // 0x38..0x3F: SRL
            0x38 => self.b = self.srl_r8(self.b),
            0x39 => self.c = self.srl_r8(self.c),
            0x3A => self.d = self.srl_r8(self.d),
            0x3B => self.e = self.srl_r8(self.e),
            0x3C => self.h = self.srl_r8(self.h),
            0x3D => self.l = self.srl_r8(self.l),
            0x3E => self.srl_hl_mem(mmu),
            0x3F => self.a = self.srl_r8(self.a),

            // 0x40..0x7F: BIT
            0x40 => self.bit_b_r8(0, self.b),
            0x41 => self.bit_b_r8(0, self.c),
            0x42 => self.bit_b_r8(0, self.d),
            0x43 => self.bit_b_r8(0, self.e),
            0x44 => self.bit_b_r8(0, self.h),
            0x45 => self.bit_b_r8(0, self.l),
            0x46 => self.bit_b_hl_mem(0, mmu),
            0x47 => self.bit_b_r8(0, self.a),

            0x48 => self.bit_b_r8(1, self.b),
            0x49 => self.bit_b_r8(1, self.c),
            0x4A => self.bit_b_r8(1, self.d),
            0x4B => self.bit_b_r8(1, self.e),
            0x4C => self.bit_b_r8(1, self.h),
            0x4D => self.bit_b_r8(1, self.l),
            0x4E => self.bit_b_hl_mem(1, mmu),
            0x4F => self.bit_b_r8(1, self.a),

            0x50 => self.bit_b_r8(2, self.b),
            0x51 => self.bit_b_r8(2, self.c),
            0x52 => self.bit_b_r8(2, self.d),
            0x53 => self.bit_b_r8(2, self.e),
            0x54 => self.bit_b_r8(2, self.h),
            0x55 => self.bit_b_r8(2, self.l),
            0x56 => self.bit_b_hl_mem(2, mmu),
            0x57 => self.bit_b_r8(2, self.a),

            0x58 => self.bit_b_r8(3, self.b),
            0x59 => self.bit_b_r8(3, self.c),
            0x5A => self.bit_b_r8(3, self.d),
            0x5B => self.bit_b_r8(3, self.e),
            0x5C => self.bit_b_r8(3, self.h),
            0x5D => self.bit_b_r8(3, self.l),
            0x5E => self.bit_b_hl_mem(3, mmu),
            0x5F => self.bit_b_r8(3, self.a),

            0x60 => self.bit_b_r8(4, self.b),
            0x61 => self.bit_b_r8(4, self.c),
            0x62 => self.bit_b_r8(4, self.d),
            0x63 => self.bit_b_r8(4, self.e),
            0x64 => self.bit_b_r8(4, self.h),
            0x65 => self.bit_b_r8(4, self.l),
            0x66 => self.bit_b_hl_mem(4, mmu),
            0x67 => self.bit_b_r8(4, self.a),

            0x68 => self.bit_b_r8(5, self.b),
            0x69 => self.bit_b_r8(5, self.c),
            0x6A => self.bit_b_r8(5, self.d),
            0x6B => self.bit_b_r8(5, self.e),
            0x6C => self.bit_b_r8(5, self.h),
            0x6D => self.bit_b_r8(5, self.l),
            0x6E => self.bit_b_hl_mem(5, mmu),
            0x6F => self.bit_b_r8(5, self.a),

            0x70 => self.bit_b_r8(6, self.b),
            0x71 => self.bit_b_r8(6, self.c),
            0x72 => self.bit_b_r8(6, self.d),
            0x73 => self.bit_b_r8(6, self.e),
            0x74 => self.bit_b_r8(6, self.h),
            0x75 => self.bit_b_r8(6, self.l),
            0x76 => self.bit_b_hl_mem(6, mmu),
            0x77 => self.bit_b_r8(6, self.a),

            0x78 => self.bit_b_r8(7, self.b),
            0x79 => self.bit_b_r8(7, self.c),
            0x7A => self.bit_b_r8(7, self.d),
            0x7B => self.bit_b_r8(7, self.e),
            0x7C => self.bit_b_r8(7, self.h),
            0x7D => self.bit_b_r8(7, self.l),
            0x7E => self.bit_b_hl_mem(7, mmu),
            0x7F => self.bit_b_r8(7, self.a),

            // 0x80..0xBF: RES
            0x80 => self.b = self.res_b_r8(0, self.b),
            0x81 => self.c = self.res_b_r8(0, self.c),
            0x82 => self.d = self.res_b_r8(0, self.d),
            0x83 => self.e = self.res_b_r8(0, self.e),
            0x84 => self.h = self.res_b_r8(0, self.h),
            0x85 => self.l = self.res_b_r8(0, self.l),
            0x86 => self.res_b_hl_mem(0, mmu),
            0x87 => self.a = self.res_b_r8(0, self.a),

            0x88 => self.b = self.res_b_r8(1, self.b),
            0x89 => self.c = self.res_b_r8(1, self.c),
            0x8A => self.d = self.res_b_r8(1, self.d),
            0x8B => self.e = self.res_b_r8(1, self.e),
            0x8C => self.h = self.res_b_r8(1, self.h),
            0x8D => self.l = self.res_b_r8(1, self.l),
            0x8E => self.res_b_hl_mem(1, mmu),
            0x8F => self.a = self.res_b_r8(1, self.a),

            0x90 => self.b = self.res_b_r8(2, self.b),
            0x91 => self.c = self.res_b_r8(2, self.c),
            0x92 => self.d = self.res_b_r8(2, self.d),
            0x93 => self.e = self.res_b_r8(2, self.e),
            0x94 => self.h = self.res_b_r8(2, self.h),
            0x95 => self.l = self.res_b_r8(2, self.l),
            0x96 => self.res_b_hl_mem(2, mmu),
            0x97 => self.a = self.res_b_r8(2, self.a),

            0x98 => self.b = self.res_b_r8(3, self.b),
            0x99 => self.c = self.res_b_r8(3, self.c),
            0x9A => self.d = self.res_b_r8(3, self.d),
            0x9B => self.e = self.res_b_r8(3, self.e),
            0x9C => self.h = self.res_b_r8(3, self.h),
            0x9D => self.l = self.res_b_r8(3, self.l),
            0x9E => self.res_b_hl_mem(3, mmu),
            0x9F => self.a = self.res_b_r8(3, self.a),

            0xA0 => self.b = self.res_b_r8(4, self.b),
            0xA1 => self.c = self.res_b_r8(4, self.c),
            0xA2 => self.d = self.res_b_r8(4, self.d),
            0xA3 => self.e = self.res_b_r8(4, self.e),
            0xA4 => self.h = self.res_b_r8(4, self.h),
            0xA5 => self.l = self.res_b_r8(4, self.l),
            0xA6 => self.res_b_hl_mem(4, mmu),
            0xA7 => self.a = self.res_b_r8(4, self.a),

            0xA8 => self.b = self.res_b_r8(5, self.b),
            0xA9 => self.c = self.res_b_r8(5, self.c),
            0xAA => self.d = self.res_b_r8(5, self.d),
            0xAB => self.e = self.res_b_r8(5, self.e),
            0xAC => self.h = self.res_b_r8(5, self.h),
            0xAD => self.l = self.res_b_r8(5, self.l),
            0xAE => self.res_b_hl_mem(5, mmu),
            0xAF => self.a = self.res_b_r8(5, self.a),

            0xB0 => self.b = self.res_b_r8(6, self.b),
            0xB1 => self.c = self.res_b_r8(6, self.c),
            0xB2 => self.d = self.res_b_r8(6, self.d),
            0xB3 => self.e = self.res_b_r8(6, self.e),
            0xB4 => self.h = self.res_b_r8(6, self.h),
            0xB5 => self.l = self.res_b_r8(6, self.l),
            0xB6 => self.res_b_hl_mem(6, mmu),
            0xB7 => self.a = self.res_b_r8(6, self.a),

            0xB8 => self.b = self.res_b_r8(7, self.b),
            0xB9 => self.c = self.res_b_r8(7, self.c),
            0xBA => self.d = self.res_b_r8(7, self.d),
            0xBB => self.e = self.res_b_r8(7, self.e),
            0xBC => self.h = self.res_b_r8(7, self.h),
            0xBD => self.l = self.res_b_r8(7, self.l),
            0xBE => self.res_b_hl_mem(7, mmu),
            0xBF => self.a = self.res_b_r8(7, self.a),

            // 0xC0..0xFF: SET
            0xC0 => self.b = self.set_b_r8(0, self.b),
            0xC1 => self.c = self.set_b_r8(0, self.c),
            0xC2 => self.d = self.set_b_r8(0, self.d),
            0xC3 => self.e = self.set_b_r8(0, self.e),
            0xC4 => self.h = self.set_b_r8(0, self.h),
            0xC5 => self.l = self.set_b_r8(0, self.l),
            0xC6 => self.set_b_hl_mem(0, mmu),
            0xC7 => self.a = self.set_b_r8(0, self.a),

            0xC8 => self.b = self.set_b_r8(1, self.b),
            0xC9 => self.c = self.set_b_r8(1, self.c),
            0xCA => self.d = self.set_b_r8(1, self.d),
            0xCB => self.e = self.set_b_r8(1, self.e),
            0xCC => self.h = self.set_b_r8(1, self.h),
            0xCD => self.l = self.set_b_r8(1, self.l),
            0xCE => self.set_b_hl_mem(1, mmu),
            0xCF => self.a = self.set_b_r8(1, self.a),

            0xD0 => self.b = self.set_b_r8(2, self.b),
            0xD1 => self.c = self.set_b_r8(2, self.c),
            0xD2 => self.d = self.set_b_r8(2, self.d),
            0xD3 => self.e = self.set_b_r8(2, self.e),
            0xD4 => self.h = self.set_b_r8(2, self.h),
            0xD5 => self.l = self.set_b_r8(2, self.l),
            0xD6 => self.set_b_hl_mem(2, mmu),
            0xD7 => self.a = self.set_b_r8(2, self.a),

            0xD8 => self.b = self.set_b_r8(3, self.b),
            0xD9 => self.c = self.set_b_r8(3, self.c),
            0xDA => self.d = self.set_b_r8(3, self.d),
            0xDB => self.e = self.set_b_r8(3, self.e),
            0xDC => self.h = self.set_b_r8(3, self.h),
            0xDD => self.l = self.set_b_r8(3, self.l),
            0xDE => self.set_b_hl_mem(3, mmu),
            0xDF => self.a = self.set_b_r8(3, self.a),

            0xE0 => self.b = self.set_b_r8(4, self.b),
            0xE1 => self.c = self.set_b_r8(4, self.c),
            0xE2 => self.d = self.set_b_r8(4, self.d),
            0xE3 => self.e = self.set_b_r8(4, self.e),
            0xE4 => self.h = self.set_b_r8(4, self.h),
            0xE5 => self.l = self.set_b_r8(4, self.l),
            0xE6 => self.set_b_hl_mem(4, mmu),
            0xE7 => self.a = self.set_b_r8(4, self.a),

            0xE8 => self.b = self.set_b_r8(5, self.b),
            0xE9 => self.c = self.set_b_r8(5, self.c),
            0xEA => self.d = self.set_b_r8(5, self.d),
            0xEB => self.e = self.set_b_r8(5, self.e),
            0xEC => self.h = self.set_b_r8(5, self.h),
            0xED => self.l = self.set_b_r8(5, self.l),
            0xEE => self.set_b_hl_mem(5, mmu),
            0xEF => self.a = self.set_b_r8(5, self.a),

            0xF0 => self.b = self.set_b_r8(6, self.b),
            0xF1 => self.c = self.set_b_r8(6, self.c),
            0xF2 => self.d = self.set_b_r8(6, self.d),
            0xF3 => self.e = self.set_b_r8(6, self.e),
            0xF4 => self.h = self.set_b_r8(6, self.h),
            0xF5 => self.l = self.set_b_r8(6, self.l),
            0xF6 => self.set_b_hl_mem(6, mmu),
            0xF7 => self.a = self.set_b_r8(6, self.a),

            0xF8 => self.b = self.set_b_r8(7, self.b),
            0xF9 => self.c = self.set_b_r8(7, self.c),
            0xFA => self.d = self.set_b_r8(7, self.d),
            0xFB => self.e = self.set_b_r8(7, self.e),
            0xFC => self.h = self.set_b_r8(7, self.h),
            0xFD => self.l = self.set_b_r8(7, self.l),
            0xFE => self.set_b_hl_mem(7, mmu),
            0xFF => self.a = self.set_b_r8(7, self.a),
        }
    }

    fn fetch_byte(&mut self, mmu: &Mmu, pc: u16) -> u8 {
        mmu.read_byte(pc)
    }
}
