
pub struct Memory {
    data: [u8; 8192],   //8192 kb
}

impl Memory {
    pub fn new()-> Memory {
        Memory {
            data: [0; 8192],
        }
    }
}