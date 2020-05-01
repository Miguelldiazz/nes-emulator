pub mod cpu;
pub mod memory;
pub mod utils;

use crate::utils::*;

fn main() {
    let a: u8 = 4;
    println!("{}", get_bit_at(a, 1));
}
