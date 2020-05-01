pub mod cpu;
pub mod memory;
pub mod utils;

use crate::utils::*;

fn main() {
    let a: u8 = 1;
    println!("{}", a.wrapping_div(2));
}
