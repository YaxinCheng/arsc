use crate::components::Arsc;
use crate::writer::component_sizing::ByteSizing;

mod component_sizing;

pub fn write(arsc: Arsc) {
    let size = arsc.packages[0].key_names.size();
    println!("{size}")
}
