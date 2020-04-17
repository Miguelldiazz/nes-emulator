use crate::mem::*;
use crate::registers::*;

pub struct Cpu {
    mem: Memory,
    regs: Registers,
}

impl Cpu {
    pub fn new()-> Cpu {
        Cpu {
            mem: Memory::new(),
            regs: Registers::new(),
        }
    }
}

