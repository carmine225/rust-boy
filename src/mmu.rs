pub struct Mmu {
    data: [u8; 65536],
}

impl Mmu {
    pub fn new() -> Self {
        Mmu { data: [0; 65536] }
    }
    pub fn read_byte(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }
}
