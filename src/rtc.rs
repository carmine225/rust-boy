use chrono::{Datelike, Local, Timelike};

pub struct Rtc {
    // Flag o offset se il gioco ferma/modifica l'ora
    pub halt: bool,
}

impl Rtc {
    pub fn new() -> Self {
        Self { halt: false }
    }

    /// Restituisce i 5 registri nello stile esatto richiesto da MBC3
    pub fn get_mbc3_registers(&self) -> [u8; 5] {
        let now = Local::now();

        let seconds = now.second() as u8; // 0..=59
        let minutes = now.minute() as u8; // 0..=59
        let hours = now.hour() as u8; // 0..=23

        // Usa il giorno dell'anno (1..=366) per simulare il contatore giorni dell'MBC3
        let day_of_year = now.ordinal() as u16;

        let day_low = (day_of_year & 0xFF) as u8;
        let day_high = ((day_of_year >> 8) & 0x01) as u8; // Bit 0: 9° bit dei giorni

        // Bit 6 = Halt, Bit 0 = Day Bit 8
        let control = (day_high) | if self.halt { 0x40 } else { 0x00 };

        [seconds, minutes, hours, day_low, control]
    }
}
