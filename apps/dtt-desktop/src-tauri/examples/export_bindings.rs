#![forbid(unsafe_code)]
// Binding generation needs DTOs only, not a desktop windowing runtime.
#[path = "../src/dto/mod.rs"]
mod dto;
#[allow(dead_code)]
#[path = "../src/error.rs"]
mod error;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    dto::export_bindings()
}
