use arsc::parse;
use arsc::write;

fn main() {
    let arsc = parse("/Users/cheng/Desktop/resources.arsc").expect("");
    write(arsc)
}
