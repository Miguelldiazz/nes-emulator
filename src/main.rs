pub mod cpu;

fn main() {
    let a: u8 = 255;
    println!("{}", a.wrapping_add(255));
}
