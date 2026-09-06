use sila_printer::list_printers;

fn main() {
    let printers = list_printers().unwrap();
    println!("{:?}", printers);
}
