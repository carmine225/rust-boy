use crate::{cpu::Cpu, mmu::Mmu, timer::Timer};
pub struct System {
    pub cpu: Cpu,
    //pub ppu: Ppu,
    //pub apu: Apu,
    pub mmu: Mmu,
    pub timer: Timer,
    cycles: u32,
}

impl System {
    pub fn new(mmu: Mmu) -> Self {
        System {
            cpu: Cpu::new(),
            mmu,
            //ppu: Ppu::new(),
            //apu: Apu::new(),
            timer: Timer::new(),
            cycles: 0,
        }
    }
    fn step(&mut self) {
        self.cycles = self.cpu.step(&mut self.mmu, &mut self.timer);
        self.timer
            .tick(self.cycles, &mut self.mmu.io_registers[0x0F]);
    }
    pub fn step_frame(&mut self) {
        const CYCLES_PER_FRAME: u32 = 70224;
        let mut frame_cycles = 0;
        while frame_cycles < CYCLES_PER_FRAME {
            self.step();
            frame_cycles += self.cycles;
        }
    }
}
