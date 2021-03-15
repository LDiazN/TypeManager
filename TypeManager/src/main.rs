mod type_system;
mod driver;
mod utils;
mod test_suite;

fn main() {
    let mut program = driver::Program::new();

    println!("¡Bienvenido al simulador de tipos de Luis!\n");
    println!("  -powered by Rust ⚙️ 😎\n\n");

    while program.should_run() {
        program.run()
    }
}
