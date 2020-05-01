use crate::memory::Memory;
use crate::utils::*;

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

    fn set_zero_flag(&mut self, status: bool) {
        if status {
            self.regs.p = self.regs.p | 0x02; //z = 1
        } else {
            self.regs.p = self.regs.p & 0xfd; //z = 0
        }
    }

    fn set_negative_flag(&mut self, status: bool) {
        if status {
            self.regs.p = self.regs.p | 0x80; //n = 1
        } else {
            self.regs.p = self.regs.p & 0x7f; //n = 0
        }
    }

    fn set_overflow_flag_ex(&mut self, aux: u8) {
        if aux & 0x80 == self.regs.a & 0x80 {
            self.regs.p = self.regs.p | 0x40; //v = 1
        } else {
            self.regs.p = self.regs.p & 0xbf; //v = 0
        }
    }

    fn set_overflow_flag(&mut self, status: bool) {
        if status {
            self.regs.p = self.regs.p | 0x40; //v = 1
        } else {
            self.regs.p = self.regs.p & 0xbf; //v = 0
        }
    }

    fn set_carry_flag(&mut self, status: bool) {
        if status {
            self.regs.p = self.regs.p | 0x01; //c =1
        } else {
            self.regs.p = self.regs.p & 0xfe; //c = 0
        }
    }

    fn set_decimal_flag(&mut self, status: bool) {
        if status {
            self.regs.p = self.regs.p | 0x08; //d = 1
        } else {
            self.regs.p = self.regs.p & 0xf7; //d = 0
        }
    }

    fn set_interrupt_flag(&mut self, status: bool) {
        if status {
            self.regs.p = self.regs.p | 0x04; //i = 1
        } else {
            self.regs.p = self.regs.p & 0xfb; //i = 0
        }
    }

    fn check_wrap_carry(&mut self, aux: u8) {
        if aux > self.regs.a {
            self.set_carry_flag(true);
        } else {
            self.set_carry_flag(false);
        }
    }

    fn adc(&mut self, value: u8) {
        let carry = self.regs.p & 0x01;
        let aux = self.regs.a;
        self.regs.a = self.regs.a.wrapping_add(value).wrapping_add(carry);
        self.check_wrap_carry(aux);
        self.set_overflow_flag_ex(aux);
        self.set_negative_flag(get_bit_at(self.regs.a, NEGATIVE) == SET);        
        self.set_zero_flag(self.regs.a == 0);
    }

    fn and(&mut self, value: u8) {
        self.regs.a = self.regs.a & value;
        self.set_zero_flag(self.regs.a == 0);
        self.set_negative_flag(get_bit_at(self.regs.a, NEGATIVE) == SET);
    }

    fn get_immediate(&mut self) -> u16 {
        let ret = self.regs.pc;
        self.regs.pc += 1;
        ret
    }

    fn get_zero(&mut self) -> u16 {
        let addr: u16 = 0xff & self.mem.read(self.regs.pc) as u16;
        self.regs.pc += 1;
        addr
    }

    fn get_zero_x(&mut self) -> u16 {
        let addr: u8 = self.mem.read(self.regs.pc).wrapping_add(self.regs.x);
        self.regs.pc += 1;
        addr as u16
    }

    fn get_absolute(&mut self) -> u16 {
        let mut addr: u16 = self.mem.read(self.regs.pc) as u16;
        self.regs.pc += 1;
        addr += (self.mem.read(self.regs.pc) as u16) << 8;
        self.regs.pc += 1;
        addr
    }

    fn get_absolute_x(&mut self) -> u16 {
        let mut addr: u16 = self.mem.read(self.regs.pc) as u16;
        self.regs.pc += 1;
        addr += (self.mem.read(self.regs.pc) as u16) << 8;
        self.regs.pc += 1;
        addr = addr.wrapping_add(self.regs.x as u16);
        addr
    }

    fn get_absolute_y(&mut self) -> u16 {
        let mut addr: u16 = self.mem.read(self.regs.pc) as u16;
        self.regs.pc += 1;
        addr += (self.mem.read(self.regs.pc) as u16) << 8;
        self.regs.pc += 1;
        addr = addr.wrapping_add(self.regs.y as u16);

        addr
    }

    fn get_indirect_x(&mut self) -> u16 {
        let mut zero_addr: u8 = self.mem.read(self.regs.pc);
        self.regs.pc += 1;
        zero_addr = zero_addr.wrapping_add(self.regs.x);
        let mut addr: u16 = self.mem.read(zero_addr as u16) as u16;
        addr += (self.mem.read(zero_addr.wrapping_add(1) as u16) as u16) << 8;

        addr
    }

    fn get_indirect_y(&mut self) -> u16 {
        let zero_addr: u8 = self.mem.read(self.regs.pc);
        self.regs.pc += 1;
        let mut addr: u16 = self.mem.read(zero_addr as u16) as u16;
        addr += (self.mem.read(zero_addr.wrapping_add(1) as u16) as u16) << 8;
        addr = addr.wrapping_add(self.regs.y as u16);
        
        addr
    }

    fn branch_if(&mut self, bit: u8, set: u8) {
        let jump = self.mem.read(self.regs.pc);
        self.regs.pc += 1;
        if get_bit_at(self.regs.p, bit) == set {
            self.regs.pc += jump as u16;
        }
    }

    fn asl_acc(&mut self) {
        self.set_carry_flag(get_bit_at(self.regs.a, NEGATIVE) != 0);  //c = 1 if bits[7] == 1 else c = 0
        self.regs.a = self.regs.a.wrapping_mul(2);
        self.set_negative_flag(get_bit_at(self.regs.a, NEGATIVE) == SET);
        self.set_zero_flag(self.regs.a == 0);
    }

    fn asl_mem(&mut self, addr: u16) {
        let mut value = self.mem.read(addr);
        self.set_carry_flag(get_bit_at(value, NEGATIVE) != 0);  //c = 1 if bits[7] == 1 else c = 0        
        value = value.wrapping_mul(2);
        self.set_negative_flag(get_bit_at(value, NEGATIVE) == SET);
        self.set_zero_flag(value == 0);
        self.mem.write(addr, value);
    }

    fn cmp(&mut self, value: u8) {
        let result = self.regs.a.wrapping_sub(value);
        self.set_carry_flag(self.regs.a >= value);
        self.set_zero_flag(self.regs.a == value);
        self.set_negative_flag(get_bit_at(result, NEGATIVE) == SET);
    }

    fn cpx(&mut self, value: u8) {
        let result = self.regs.x.wrapping_sub(value);
        self.set_carry_flag(self.regs.x >= value);
        self.set_zero_flag(self.regs.x == value);
        self.set_negative_flag(get_bit_at(result, NEGATIVE) == SET);
    }

    fn cpy(&mut self, value: u8) {
        let result = self.regs.y.wrapping_sub(value);
        self.set_carry_flag(self.regs.y >= value);
        self.set_zero_flag(self.regs.y == value);
        self.set_negative_flag(get_bit_at(result, NEGATIVE) == SET);
    }

    fn dec(&mut self, addr: u16) {
        let mut value = self.mem.read(addr);
        value = value.wrapping_sub(1);
        self.set_zero_flag(value == 0);
        self.set_negative_flag(get_bit_at(value, NEGATIVE) == SET);
        self.mem.write(addr, value);
    }

    fn dex(&mut self) {
        self.regs.x = self.regs.x.wrapping_sub(1);
        self.set_zero_flag(self.regs.x == 0);
        self.set_negative_flag(get_bit_at(self.regs.x, NEGATIVE) == SET);
    }

    fn dey(&mut self) {
        self.regs.y = self.regs.y.wrapping_sub(1);
        self.set_zero_flag(self.regs.y == 0);
        self.set_negative_flag(get_bit_at(self.regs.y, NEGATIVE) == SET);
    }

    fn inc(&mut self, addr: u16) {
        let mut value = self.mem.read(addr);
        value = value.wrapping_add(1);
        self.set_zero_flag(value == 0);
        self.set_negative_flag(get_bit_at(value, NEGATIVE) == SET);
        self.mem.write(addr, value);
    }

    fn inx(&mut self) {
        self.regs.x = self.regs.x.wrapping_add(1);
        self.set_zero_flag(self.regs.x == 0);
        self.set_negative_flag(get_bit_at(self.regs.x, NEGATIVE) == SET);
    }

    fn iny(&mut self) {
        self.regs.y = self.regs.y.wrapping_add(1);
        self.set_zero_flag(self.regs.y == 0);
        self.set_negative_flag(get_bit_at(self.regs.y, NEGATIVE) == SET);
    }

    fn eor(&mut self, value: u8) {
        self.regs.a = self.regs.a | value;
        self.set_zero_flag(self.regs.a == 0);
        self.set_negative_flag(get_bit_at(self.regs.a, NEGATIVE) == SET)
    }

    fn jmp(&mut self, addr: u16) {
        self.regs.pc = addr;
    }

    pub fn next_instruction(&mut self) {
        let opcode = self.mem.read(self.regs.pc);
        self.regs.pc += 1;
        let value: u8;
        let addr: u16;

        match opcode {
            //ADC
            0x69 => {
                addr = self.get_immediate();
                value = self.mem.read(addr);
                self.adc(value);
            },
            0x65 => {
                addr = self.get_zero();
                value = self.mem.read(addr);
                self.adc(value);
            },
            0x75 => {
                addr = self.get_zero_x();
                value = self.mem.read(addr);
                self.adc(value);
            },
            0x6d => {
                addr = self.get_absolute();
                value = self.mem.read(addr);
                self.adc(value);
            },
            0x7d => {
                addr = self.get_absolute_x();
                value = self.mem.read(addr);
                self.adc(value);
            },
            0x79 => {
                addr = self.get_absolute_y();
                value = self.mem.read(addr);
                self.adc(value);
            },
            0x61 => {
                addr = self.get_indirect_x();
                value = self.mem.read(addr);
                self.adc(value);
            },
            0x71 => {
                addr = self.get_indirect_y();
                value = self.mem.read(addr);
                self.adc(value);
            },
            //AND
            0x29 => {
                addr = self.get_immediate();
                value = self.mem.read(addr);
                self.and(value);
            },
            0x25 => {
                addr = self.get_zero();
                value = self.mem.read(addr);
                self.and(value);
            },
            0x35 => {
                addr = self.get_zero_x();
                value = self.mem.read(addr);
                self.and(value);
            },
            0x2d => {
                addr = self.get_absolute();
                value = self.mem.read(addr);
                self.and(value);
            },
            0x3d => {
                addr = self.get_absolute_x();
                value = self.mem.read(addr);
                self.and(value);
            },
            0x39 => {
                addr = self.get_absolute_y();
                value = self.mem.read(addr);
                self.and(value);
            },
            0x21 => {
                addr = self.get_indirect_x();
                value = self.mem.read(addr);
                self.and(value);
            },
            0x31 => {
                addr = self.get_indirect_y();
                value = self.mem.read(addr);
                self.and(value);
            },
            //ASL
            0x0a => self.asl_acc(),
            0x06 => {
                addr = self.get_zero();
                self.asl_mem(addr);
            },
            0x16 => {
                addr = self.get_zero_x();
                self.asl_mem(addr);
            },
            0x0e => {
                addr = self.get_absolute();
                self.asl_mem(addr);
            },
            0x1e => {
                addr = self.get_absolute_x();
                self.asl_mem(addr);
            },
            //BCC
            0x90 => self.branch_if(CARRY, CLEAR),
            //BCS
            0xb0 => self.branch_if(CARRY, SET),
            //BEQ
            0xf0 => self.branch_if(ZERO, SET),
            //BIT
            0x24 => (),/////////////////////////////////////////////////////////////////////////////////
            0x2c => (),/////////////////////////////////////////////////////////////////////////////////
            //BMI
            0x30 => self.branch_if(NEGATIVE, SET),
            //BNE
            0xd0 => self.branch_if(ZERO, CLEAR),
            //BPL
            0x10 => self.branch_if(NEGATIVE, CLEAR),
            //BRK
            0x00 => (),/////////////////////////////////////////////////////////////////////////////////
            //BVC
            0x50 => self.branch_if(OVERFLOW, CLEAR),
            //BVS
            0x70 => self.branch_if(OVERFLOW, SET),
            //CLC
            0x18 => self.set_carry_flag(false),
            //CLD
            0xd8 => self.set_decimal_flag(false),
            //CLI
            0x58 => self.set_interrupt_flag(false),
            //CLV
            0xb8 => self.set_overflow_flag(false),
            //CMP
            0xc9 => {
                addr = self.get_immediate();
                value = self.mem.read(addr);
                self.cmp(value);
            },
            0xc5 => {
                addr = self.get_zero();
                value = self.mem.read(addr);
                self.cmp(value);
            },
            0xd5 => {
                addr = self.get_zero_x();
                value = self.mem.read(addr);
                self.cmp(value);
            },
            0xcd => {
                addr = self.get_absolute();
                value = self.mem.read(addr);
                self.cmp(value);
            },
            0xdd => {
                addr = self.get_absolute_x();
                value = self.mem.read(addr);
                self.cmp(value);
            },
            0xd9 => {
                addr = self.get_absolute_y();
                value = self.mem.read(addr);
                self.cmp(value);
            },
            0xc1 => {
                addr = self.get_indirect_x();
                value = self.mem.read(addr);
                self.cmp(value);
            },
            0xd1 => {
                addr = self.get_indirect_y();
                value = self.mem.read(addr);
                self.cmp(value);
            },
            //CPX
            0xe0 => {
                addr = self.get_immediate();
                value = self.mem.read(addr);
                self.cpx(value);
            },
            0xe4 => {
                addr = self.get_zero();
                value = self.mem.read(addr);
                self.cpx(value);
            },
            0xec => {
                addr = self.get_absolute();
                value = self.mem.read(addr);
                self.cpx(value);
            },
            //CPY
            0xc0 => {
                addr = self.get_immediate();
                value = self.mem.read(addr);
                self.cpy(value);
            },
            0xc4 => {
                addr = self.get_zero();
                value = self.mem.read(addr);
                self.cpy(value);
            },
            0xcc => {
                addr = self.get_absolute();
                value = self.mem.read(addr);
                self.cpy(value);
            },
            //DEC
            0xc6 => {
                addr = self.get_zero();
                self.dec(addr);
            },
            0xd6 => {
                addr = self.get_zero_x();
                self.dec(addr);
            },
            0xce => {
                addr = self.get_absolute();
                self.dec(addr);
            },
            0xde => {
                addr = self.get_absolute_x();
                self.dec(addr);
            },
            //DEX
            0xca => self.dex(),
            //DEY
            0x88 => self.dey(),
            //EOR
            0x49 => {
                addr = self.get_immediate();
                value = self.mem.read(addr);
                self.eor(value);
            },
            0x45 => {
                addr = self.get_zero();
                value = self.mem.read(addr);
                self.eor(value);
            },
            0x55 => {
                addr = self.get_zero_x();
                value = self.mem.read(addr);
                self.eor(value);
            },
            0x4d => {
                addr = self.get_absolute();
                value = self.mem.read(addr);
                self.eor(value);
            },
            0x5d => {
                addr = self.get_absolute_x();
                value = self.mem.read(addr);
                self.eor(value);
            },
            0x59 => {
                addr = self.get_absolute_y();
                value = self.mem.read(addr);
                self.eor(value);
            },
            0x41 => {
                addr = self.get_indirect_x();
                value = self.mem.read(addr);
                self.eor(value);
            },
            0x51 => {
                addr = self.get_indirect_y();
                value = self.mem.read(addr);
                self.eor(value);
            },
            //INC
            0xe6 => {
                addr = self.get_zero();
                self.inc(addr);
            },
            0xf6 => {
                addr = self.get_zero_x();
                self.inc(addr);
            },
            0xee => {
                addr = self.get_absolute();
                self.inc(addr);
            }, 
            0xfe => {
                addr = self.get_absolute_x();
                self.inc(addr);
            },
            //INX
            0xe8 => self.inx(),
            //INY
            0xc8 => self.iny(),
            //JMP
            0x4c => {
                addr = self.get_absolute();
                self.jmp(addr);
            },
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
