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
    data: [u8; 2048], //2 kb
}

impl Memory {
    pub fn new() -> Memory {
        Memory { data: [0; 2048] }
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

    fn adc(&mut self, value: u8) {
        let carry = self.regs.p & 0x01;
        let aux = self.regs.a;
        self.regs.a = self.regs.a.wrapping_add(value).wrapping_add(carry);

        if aux > self.regs.a {
            self.regs.p = self.regs.p | 0x01; //c =1
        } else {
            self.regs.p = self.regs.p & 0xfe; //c = 0
        }

        if aux & 0x80 == self.regs.a & 0x80 {
            self.regs.p = self.regs.p | 0x40; //v = 1
        } else {
            self.regs.p = self.regs.p & 0xbf; //v = 0
        }

        if self.regs.a & 0x80 == 0x80 {
            self.regs.p = self.regs.p | 0x80; //n = 1
        } else {
            self.regs.p = self.regs.p & 0x7f; //n = 0
        }

        if self.regs.a == 0x00 {
            self.regs.p = self.regs.p | 0x02; //z = 1
        } else {
            self.regs.p = self.regs.p & 0xfd; //z = 0
        }

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

    pub fn next_instruction(&mut self) {
        let opcode = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;
        let value: u8;

        match opcode {
            0x69 => {
                value = self.get_immediate();
                self.adc(value);
            },
            0x65 => {
                value = self.get_zero();
                self.adc(value);
            },
            0x75 => (),
            0x6d => (),
            0x7d => (),
            0x79 => (),
            0x61 => (),
            0x71 => (),
            _ => println!("Error"),
        }
    }
}
