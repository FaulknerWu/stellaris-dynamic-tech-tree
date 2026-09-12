fn main() -> Result<(), Box<dyn std::error::Error>> {
    dtt_desktop::export_bindings()?;
    Ok(())
}
