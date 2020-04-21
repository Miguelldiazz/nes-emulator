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
    data: [u8; 8192], //8192 kb
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

    fn get_immediate(&mut self) -> u8 {
        let ret = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;
        ret
    }

    fn get_zero(&mut self) -> u8 {
        let addr: u16 = 0x00ff & self.mem.data[self.regs.pc as usize] as u16;
        self.regs.pc += 1;
        self.mem.dagitta[addr as usize]
    }

    pub fn next_instruction(&mut self) {
        let opcode = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;

        match opcode {
            0x69 => (),
            0x65 => (),
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
