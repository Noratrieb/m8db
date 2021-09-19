mod parse;
mod run;

fn main() {
    println!(
        "m8db - M8 Debugger
(C) Nilstrieb (https://github.com/Nilstrieb/m8db)
Type 'help' for help
    "
    );

    run::start(std::env::args().nth(1));
}
