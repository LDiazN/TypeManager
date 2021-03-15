mod type_system;
mod driver;
mod utils;

fn main() {
    let mut program = driver::Program::new();

    println!("¡Bienvenido al simulador de tipos de Luis!");
    println!("powered by Rust ⚙️😎");

    while program.should_run() {
        program.run()
    }
}
