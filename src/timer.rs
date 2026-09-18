pub struct Timer {
    pub div: u16, // Contatore interno a 16 bit per gestire DIV e TIMA
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
        // Incrementa il contatore interno a 16 bit
        let old_div = self.div;
        self.div = self.div.wrapping_add(cycles as u16);

        // FF04 (DIV visibile) corrisponde ai 8 bit piu significativi (self.div >> 8)

        // Se il timer non e abilitato (bit 2 di TAC), non aggiorniamo TIMA
        if self.tac & 0x04 == 0 {
            return;
        }

        // Seleziona il bit di clock da monitorare in base ai bit 0-1 di TAC
        let bit_to_check = match self.tac & 0x03 {
            0 => 9, // 4096 Hz (ogni 1024 cicli T)
            1 => 3, // 262144 Hz (ogni 64 cicli T)
            2 => 5, // 65536 Hz (ogni 256 cicli T)
            3 => 7, // 16384 Hz (ogni 512 cicli T)
            _ => unreachable!(),
        };

        // Rileva la transizione da 1 a 0 del bit selezionato (Falling Edge Detector)
        let old_bit = (old_div >> bit_to_check) & 1;
        let new_bit = (self.div >> bit_to_check) & 1;

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
            // Qualsiasi scrittura su DIV azzera il contatore interno
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value & 0x07,
            _ => {}
        }
    }
}
