pub mod cpu;
pub mod memory;

fn main() {
    let mut a: u8 = 3;
    a = a << 1;
    println!("{}", a);
}
