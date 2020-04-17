
pub struct Registers {
    a: u8,    //accumulator
    x: u8,    //index
    y: u8,    //index
    pc: u16,  //program counter
    sp: u8,   //stack pointer
    p: u8,    //status register
}

impl Registers {
    pub fn new()-> Registers {
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