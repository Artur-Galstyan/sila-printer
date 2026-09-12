use sila_printer::get_printer;

fn main() {
    let printers = get_printer().unwrap();
    println!("{:?}", printers);
}
