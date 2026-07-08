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

    pub fn step(&mut self, mmu: &mut Mmu, pc: u16) {
        let opcode = self.fetch_byte(mmu, pc); // Passa una reference al MMU per leggere l'opcode
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
                let mut bc = get_u16register!(self, self.b, self.c);
                self.ld_r16_imm16(&mut bc, mmu)
                set_u16register!(self, self.b, self.c, bc);
            }
            0x11 => {
                let mut de = get_u16register!(self, self.d, self.e);
                self.ld_r16_imm16(&mut de, mmu);
                set_u16register!(self, self.d, self.e, de);
            }
            0x21 => {
                let mut hl = get_u16register!(self, self.h, self.l);
                self.ld_r16_imm16(&mut hl, mmu);
                set_u16register!(self, self.h, self.l, hl);
            }
            0x31 => self.ld_r16_imm16(&mut self.sp, mmu),

            // ==========================================
            // INCREMENTI E DECREMENTI A 16 BIT
            // ==========================================
            0x03 => {
                let mut bc = get_u16register!(self, self.b, self.c);
                self.inc_r16(&mut bc)
                set_u16register!(self, self.b, self.c, bc);
            },
            0x13 => {
                let mut de = get_u16register!(self, self.d, self.e);
                self.inc_r16(&mut de);
                set_u16register!(self, self.d, self.e, de);
            },
            0x23 => {
                let mut hl = get_u16register!(self, self.h, self.l);
                self.inc_r16(&mut hl);
                set_u16register!(self, self.h, self.l, hl);
            },
            0x33 => self.inc_r16(&mut self.sp),

            0x0B => {
                let mut bc = get_u16register!(self, self.b, self.c);
                self.dec_r16(&mut bc);
                set_u16register!(self, self.b, self.c, bc);
            },
            0x1B => {
                let mut de = get_u16register!(self, self.d, self.e);
                self.dec_r16(&mut de);
                set_u16register!(self, self.d, self.e, de);
            },
            0x2B => {
                let mut hl = get_u16register!(self, self.h, self.l);
                self.dec_r16(&mut hl);
                set_u16register!(self, self.h, self.l, hl);
            },
            0x3B => self.dec_r16(&mut self.sp),

            // ==========================================
            // INCREMENTI E DECREMENTI A 8 BIT
            // ==========================================
            0x04 => self.b = self.inc_r8(&mut self.b),
            0x0C => self.c = self.inc_r8(&mut self.c),
            0x14 => self.d = self.inc_r8(&mut self.d),
            0x1C => self.e = self.inc_r8(&mut self.e),
            0x24 => self.h = self.inc_r8(&mut self.h),
            0x2C => self.l = self.inc_r8(&mut self.l),
            0x34 => self.a = self.inc_r8(&mut self.a), // Speciale: incrementa la memoria puntata da (HL)
            0x3C => self.a = self.inc_r8(&mut self.a),

            0x05 => self.b = self.dec_r8(&mut self.b),
            0x0D => self.c = self.dec_r8(&mut self.c),
            0x15 => self.d = self.dec_r8(&mut self.d),
            0x1D => self.e = self.dec_r8(&mut self.e),
            0x25 => self.h =self.dec_r8(&mut self.h),
            0x2C => self.l = self.dec_r8(&mut self.l),
            0x35 => self.dec_hl_mem(mmu), // Speciale: decrementa la memoria puntata da (HL)
            0x3D => self.a = self.dec_r8(&mut self.a),

            // ==========================================
            // CARICAMENTI IMMEDIATI A 8 BIT (LD r8, n8)
            // ==========================================
            0x06 => self.ld_r8_imm8(&mut self.b, mmu),
            0x0E => self.ld_r8_imm8(&mut self.c, mmu),
            0x16 => self.ld_r8_imm8(&mut self.d, mmu),
            0x1E => self.ld_r8_imm8(&mut self.e, mmu),
            0x26 => self.ld_r8_imm8(&mut self.h, mmu),
            0x2E => self.ld_r8_imm8(&mut self.l, mmu),
            0x36 => self.ld_hl_mem_imm8(mmu), // LD (HL), n8
            0x3E => self.ld_r8_imm8(&mut self.a, mmu),

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
            0x50 => self.d = self.ld_r8_r8( self.b),
            0x51 => self.d = self.ld_r8_r8( self.c),
            0x52 => self.d = self.ld_r8_r8( self.d),
            0x53 => self.d = self.ld_r8_r8( self.e),
            0x54 => self.d = self.ld_r8_r8( self.h),
            0x55 => self.d = self.ld_r8_r8( self.l),
            0x56 => self.d = self.ld_r8_mem_hl(mmu), // LD D, (HL)
            0x57 => self.d = self.ld_r8_r8( self.a),

            // Destinazione E
            0x58 => self.e = self.ld_r8_r8(self.b),
            0x59 => self.e = self.ld_r8_r8(self.c),
            0x5A => self.e = self.ld_r8_r8(self.d),
            0x5B => self.e = self.ld_r8_r8(self.e),
            0x5C => self.e = self.ld_r8_r8(self.h),
            0x5D => self.e = self.ld_r8_r8(self.l),
            0x5E => self.e = self.ld_r8_mem_hl(mmu), // LD E, (HL)
            0x5F => self.a = self.ld_r8_r8(self.e),

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
            0x80 => self.alu_op("alu_add", "B"),
            0x81 => self.alu_op("alu_add", "C"),
            0x82 => self.alu_op("alu_add", "D"),
            0x83 => self.alu_op("alu_add", "E"),
            0x84 => self.alu_op("alu_add", "H"),
            0x85 => self.alu_op("alu_add", "L"),
            0x86 => self.alu_add_mem_hl(mmu),
            0x87 => self.alu_op("alu_add", "A"),

            // ADC A, r8
            0x88 => self.alu_op("alu_adc", "B"),
            0x89 => self.alu_op("alu_adc", "C"),
            0x8A => self.alu_op("alu_adc", "D"),
            0x8B => self.alu_op("alu_adc", "E"),
            0x8C => self.alu_op("alu_adc", "H"),
            0x8D => self.alu_op("alu_adc", "L"),
            0x8E => self.alu_adc_mem_hl(mmu),
            0x8F => self.alu_op("alu_adc", "A"),

            // SUB A, r8
            0x90 => self.alu_op("alu_sub", "B"),
            0x91 => self.alu_op("alu_sub", "C"),
            0x92 => self.alu_op("alu_sub", "D"),
            0x93 => self.alu_op("alu_sub", "E"),
            0x94 => self.alu_op("alu_sub", "H"),
            0x95 => self.alu_op("alu_sub", "L"),
            0x96 => self.alu_sub_mem_hl(mmu),
            0x97 => self.alu_op("alu_sub", "A"),

            // SBC A, r8
            0x98 => self.alu_op("alu_sbc", "B"),
            0x99 => self.alu_op("alu_sbc", "C"),
            0x9A => self.alu_op("alu_sbc", "D"),
            0x9B => self.alu_op("alu_sbc", "E"),
            0x9C => self.alu_op("alu_sbc", "H"),
            0x9D => self.alu_op("alu_sbc", "L"),
            0x9E => self.alu_sbc_mem_hl(mmu),
            0x9F => self.alu_op("alu_sbc", "A"),

            // AND A, r8
            0xA0 => self.alu_op("alu_and", "B"),
            0xA1 => self.alu_op("alu_and", "C"),
            0xA2 => self.alu_op("alu_and", "D"),
            0xA3 => self.alu_op("alu_and", "E"),
            0xA4 => self.alu_op("alu_and", "H"),
            0xA5 => self.alu_op("alu_and", "L"),
            0xA6 => self.alu_and_mem_hl(mmu),
            0xA7 => self.alu_op("alu_and", "A"),

            // XOR A, r8
            0xA8 => self.alu_op("alu_xor", "B"),
            0xA9 => self.alu_op("alu_xor", "C"),
            0xAA => self.alu_op("alu_xor", "D"),
            0xAB => self.alu_op("alu_xor", "E"),
            0xAC => self.alu_op("alu_xor", "H"),
            0xAD => self.alu_op("alu_xor", "L"),
            0xAE => self.alu_xor_mem_hl(mmu),
            0xAF => self.alu_op("alu_xor", "A"),

            // OR A, r8
            0xB0 => self.alu_op("alu_or", "B"),
            0xB1 => self.alu_op("alu_or", "C"),
            0xB2 => self.alu_op("alu_or", "D"),
            0xB3 => self.alu_op("alu_or", "E"),
            0xB4 => self.alu_op("alu_or", "H"),
            0xB5 => self.alu_op("alu_or", "L"),
            0xB6 => self.alu_or_mem_hl(mmu),
            0xB7 => self.alu_op("alu_or", "A"),

            // CP A, r8
            0xB8 => self.alu_op("alu_cp", "B"),
            0xB9 => self.alu_op("alu_cp", "C"),
            0xBA => self.alu_op("alu_cp", "D"),
            0xBB => self.alu_op("alu_cp", "E"),
            0xBC => self.alu_op("alu_cp", "H"),
            0xBD => self.alu_op("alu_cp", "L"),
            0xBE => self.alu_cp_mem_hl(mmu),
            0xBF => self.alu_op("alu_cp", "A"),

            // ==========================================
            // GESTIONE STACK, CALL, RET, JP
            // ==========================================
            // POP & PUSH
            0xC1 => self.pop_r16(Reg16::BC),
            0xD1 => self.pop_r16(Reg16::DE),
            0xE1 => self.pop_r16(Reg16::HL),
            0xF1 => self.pop_r16(Reg16::AF),

            0xC5 => self.push_r16(Reg16::BC),
            0xD5 => self.push_r16(Reg16::DE),
            0xE5 => self.push_r16(Reg16::HL),
            0xF5 => self.push_r16(Reg16::AF),

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
            0xC3 => self.jp_inconditional(),
            0xE9 => self.jp_hl(),

            // CALL Condizionati ed Incondizionati
            0xC4 => self.call_cond(!self.get_flag_z()),
            0xCC => self.call_cond(self.get_flag_z()),
            0xD4 => self.call_cond(!self.get_flag_c()),
            0xDC => self.call_cond(self.get_flag_c()),
            0xCD => self.call_inconditional(),

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
            0xCB => {
                let cb_opcode = self.fetch_byte(mmu, pc.wrapping_add(1));
                self.cb_prefixed(cb_opcode, mmu);
            }

            _ => panic!("Opcode non valido o non implementato: {:#04X}", opcode),
        }
    }

    fn cb_prefixed(&mut self, cb_opcode: u8, mmu: &mut Mmu) {
        match cb_opcode {
            0x00 => self.cb_0x00(mmu),
            0x01 => self.cb_0x01(mmu),
            0x02 => self.cb_0x02(mmu),
            0x03 => self.cb_0x03(mmu),
            0x04 => self.cb_0x04(mmu),
            0x05 => self.cb_0x05(mmu),
            0x06 => self.cb_0x06(mmu),
            0x07 => self.cb_0x07(mmu),
            0x08 => self.cb_0x08(mmu),
            0x09 => self.cb_0x09(mmu),
            0x0A => self.cb_0x0a(mmu),
            0x0B => self.cb_0x0b(mmu),
            0x0C => self.cb_0x0c(mmu),
            0x0D => self.cb_0x0d(mmu),
            0x0E => self.cb_0x0e(mmu),
            0x0F => self.cb_0x0f(mmu),
            0x10 => self.cb_0x10(mmu),
            0x11 => self.cb_0x11(mmu),
            0x12 => self.cb_0x12(mmu),
            0x13 => self.cb_0x13(mmu),
            0x14 => self.cb_0x14(mmu),
            0x15 => self.cb_0x15(mmu),
            0x16 => self.cb_0x16(mmu),
            0x17 => self.cb_0x17(mmu),
            0x18 => self.cb_0x18(mmu),
            0x19 => self.cb_0x19(mmu),
            0x1A => self.cb_0x1a(mmu),
            0x1B => self.cb_0x1b(mmu),
            0x1C => self.cb_0x1c(mmu),
            0x1D => self.cb_0x1d(mmu),
            0x1E => self.cb_0x1e(mmu),
            0x1F => self.cb_0x1f(mmu),
            0x20 => self.cb_0x20(mmu),
            0x21 => self.cb_0x21(mmu),
            0x22 => self.cb_0x22(mmu),
            0x23 => self.cb_0x23(mmu),
            0x24 => self.cb_0x24(mmu),
            0x25 => self.cb_0x25(mmu),
            0x26 => self.cb_0x26(mmu),
            0x27 => self.cb_0x27(mmu),
            0x28 => self.cb_0x28(mmu),
            0x29 => self.cb_0x29(mmu),
            0x2A => self.cb_0x2a(mmu),
            0x2B => self.cb_0x2b(mmu),
            0x2C => self.cb_0x2c(mmu),
            0x2D => self.cb_0x2d(mmu),
            0x2E => self.cb_0x2e(mmu),
            0x2F => self.cb_0x2f(mmu),
            0x30 => self.cb_0x30(mmu),
            0x31 => self.cb_0x31(mmu),
            0x32 => self.cb_0x32(mmu),
            0x33 => self.cb_0x33(mmu),
            0x34 => self.cb_0x34(mmu),
            0x35 => self.cb_0x35(mmu),
            0x36 => self.cb_0x36(mmu),
            0x37 => self.cb_0x37(mmu),
            0x38 => self.cb_0x38(mmu),
            0x39 => self.cb_0x39(mmu),
            0x3A => self.cb_0x3a(mmu),
            0x3B => self.cb_0x3b(mmu),
            0x3C => self.cb_0x3c(mmu),
            0x3D => self.cb_0x3d(mmu),
            0x3E => self.cb_0x3e(mmu),
            0x3F => self.cb_0x3f(mmu),
            0x40 => self.cb_0x40(mmu),
            0x41 => self.cb_0x41(mmu),
            0x42 => self.cb_0x42(mmu),
            0x43 => self.cb_0x43(mmu),
            0x44 => self.cb_0x44(mmu),
            0x45 => self.cb_0x45(mmu),
            0x46 => self.cb_0x46(mmu),
            0x47 => self.cb_0x47(mmu),
            0x48 => self.cb_0x48(mmu),
            0x49 => self.cb_0x49(mmu),
            0x4A => self.cb_0x4a(mmu),
            0x4B => self.cb_0x4b(mmu),
            0x4C => self.cb_0x4c(mmu),
            0x4D => self.cb_0x4d(mmu),
            0x4E => self.cb_0x4e(mmu),
            0x4F => self.cb_0x4f(mmu),
            0x50 => self.cb_0x50(mmu),
            0x51 => self.cb_0x51(mmu),
            0x52 => self.cb_0x52(mmu),
            0x53 => self.cb_0x53(mmu),
            0x54 => self.cb_0x54(mmu),
            0x55 => self.cb_0x55(mmu),
            0x56 => self.cb_0x56(mmu),
            0x57 => self.cb_0x57(mmu),
            0x58 => self.cb_0x58(mmu),
            0x59 => self.cb_0x59(mmu),
            0x5A => self.cb_0x5a(mmu),
            0x5B => self.cb_0x5b(mmu),
            0x5C => self.cb_0x5c(mmu),
            0x5D => self.cb_0x5d(mmu),
            0x5E => self.cb_0x5e(mmu),
            0x5F => self.cb_0x5f(mmu),
            0x60 => self.cb_0x60(mmu),
            0x61 => self.cb_0x61(mmu),
            0x62 => self.cb_0x62(mmu),
            0x63 => self.cb_0x63(mmu),
            0x64 => self.cb_0x64(mmu),
            0x65 => self.cb_0x65(mmu),
            0x66 => self.cb_0x66(mmu),
            0x67 => self.cb_0x67(mmu),
            0x68 => self.cb_0x68(mmu),
            0x69 => self.cb_0x69(mmu),
            0x6A => self.cb_0x6a(mmu),
            0x6B => self.cb_0x6b(mmu),
            0x6C => self.cb_0x6c(mmu),
            0x6D => self.cb_0x6d(mmu),
            0x6E => self.cb_0x6e(mmu),
            0x6F => self.cb_0x6f(mmu),
            0x70 => self.cb_0x70(mmu),
            0x71 => self.cb_0x71(mmu),
            0x72 => self.cb_0x72(mmu),
            0x73 => self.cb_0x73(mmu),
            0x74 => self.cb_0x74(mmu),
            0x75 => self.cb_0x75(mmu),
            0x76 => self.cb_0x76(mmu),
            0x77 => self.cb_0x77(mmu),
            0x78 => self.cb_0x78(mmu),
            0x79 => self.cb_0x79(mmu),
            0x7A => self.cb_0x7a(mmu),
            0x7B => self.cb_0x7b(mmu),
            0x7C => self.cb_0x7c(mmu),
            0x7D => self.cb_0x7d(mmu),
            0x7E => self.cb_0x7e(mmu),
            0x7F => self.cb_0x7f(mmu),
            0x80 => self.cb_0x80(mmu),
            0x81 => self.cb_0x81(mmu),
            0x82 => self.cb_0x82(mmu),
            0x83 => self.cb_0x83(mmu),
            0x84 => self.cb_0x84(mmu),
            0x85 => self.cb_0x85(mmu),
            0x86 => self.cb_0x86(mmu),
            0x87 => self.cb_0x87(mmu),
            0x88 => self.cb_0x88(mmu),
            0x89 => self.cb_0x89(mmu),
            0x8A => self.cb_0x8a(mmu),
            0x8B => self.cb_0x8b(mmu),
            0x8C => self.cb_0x8c(mmu),
            0x8D => self.cb_0x8d(mmu),
            0x8E => self.cb_0x8e(mmu),
            0x8F => self.cb_0x8f(mmu),
            0x90 => self.cb_0x90(mmu),
            0x91 => self.cb_0x91(mmu),
            0x92 => self.cb_0x92(mmu),
            0x93 => self.cb_0x93(mmu),
            0x94 => self.cb_0x94(mmu),
            0x95 => self.cb_0x95(mmu),
            0x96 => self.cb_0x96(mmu),
            0x97 => self.cb_0x97(mmu),
            0x98 => self.cb_0x98(mmu),
            0x99 => self.cb_0x99(mmu),
            0x9A => self.cb_0x9a(mmu),
            0x9B => self.cb_0x9b(mmu),
            0x9C => self.cb_0x9c(mmu),
            0x9D => self.cb_0x9d(mmu),
            0x9E => self.cb_0x9e(mmu),
            0x9F => self.cb_0x9f(mmu),
            0xA0 => self.cb_0xa0(mmu),
            0xA1 => self.cb_0xa1(mmu),
            0xA2 => self.cb_0xa2(mmu),
            0xA3 => self.cb_0xa3(mmu),
            0xA4 => self.cb_0xa4(mmu),
            0xA5 => self.cb_0xa5(mmu),
            0xA6 => self.cb_0xa6(mmu),
            0xA7 => self.cb_0xa7(mmu),
            0xA8 => self.cb_0xa8(mmu),
            0xA9 => self.cb_0xa9(mmu),
            0xAA => self.cb_0xaa(mmu),
            0xAB => self.cb_0xab(mmu),
            0xAC => self.cb_0xac(mmu),
            0xAD => self.cb_0xad(mmu),
            0xAE => self.cb_0xae(mmu),
            0xAF => self.cb_0xaf(mmu),
            0xB0 => self.cb_0xb0(mmu),
            0xB1 => self.cb_0xb1(mmu),
            0xB2 => self.cb_0xb2(mmu),
            0xB3 => self.cb_0xb3(mmu),
            0xB4 => self.cb_0xb4(mmu),
            0xB5 => self.cb_0xb5(mmu),
            0xB6 => self.cb_0xb6(mmu),
            0xB7 => self.cb_0xb7(mmu),
            0xB8 => self.cb_0xb8(mmu),
            0xB9 => self.cb_0xb9(mmu),
            0xBA => self.cb_0xba(mmu),
            0xBB => self.cb_0xbb(mmu),
            0xBC => self.cb_0xbc(mmu),
            0xBD => self.cb_0xbd(mmu),
            0xBE => self.cb_0xbe(mmu),
            0xBF => self.cb_0xbf(mmu),
            0xC0 => self.cb_0xc0(mmu),
            0xC1 => self.cb_0xc1(mmu),
            0xC2 => self.cb_0xc2(mmu),
            0xC3 => self.cb_0xc3(mmu),
            0xC4 => self.cb_0xc4(mmu),
            0xC5 => self.cb_0xc5(mmu),
            0xC6 => self.cb_0xc6(mmu),
            0xC7 => self.cb_0xc7(mmu),
            0xC8 => self.cb_0xc8(mmu),
            0xC9 => self.cb_0xc9(mmu),
            0xCA => self.cb_0xca(mmu),
            0xCB => self.cb_0xcb(mmu),
            0xCC => self.cb_0xcc(mmu),
            0xCD => self.cb_0xcd(mmu),
            0xCE => self.cb_0xce(mmu),
            0xCF => self.cb_0xcf(mmu),
            0xD0 => self.cb_0xd0(mmu),
            0xD1 => self.cb_0xd1(mmu),
            0xD2 => self.cb_0xd2(mmu),
            0xD3 => self.cb_0xd3(mmu),
            0xD4 => self.cb_0xd4(mmu),
            0xD5 => self.cb_0xd5(mmu),
            0xD6 => self.cb_0xd6(mmu),
            0xD7 => self.cb_0xd7(mmu),
            0xD8 => self.cb_0xd8(mmu),
            0xD9 => self.cb_0xd9(mmu),
            0xDA => self.cb_0xda(mmu),
            0xDB => self.cb_0xdb(mmu),
            0xDC => self.cb_0xdc(mmu),
            0xDD => self.cb_0xdd(mmu),
            0xDE => self.cb_0xde(mmu),
            0xDF => self.cb_0xdf(mmu),
            0xE0 => self.cb_0xe0(mmu),
            0xE1 => self.cb_0xe1(mmu),
            0xE2 => self.cb_0xe2(mmu),
            0xE3 => self.cb_0xe3(mmu),
            0xE4 => self.cb_0xe4(mmu),
            0xE5 => self.cb_0xe5(mmu),
            0xE6 => self.cb_0xe6(mmu),
            0xE7 => self.cb_0xe7(mmu),
            0xE8 => self.cb_0xe8(mmu),
            0xE9 => self.cb_0xe9(mmu),
            0xEA => self.cb_0xea(mmu),
            0xEB => self.cb_0xeb(mmu),
            0xEC => self.cb_0xec(mmu),
            0xED => self.cb_0xed(mmu),
            0xEE => self.cb_0xee(mmu),
            0xEF => self.cb_0xef(mmu),
            0xF0 => self.cb_0xf0(mmu),
            0xF1 => self.cb_0xf1(mmu),
            0xF2 => self.cb_0xf2(mmu),
            0xF3 => self.cb_0xf3(mmu),
            0xF4 => self.cb_0xf4(mmu),
            0xF5 => self.cb_0xf5(mmu),
            0xF6 => self.cb_0xf6(mmu),
            0xF7 => self.cb_0xf7(mmu),
            0xF8 => self.cb_0xf8(mmu),
            0xF9 => self.cb_0xf9(mmu),
            0xFA => self.cb_0xfa(mmu),
            0xFB => self.cb_0xfb(mmu),
            0xFC => self.cb_0xfc(mmu),
            0xFD => self.cb_0xfd(mmu),
            0xFE => self.cb_0xfe(mmu),
            0xFF => self.cb_0xff(mmu),
        }
    }

    fn fetch_byte(&mut self, mmu: &Mmu, pc: u16) -> u8 {
        let byte = mmu.read_byte(pc);
        byte
    }
}
