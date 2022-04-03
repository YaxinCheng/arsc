use arsc::parse;
use arsc::write;

fn main() {
    let arsc = parse("/Users/cheng/Desktop/resources.arsc").expect("failed to read");
    write(arsc, "/Users/cheng/Desktop/output.arsc").expect("Failed to write");
}
