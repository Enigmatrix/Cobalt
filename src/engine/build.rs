//! Build script for the engine

/// Bundle `VCRUNTIME140.DLL` on Windows using Hybrid Linking
fn main() -> std::io::Result<()> {
    static_vcruntime::metabuild();
    Ok(())
}
