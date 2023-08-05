use std::fmt;

const Z_FLAG: u8 = 0x80; //  0b1000_0000
const N_FLAG: u8 = 0x40; //  0b0100_0000
const HC_FLAG: u8 = 0x20; // 0b0010_0000
const C_FLAG: u8 = 0x10; //  0b0001_0000

use crate::mmu;

pub struct Cpu {
    mmu: mmu::Mmu,
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    f: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            mmu: mmu::Mmu::new(),
            pc: 0,
            sp: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            f: 0,
        }
    }

    pub fn execute(&mut self, mmu: &mut mmu::Mmu) {
        let opcode = self.mmu.memory[self.pc as usize];
        //println!("executing opcode: {:#04x}", opcode);
        //std::thread::sleep(std::time::Duration::from_secs(20));

        match opcode {
            0x31 => {
                // load next two bytes into SP.
                self.sp = u16_from_u8s(mmu.read_byte(self.pc + 2), mmu.read_byte(self.pc + 1));

                self.pc += 3;
            }
            0xaf => {
                // XOR a with itself.
                self.a ^= self.a;
                self.pc += 1;
            }
            0x21 => {
                // load next two bytes into HL.
                self.h = self.mmu.read_byte(self.pc + 2);
                self.l = self.mmu.read_byte(self.pc + 1);
                self.pc += 3;
            }
            0x32 => {
                // decrement contents of HL and write contents of A to addr in HL.
                let hl_data = self.hl();
                self.set_hl(hl_data - 1);
                self.mmu.write_byte(self.hl(), self.a);
                self.pc += 1;
            }
            0xCB => {
                let cb_code = self.mmu.read_byte(self.pc + 1);
                self.execute_cb(cb_code);
                self.pc += 2;
            }
            0x20 => {
                // conditionally jump the pc the number of the next byte as a signed int if zero flag is not set.
                if !self.z() {
                    let jump = self.mmu.read_byte(self.pc + 1);
                    self.pc += 2;
                    self.pc = self.pc.wrapping_add((jump as i8) as u16);
                } else {
                    self.pc += 2;
                }
            }
            0x0e => {
                // load next 8 bits into C.
                self.c = self.mmu.read_byte(self.pc + 1);
                self.pc += 2;
            }
            0x3e => {
                // load next 8 bits into A.
                self.a = self.mmu.read_byte(self.pc + 1);
                self.pc += 2;
            }
            0xe2 => {
                // load a into addr 0xff00 + c.
                self.mmu.write_byte(0xff00 + (self.c as u16), self.a);
                self.pc += 1;
            }
            0x0c => {
                // increment C.
                let hc = eight_bit_hc(self.c, 1);
                let result = self.c.wrapping_add(1);
                if result == 0 {
                    self.set_z(true);
                } else {
                    self.set_z(false);
                }
                if hc {
                    self.set_hc(true);
                } else {
                    self.set_hc(false);
                }

                self.set_n(false);

                self.pc += 1;
            }
            0x77 => {
                // load A into memory location specified by HL.
                self.mmu.write_byte(self.hl(), self.a);
                self.pc += 1;
            }
            0xe0 => {
                let operand = self.mmu.read_byte(self.pc + 1);
                self.mmu.write_byte(0xff00 + (operand as u16), self.a);
                self.pc += 2;
            }
            0x11 => {
                self.set_de(u16_from_u8s(
                    self.mmu.read_byte(self.pc + 2),
                    self.mmu.read_byte(self.pc + 1),
                ));

                self.pc += 3;
            }
            0x1a => {
                // load contents of addr pointed to by DE into A.
                self.a = self.mmu.read_byte(self.de());
                self.pc += 1;
            }
            0xcd => {
                // call

                let pc = u8s_from_16(self.pc);
                self.mmu.write_byte(self.sp - 1, pc.0);

                self.mmu.write_byte(self.sp - 2, pc.1);

                self.sp -= 2;

                self.pc += 3;
            }
            0x13 => {
                // increment DE.
                self.set_de(self.de() + 1);
                self.pc += 1;
            }
            0x7b => {
                // load E into A.
                self.a = self.e;
                self.pc += 1;
            }
            0xfe => {
                // compare a with next 8 bits by subtraction.
                let operand = self.mmu.read_byte(self.pc + 1);
                let result = self.a.wrapping_sub(operand);

                self.set_n(true);
                if result == 0 {
                    self.set_z(true);
                } else {
                    self.set_z(false);
                }
                self.pc += 2;
            }
            0x06 => {
                // load next 8 bits into B.
                self.b = self.mmu.read_byte(self.pc + 1);
                self.pc += 2;
            }
            0x22 => {
                // increment contents of HL and write contents of A to addr in HL.
                let hl_data = self.hl();

                self.set_hl(hl_data + 1);
                self.mmu.write_byte(self.hl(), self.a);
                self.pc += 1;
            }
            0x23 => {
                self.set_hl(self.hl() + 1);
                self.pc += 1;
            }
            0x05 => {
                // decrement B.
                let result = self.b.wrapping_sub(1);
                self.set_n(true);

                self.b = result;
                if result == 0 {
                    self.set_z(true);
                } else {
                    self.set_z(false);
                }

                self.pc += 1;
            }
            0xea => {
                let addr = u16_from_u8s(
                    self.mmu.read_byte(self.pc + 2),
                    self.mmu.read_byte(self.pc + 1),
                );

                self.mmu.write_byte(addr, self.a);

                self.pc += 3;
            }
            0x3d => {
                // decrement A.
                let result = self.a.wrapping_sub(1);
                self.set_n(true);

                self.a = result;
                if result == 0 {
                    self.set_z(true);
                } else {
                    self.set_z(false)
                }

                self.pc += 1;
            }
            0x28 => {
                // conditionally jump the pc the number of the next byte as a signed int if zero flag is  set.
                if self.z() {
                    let jump = self.mmu.read_byte(self.pc + 1);
                    self.pc += 2;
                    self.pc = self.pc.wrapping_add((jump as i8) as u16);
                } else {
                    self.pc += 2;
                }
            }
            0x0d => {
                // decrement C.
                let result = self.c.wrapping_sub(1);
                self.set_n(true);

                self.c = result;
                if result == 0 {
                    self.set_z(true);
                } else {
                    self.set_z(false)
                }

                self.pc += 1;
            }
            0x2e => {
                // load next 8 bits into L.
                self.l = self.mmu.read_byte(self.pc + 1);
                self.pc += 2;
            }
            0x18 => {
                // jump relative.
                let jump = self.mmu.read_byte(self.pc + 1);
                self.pc += 2;
                self.pc = self.pc.wrapping_add((jump as i8) as u16);
            }
            0x67 => {
                // load A into H.
                self.h = self.a;
                self.pc += 1;
            }
            0x57 => {
                // load A into D.
                self.d = self.a;
                self.pc += 1;
            }
            0x04 => {
                // increment B.
                let result = self.b.wrapping_add(1);
                self.set_n(false);
                self.set_hc(eight_bit_hc(self.b, 1));
                self.b = result;
                if result == 0 {
                    self.set_z(true);
                } else {
                    self.set_z(false);
                }

                self.pc += 1;
            }
            0x1e => {
                // load next 8 bits into E.
                self.e = self.mmu.read_byte(self.pc + 1);
                self.pc += 2;
            }
            0xf0 => {
                let addr = 0xff00 | self.mmu.read_byte(self.pc + 1) as u16;
                //println!("addr: {:#06x}", addr);
                //println!("at addr: {:#04x}", mmu.read_byte(addr));
                // let data = mmu.read_byte(addr);
                self.a = mmu.read_byte(addr);

                //println!("in a register: {:#04x}", self.a);

                self.pc += 2;
            }
            // 0x1d => {
            //     // decrement E.
            //     let result = self.e.wrapping_sub(1);
            //     self.set_n(true);

            //     self.e = result;
            //     if result == 0 {
            //         self.set_z(true);
            //     } else {
            //         self.set_z(false)
            //     }

            //     self.pc += 1;
            // }
            0x24 => {
                // increment H.
                let hc = eight_bit_hc(self.h, 1);
                let result = self.h.wrapping_add(1);
                if result == 0 {
                    self.set_z(true);
                } else {
                    self.set_z(false);
                }
                if hc {
                    self.set_hc(true);
                } else {
                    self.set_hc(false);
                }

                self.set_n(false);

                self.pc += 1;
            }
            0x7c => {
                // load contents of H into A
                self.a = self.h;
                self.pc += 1;
            }
            0x90 => {
                // A-B store in A
                let result = self.a.wrapping_sub(self.b);
                self.set_n(true);
                self.set_z(result == 0);
                self.a = result;
                self.pc += 1;
            }
            0x15 => {
                // decrement D.
                let result = self.d.wrapping_sub(1);
                self.set_n(true);

                self.d = result;
                self.set_z(result == 0);

                self.pc += 1;
            }
            0x16 => {
                // load next 8 bits into D.
                self.d = self.mmu.read_byte(self.pc + 1);
                self.pc += 2;
            }
            // 0xbe => {
            //     // compare a with contents of HL.
            //     let operand = self.mmu.read_byte(self.hl());
            //     let result = self.a.wrapping_sub(operand);

            //     self.set_n(true);
            //     self.set_z(result == 0);
            //     self.pc += 1;
            // }
            _ => unimplemented!("opcode {:#04x} not implemented", opcode),
        }
    }

    fn z(&self) -> bool {
        ((self.f & 0b1000_0000) >> 7) == 1
    }

    fn set_z(&mut self, bit: bool) {
        match bit {
            true => {
                self.f |= Z_FLAG;
            }
            false => {
                self.f &= !Z_FLAG;
            }
        }
    }

    fn n(&self) -> bool {
        ((self.f & 0b0100_0000) >> 6) == 1
    }

    fn set_n(&mut self, bit: bool) {
        match bit {
            true => {
                self.f |= N_FLAG;
            }
            false => {
                self.f &= !N_FLAG;
            }
        }
    }

    fn hc(&self) -> bool {
        ((self.f & 0b0010_0000) >> 5) == 1
    }

    fn set_hc(&mut self, bit: bool) {
        match bit {
            true => {
                self.f |= HC_FLAG;
            }
            false => {
                self.f &= !HC_FLAG;
            }
        }
    }

    fn hl(&self) -> u16 {
        u16_from_u8s(self.h, self.l)
    }

    fn set_hl(&mut self, data: u16) {
        let u8s = u8s_from_16(data);
        self.h = u8s.0;
        self.l = u8s.1;
    }

    fn de(&self) -> u16 {
        u16_from_u8s(self.d, self.e)
    }

    fn set_de(&mut self, data: u16) {
        let u8s = u8s_from_16(data);
        self.d = u8s.0;
        self.e = u8s.1;
    }

    fn execute_cb(&mut self, opcode: u8) {
        match opcode {
            0x7c => {
                self.set_z((self.h & 0x80) >> 7 == 0);
            }
            _ => unimplemented!("0xcb opcode {:#04x} not implemented", opcode),
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A: {:#04x} F: {:#b} B: {:#04x} C:{:#04x} D: {:#04x} E: {:#04x} H: {:#04x} L: {:#04x} PC: {:#06x} SP: {:#06x}",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.pc, self.sp
        )
    }
}

fn u16_from_u8s(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

fn u8s_from_16(data: u16) -> (u8, u8) {
    let msb = (data >> 8) as u8;
    let lsb = (data & 0x00ff) as u8;

    (msb, lsb)
}

fn eight_bit_hc(a: u8, b: u8) -> bool {
    (((a & 0xF) + (b & 0xF)) & 0x10) == 0x10
}

fn sixteen_bit_hc(a: u16, b: u16) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u16_from_u8s() {
        assert_eq!(u16_from_u8s(0xff, 0xfe), 0xfffe);
    }
}
