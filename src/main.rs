use dmg::cpu;
use dmg::mmu;
use dmg::ppu;

fn main() {
    let mut cpu = cpu::Cpu::new();
    let mut ppu = ppu::Ppu::new();
    let mut mmu = mmu::Mmu::new();

    loop {
        cpu.execute(&mut mmu);
        ppu.tick(&mut mmu);
        //println!("{:?}", cpu);
    }
}
