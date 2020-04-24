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
            pc: 0x8000,
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

    fn bne(&mut self) {
        let jump = self.mem.data[self.regs.pc as usize];
        self.regs.pc += 1;
        if self.regs.p & 0x02 != 0x02 {
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
            //BIT
            0x24 => (),
            0x2c => (),
            //BMI
            0x30 => (),
            //BNE
            0xd0 => self.bne(),
            //BPL
            0x10 => (),
            //BRK
            0x00 => (),
            //BVC
            0x50 => (),
            //BVS
            0x70 => (),
            //CLC
            0x18 => (),
            //CLD
            0xd8 => (),
            //CLI
            0x58 => (),
            //CLV
            0xb8 => (),
            //CMP
            0xc9 => (),
            0xc5 => (),
            0xd5 => (),
            0xcd => (),
            0xdd => (),
            0xd9 => (),
            0xc1 => (),
            0xd1 => (),
            //CPX
            0xe0 => (),
            0xe4 => (),
            0xec => (),
            //CPY
            0xc0 => (),
            0xc4 => (),
            0xcc => (),
            //DEC
            0xc6 => (),
            0xd6 => (),
            0xce => (),
            0xde => (),
            //DEX
            0xca => (),
            //DEY
            0x88 => (),
            //EOR
            0x49 => (),
            0x45 => (),
            0x55 => (),
            0x4d => (),
            0x5d => (),
            0x59 => (),
            0x41 => (),
            0x51 => (),
            //INC
            0xe6 => (),
            0xf6 => (),
            0xee => (), 
            0xfe => (),
            //INX
            0xe8 => (),
            //INY
            0xc8 => (),
            //JMP
            0x4c => (),
            0x6c => (),
            //JSR
            0x20 => (),
            //LDA
            0xa9 => (),
            0xa5 => (),
            0xb5 => (),
            0xad => (),
            0xbd => (),
            0xb9 => (),
            0xa1 => (),
            0xb1 => (),
            //LDX
            0xa2 => (),
            0xa6 => (),
            0xb6 => (),
            0xae => (),
            0xbe => (),
            //LDY
            0xa0 => (),
            0xa4 => (),
            0xb4 => (),
            0xac => (),
            0xbc => (),
            //LSR
            0x4a => (),
            0x46 => (),
            0x56 => (),
            0x4e => (),
            0x5e => (),
            //NOP
            0xea => (),
            //ORA
            0x09 => (),
            0x05 => (),
            0x15 => (),
            0x0d => (), 
            0x1d => (),
            0x19 => (),
            0x01 => (),
            0x11 => (),
            //PHA
            0x48 => (),
            //PHP
            0x08 => (),
            //PLA
            0x68 => (),
            //PLP
            0x28 => (),
            //ROL
            0x2a => (),
            0x26 => (),
            0x36 => (),
            0x2e => (),
            0x3e => (),
            //ROR
            0x6a => (),
            0x66 => (),
            0x76 => (),
            0x6e => (),
            0x7e => (),
            //RTI
            0x40 => (),
            //RTS
            0x60 => (),
            //SBC
            0xe9 => (),
            0xe5 => (),
            0xf5 => (),
            0xed => (),
            0xfd => (),
            0xf9 => (),
            0xe1 => (),
            0xf1 => (),
            //SEC
            0x38 => (),
            //SED
            0xf8 => (),
            //SEI
            0x78 => (),
            //STA
            0x85 => (),
            0x95 => (),
            0x8d => (),
            0x9d => (),
            0x99 => (),
            0x81 => (),
            0x91 => (),
            //STX
            0x86 => (),
            0x96 => (),
            0x8e => (),
            //STY
            0x84 => (),
            0x94 => (),
            0x8c => (),
            //TAX
            0xaa => (),
            //TAY
            0xa8 => (),
            //TSX
            0xba => (),
            //TXA
            0x8a => (),
            //TXS
            0x9a => (),
            //TYA
            0x98 => (),
            _ => println!("Error"),
        }
    }
}
