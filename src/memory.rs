pub struct Memory {
    data: [u8; 8192], //ram (0000-3fff), i/o (4000-7fff), rom(8000-ffff)
}

impl Memory {
    pub fn new() -> Memory {
        Memory { data: [0; 8192] }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }
}