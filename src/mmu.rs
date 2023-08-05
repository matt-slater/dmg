pub const MEM_SIZE: usize = 0x10000; // 2^16, 65536

#[derive(Debug)]
pub struct Mmu {
    pub memory: [u8; MEM_SIZE],
}

impl Mmu {
    pub fn new() -> Self {
        let mut mmu = Mmu {
            memory: [0; MEM_SIZE],
        };

        let rom = include_bytes!("dmg_boot.bin");

        mmu.memory[..rom.len()].copy_from_slice(rom);

        mmu
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            _ => self.memory[addr as usize] = data,
        }
    }
}
