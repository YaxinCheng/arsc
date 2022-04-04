# arsc
arsc is a Rust library that provides the ability to parse and write Android resource file (arsc)

```toml
[dependencies]
arsc = "0.1"
```

Compiler support: *rustc 1.59+*

## Example

```rust
use arsc::{parse, write};

fn main() -> std::io::Result<()> {
  let arsc = parse("/resources.arsc")?;
  let _ = write(&arsc, "/output.arsc")?;
  Ok(())
}
```

## Getting Started

This section talks about how to compile the project

### Prerequisites:

* Rust 1.59 or above
* Cargo
* Git

### Compile

```bash
cd SOME_DIR
git clone https://github.com/YaxinCheng/arsc.git
cd arsc
cargo build --release
```

### 

