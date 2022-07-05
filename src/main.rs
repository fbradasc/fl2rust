/*!
# fl2rust
A fluid (fltk ui designer) file to Rust transpiler.
### As an executable
You can run fl2rust on the command-line by installing using cargo-install:
```ignore
$ cargo install fl2rust
```
Then run:
```ignore
$ fl2rust <fl-file>.fl > <output-file>.rs
```
### As a library
To automate things through cargo, you can use fl2rust as a library by adding it to your build-dependencies:
```toml
# Cargo.toml
[dependencies]
fltk = "1"
[build-dependencies]
fl2rust = "0.4"
```
```rust,no_run
// build.rs
fn main() {
    use std::path::PathBuf;
    use std::env;
    println!("cargo:rerun-if-changed=src/myuifile.fl");
    let g = fl2rust::Generator::default();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    g.in_out("src/myuifile.fl", out_path.join("myuifile.rs").to_str().unwrap()).expect("Failed to generate rust from fl file!");
}
```
```ignore
# src/myuifile.fl -> generated via fluid
# data file for the Fltk User Interface Designer (fluid)
version 1.0400
header_name {.h}
code_name {.cxx}
class UserInterface {open
} {
  Function {make_window()} {open
  } {
    Fl_Window {} {open selected
      xywh {138 161 440 355} type Double visible
    } {
      Fl_Button but {
        label {Click me}
        xywh {175 230 95 45}
      }
    }
  }
}
```
```rust,ignore
// src/myuifile.rs
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(clippy::needless_update)]
include!(concat!(env!("OUT_DIR"), "/myuifile.rs"));
```
```rust,ignore
// src/main.rs
use fltk::{prelude::*, *};
mod myuifile;
fn main() {
    let app = app::App::default();
    let mut ui = myuifile::UserInterface::make_window();
    ui.but.set_callback(move |_| {
        println!("Works!");
    });
    app.run().unwrap();
}
```
*/

#![allow(clippy::needless_doctest_main)]

use std::error;
use std::fs;

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let f = fs::read_to_string(&args[1])?;
    let ast = fl2rust::parser::parse(&f);
    println!("{}", fl2rust::gen::generate_with_directives_preamble(&ast));
    if args.contains(&"--print-ast".to_string()) {
        for elem in ast {
            println!("{:?}", elem);
        }
    }
    Ok(())
}
