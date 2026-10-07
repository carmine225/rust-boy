pub struct Timer {
    pub div: u16, // Contatore interno a 16 bit
    pub tima: u8, // FF05: Contatore programmabile
    pub tma: u8,  // FF06: Valore di ricarica
    pub tac: u8,  // FF07: Controllo e frequenza
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    /// Fa avanzare il timer in base ai cicli T consumati dalla CPU
    pub fn tick(&mut self, cycles: u32, if_register: &mut u8) {
        let bit_to_check = match self.tac & 0x03 {
            0 => 9, // 4096 Hz   (ogni 1024 cicli T -> bit 9)
            1 => 3, // 262144 Hz (ogni 16 cicli T   -> bit 3)
            2 => 5, // 65536 Hz  (ogni 64 cicli T   -> bit 5)
            3 => 7, // 16384 Hz  (ogni 256 cicli T  -> bit 7)
            _ => unreachable!(),
        };

        let timer_enabled = (self.tac & 0x04) != 0;

        // Avanziamo ciclo T per ciclo T per non perdere fronti multipli in un solo step CPU
        for _ in 0..cycles {
            let old_div = self.div;
            self.div = self.div.wrapping_add(1);

            if timer_enabled {
                let old_bit = (old_div >> bit_to_check) & 1;
                let new_bit = (self.div >> bit_to_check) & 1;

                // Transizione 1 -> 0 (Falling Edge)
                if old_bit == 1 && new_bit == 0 {
                    let (new_tima, overflow) = self.tima.overflowing_add(1);
                    if overflow {
                        self.tima = self.tma; // Ricarica da TMA
                        *if_register |= 0x04; // Richiede interrupt Timer (bit 2 di IF)
                    } else {
                        self.tima = new_tima;
                    }
                }
            }
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8, // I bit 3-7 ritornano 1 su hardware DMG
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // Scrittura su DIV azzera il contatore interno
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value & 0x07,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_div_write_resets() {
        let mut timer = Timer::new();
        let mut if_reg = 0;
        timer.tick(256 * 4, &mut if_reg);
        assert_ne!(timer.read_byte(0xFF04), 0);

        timer.write_byte(0xFF04, 0xFF);
        assert_eq!(timer.read_byte(0xFF04), 0);
    }

    #[test]
    fn test_timer_overflow_triggers_if() {
        let mut timer = Timer::new();
        let mut if_reg = 0;

        timer.write_byte(0xFF06, 0x50); // TMA = 0x50
        timer.write_byte(0xFF05, 0xFE); // TIMA = 0xFE
        timer.write_byte(0xFF07, 0x05); // Enable | Clock 262144Hz (bit 3)

        // 16 cicli T -> TIMA diventa 0xFF
        timer.tick(16, &mut if_reg);
        assert_eq!(timer.read_byte(0xFF05), 0xFF);
        assert_eq!(if_reg & 0x04, 0);

        // Altri 16 cicli T -> Overflow! TIMA va a TMA (0x50) e IF bit 2 si attiva
        timer.tick(16, &mut if_reg);
        assert_eq!(timer.read_byte(0xFF05), 0x50);
        assert_ne!(if_reg & 0x04, 0);
    }
}
