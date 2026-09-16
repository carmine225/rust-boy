use chrono::{DateTime, Local};

pub struct Rtc {
    pub halt: bool,
    seconds: u8,
    minutes: u8,
    hours: u8,
    days: u16,
    carry: bool,
    last_update: DateTime<Local>,
}

impl Rtc {
    pub fn new() -> Self {
        Self {
            halt: false,
            seconds: 0,
            minutes: 0,
            hours: 0,
            days: 0,
            carry: false,
            last_update: Local::now(),
        }
    }

    fn update(&mut self) {
        let now = Local::now();
        if !self.halt {
            let elapsed = now
                .signed_duration_since(self.last_update)
                .num_seconds()
                .max(0) as u64;
            let current = self.seconds as u64
                + self.minutes as u64 * 60
                + self.hours as u64 * 3600
                + self.days as u64 * 86400
                + elapsed;
            let day = current / 86400;
            self.days = (day % 512) as u16;
            self.carry |= day >= 512;
            let day_seconds = current % 86400;
            self.hours = (day_seconds / 3600) as u8;
            self.minutes = ((day_seconds % 3600) / 60) as u8;
            self.seconds = (day_seconds % 60) as u8;
        }
        self.last_update = now;
    }

    pub fn get_mbc3_registers(&mut self) -> [u8; 5] {
        self.update();
        [
            self.seconds,
            self.minutes,
            self.hours,
            self.days as u8,
            ((self.days >> 8) as u8 & 0x01)
                | if self.halt { 0x40 } else { 0 }
                | if self.carry { 0x80 } else { 0 },
        ]
    }

    pub fn set_mbc3_register(&mut self, index: usize, value: u8) {
        self.update();
        match index {
            0 => self.seconds = value % 60,
            1 => self.minutes = value % 60,
            2 => self.hours = value % 24,
            3 => self.days = (self.days & 0x100) | value as u16,
            4 => {
                self.days = (self.days & 0xFF) | ((value as u16 & 0x01) << 8);
                self.halt = value & 0x40 != 0;
                self.carry = value & 0x80 != 0;
            }
            _ => {}
        }
        self.last_update = Local::now();
    }
}
