pub struct Timer {
    pub div: u16, // Contatore interno a 16 bit
    pub tima: u8, // FF05: Contatore programmabile
    pub tma: u8,  // FF06: Valore di ricarica
    pub tac: u8,  // FF07: Controllo e frequenza
    reload_delay: Option<u8>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            reload_delay: None,
        }
    }

    fn timer_signal(&self) -> bool {
        if self.tac & 0x04 == 0 {
            return false;
        }

        let bit_to_check = match self.tac & 0x03 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!(),
        };
        (self.div >> bit_to_check) & 1 != 0
    }

    fn increment_tima(&mut self) {
        if self.reload_delay.is_some() {
            return;
        }

        let (new_tima, overflow) = self.tima.overflowing_add(1);
        self.tima = new_tima;
        if overflow {
            self.reload_delay = Some(4);
        }
    }

    /// Fa avanzare il timer in base ai cicli T consumati dalla CPU
    pub fn tick(&mut self, cycles: u32, if_register: &mut u8) {
        // Avanziamo ciclo T per ciclo T per non perdere fronti multipli in un solo step CPU
        for _ in 0..cycles {
            if let Some(delay) = self.reload_delay {
                if delay == 1 {
                    self.tima = self.tma;
                    *if_register |= 0x04;
                    self.reload_delay = None;
                } else {
                    self.reload_delay = Some(delay - 1);
                }
            }

            let old_signal = self.timer_signal();
            self.div = self.div.wrapping_add(1);
            if old_signal && !self.timer_signal() {
                self.increment_tima();
            }
        }
    }
    // ---

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
            0xFF04 => {
                let old_signal = self.timer_signal();
                self.div = 0;
                if old_signal && !self.timer_signal() {
                    self.increment_tima();
                }
            }
            0xFF05 => {
                self.tima = value;
                self.reload_delay = None;
            }
            0xFF06 => self.tma = value,
            0xFF07 => {
                let old_signal = self.timer_signal();
                self.tac = value & 0x07;
                if old_signal && !self.timer_signal() {
                    self.increment_tima();
                }
            }
            _ => {}
        }
    }
    // ---
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

        // Altri 16 cicli T causano l'overflow; il reload è ritardato di 4 T-cycles.
        timer.tick(16, &mut if_reg);
        assert_eq!(timer.read_byte(0xFF05), 0);
        assert_eq!(if_reg & 0x04, 0);

        timer.tick(3, &mut if_reg);
        assert_eq!(timer.read_byte(0xFF05), 0);
        assert_eq!(if_reg & 0x04, 0);

        timer.tick(1, &mut if_reg);
        assert_eq!(timer.read_byte(0xFF05), 0x50);
        assert_ne!(if_reg & 0x04, 0);
    }

    #[test]
    fn each_tac_frequency_increments_tima_at_the_expected_period() {
        for (frequency, period) in [(0, 1024), (1, 16), (2, 64), (3, 256)] {
            let mut timer = Timer::new();
            let mut if_reg = 0;
            timer.write_byte(0xFF07, 0x04 | frequency);

            timer.tick(period - 1, &mut if_reg);
            assert_eq!(timer.tima, 0, "TAC frequency {frequency}");
            timer.tick(1, &mut if_reg);
            assert_eq!(timer.tima, 1, "TAC frequency {frequency}");
        }
    }

    #[test]
    fn timer_register_writes_and_hardware_masks_are_applied() {
        let mut timer = Timer::new();
        let mut if_reg = 0;

        timer.tick(255, &mut if_reg);
        assert_eq!(timer.read_byte(0xFF04), 0);
        timer.tick(1, &mut if_reg);
        assert_eq!(timer.read_byte(0xFF04), 1);

        timer.write_byte(0xFF04, 0xFF);
        timer.write_byte(0xFF05, 0x12);
        timer.write_byte(0xFF06, 0x34);
        timer.write_byte(0xFF07, 0xFF);
        assert_eq!(timer.read_byte(0xFF04), 0);
        assert_eq!(timer.read_byte(0xFF05), 0x12);
        assert_eq!(timer.read_byte(0xFF06), 0x34);
        assert_eq!(timer.read_byte(0xFF07), 0xFF);
    }

    #[test]
    fn disabled_timer_does_not_increment_tima() {
        let mut timer = Timer::new();
        let mut if_reg = 0;
        timer.tick(2048, &mut if_reg);

        assert_eq!(timer.tima, 0);
        assert_eq!(if_reg & 0x04, 0);
    }

    #[test]
    fn div_reset_and_tac_change_increment_tima_on_falling_edge() {
        let mut timer = Timer::new();
        let mut if_reg = 0;
        timer.write_byte(0xFF07, 0x05);
        timer.tick(8, &mut if_reg);
        timer.write_byte(0xFF04, 0);
        assert_eq!(timer.tima, 1);

        timer.tick(8, &mut if_reg);
        timer.write_byte(0xFF07, 0x04);
        assert_eq!(timer.tima, 2);
    }

    #[test]
    fn writing_tima_during_overflow_delay_cancels_reload() {
        let mut timer = Timer::new();
        let mut if_reg = 0;
        timer.write_byte(0xFF05, 0xFF);
        timer.write_byte(0xFF07, 0x05);
        timer.tick(16, &mut if_reg);
        assert_eq!(timer.tima, 0);

        timer.write_byte(0xFF05, 0x77);
        timer.tick(4, &mut if_reg);
        assert_eq!(timer.tima, 0x77);
        assert_eq!(if_reg & 0x04, 0);
    }

    #[test]
    fn tma_write_during_overflow_delay_changes_reload_value() {
        let mut timer = Timer::new();
        let mut if_reg = 0;
        timer.write_byte(0xFF05, 0xFF);
        timer.write_byte(0xFF06, 0x50);
        timer.write_byte(0xFF07, 0x05);
        timer.tick(16, &mut if_reg);

        timer.write_byte(0xFF06, 0xA5);
        timer.tick(4, &mut if_reg);
        assert_eq!(timer.tima, 0xA5);
        assert_eq!(if_reg & 0x04, 0x04);
    }
}
