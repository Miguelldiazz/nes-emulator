struct Registers {
    a: u8,   //accumulator
    x: u8,   //index
    y: u8,   //index
    pc: u16, //program counter
    sp: u8,  //stack pointer
    p: u8,   //status register
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0x0,
            x: 0x0,
            y: 0x0,
            pc: 0x0,
            sp: 0xfd,
            p: 0x34,
        }
    }
}

struct Memory {
    data: [u8; 8192], //ram (0000-3fff), i/o (4000-7fff), rom(8000-ffff)
}

impl Memory {
    pub fn new() -> Memory {
        Memory { data: [0; 8192] }
    }
}

pub struct Cpu {
    mem: Memory,
    regs: Registers,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            mem: Memory::new(),
            regs: Registers::new(),
        }
    }

    fn set_zero_flag(&mut self) {
        if self.regs.a == 0x00 {
            self.regs.p = self.regs.p | 0x02; //z = 1
        } else {
            self.regs.p = self.regs.p & 0xfd; //z = 0
        }
    }

    fn set_negative_flag(&mut self) {
        if self.regs.a & 0x80 == 0x80 {
            self.regs.p = self.regs.p | 0x80; //n = 1
        } else {
            self.regs.p = self.regs.p & 0x7f; //n = 0
        }
    }

    fn set_overflow_flag(&mut self, aux: u8) {
        if aux & 0x80 == self.regs.a & 0x80 {
            self.regs.p = self.regs.p | 0x40; //v = 1
        } else {
            self.regs.p = self.regs.p & 0xbf; //v = 0
        }
    }

    fn set_carry_flag(&mut self, aux: u8) {
        if aux > self.regs.a {
            self.regs.p = self.regs.p | 0x01; //c =1
        } else {
            self.regs.p = self.regs.p & 0xfe; //c = 0
        }
    }

    fn adc(&mut self, value: u8) {
        let carry = self.regs.p & 0x01;
        let aux = self.regs.a;
        self.regs.a = self.regs.a.wrapping_add(value).wrapping_add(carry);
        self.set_carry_flag(aux);
        self.set_overflow_flag(aux);
        self.set_negative_flag();        
        self.set_zero_flag();
    }

    fn and(&mut self, value: u8) {
        self.regs.a = self.regs.a & value;
        self.set_zero_flag();
        self.set_negative_flag();
    }

    fn get_immediate(&mut self) -> u8 {
        let ret = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;
        ret
    }

    fn get_zero(&mut self) -> u8 {
        let addr: u16 = 0x00ff & self.mem.data[self.regs.pc as usize] as u16;
        self.regs.pc += 1;
        self.mem.data[addr as usize]
    }

    fn get_zero_x(&mut self) -> u8 {
        let addr: u8 = self.mem.data[self.regs.pc as usize].wrapping_add(self.regs.x);
        self.regs.pc += 1;
        self.mem.data[addr as usize]
    }

    fn get_absolute(&mut self) -> u8 {
        let mut addr: u16 = self.mem.data[self.regs.pc as usize] as u16;
        self.regs.pc += 1;
        addr += (self.mem.data[self.regs.pc as usize] as u16) << 8;
        self.regs.pc += 1;
        self.mem.data[addr as usize]
    }

    fn get_absolute_x(&mut self) -> u8 {
        let mut addr: u16 = self.mem.data[self.regs.pc as usize] as u16;
        self.regs.pc += 1;
        addr += (self.mem.data[self.regs.pc as usize] as u16) << 8;
        self.regs.pc += 1;
        addr = addr.wrapping_add(self.regs.x as u16);
        self.mem.data[addr as usize]
    }

    fn get_absolute_y(&mut self) -> u8 {
        let mut addr: u16 = self.mem.data[self.regs.pc as usize] as u16;
        self.regs.pc += 1;
        addr += (self.mem.data[self.regs.pc as usize] as u16) << 8;
        self.regs.pc += 1;
        addr = addr.wrapping_add(self.regs.y as u16);
        self.mem.data[addr as usize]
    }

    fn get_indirect_x(&mut self) -> u8 {
        let mut zero_addr: u8 = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;
        zero_addr = zero_addr.wrapping_add(self.regs.x);
        let mut addr: u16 = self.mem.data[zero_addr as usize] as u16;
        addr += (self.mem.data[(zero_addr.wrapping_add(1)) as usize] as u16) << 8;

        self.mem.data[addr as usize]
    }

    fn get_indirect_y(&mut self) -> u8 {
        let zero_addr: u8 = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;
        let mut addr: u16 = self.mem.data[zero_addr as usize] as u16;
        addr += (self.mem.data[(zero_addr.wrapping_add(1)) as usize] as u16) << 8;
        addr = addr.wrapping_add(self.regs.y as u16);

        self.mem.data[addr as usize]
    }

    fn bcc(&mut self) {
        let jump = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;
        if self.regs.p & 0x01 == 0 {
            self.regs.pc += jump as u16;
        }
    }

    fn bcs(&mut self) {
        let jump = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;
        if self.regs.p & 0x01 != 1 {
            self.regs.pc += jump as u16;
        }
    }
    
    fn beq(&mut self) {
        let jump = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;
        if self.regs.p & 0x02 == 0x02 {
            self.regs.pc += jump as u16;
        }
    }

    pub fn next_instruction(&mut self) {
        let opcode = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;
        let value: u8;

        match opcode {
            //ADC
            0x69 => {
                value = self.get_immediate();
                self.adc(value);
            },
            0x65 => {
                value = self.get_zero();
                self.adc(value);
            },
            0x75 => {
                value = self.get_zero_x();
                self.adc(value);
            },
            0x6d => {
                value = self.get_absolute();
                self.adc(value);
            },
            0x7d => {
                value = self.get_absolute_x();
                self.adc(value);
            },
            0x79 => {
                value = self.get_absolute_y();
                self.adc(value);
            },
            0x61 => {
                value = self.get_indirect_x();
                self.adc(value);
            },
            0x71 => {
                value = self.get_indirect_y();
                self.adc(value);
            },
            //AND
            0x29 => {
                value = self.get_immediate();
                self.and(value);
            },
            0x25 => {
                value = self.get_zero();
                self.and(value);
            },
            0x35 => {
                value = self.get_zero_x();
                self.and(value);
            },
            0x2d => {
                value = self.get_absolute();
                self.and(value);
            },
            0x3d => {
                value = self.get_absolute_x();
                self.and(value);
            },
            0x39 => {
                value = self.get_absolute_y();
                self.and(value);
            },
            0x21 => {
                value = self.get_indirect_x();
                self.and(value);
            },
            0x31 => {
                value = self.get_indirect_y();
                self.and(value);
            },
            //ASL
            0x0a => (),
            0x06 => (),
            0x16 => (),
            0x0e => (),
            0x1e => (),
            //BCC
            0x90 => self.bcc(),
            //BCS
            0xb0 => self.bcs(),
            //BEQ
            0xf0 => self.beq(),
            _ => println!("Error"),
        }
    }
}
