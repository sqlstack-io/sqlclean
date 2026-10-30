//! Rebuild the executable when the bundled skill changes.

fn main() {
    println!("cargo:rerun-if-changed=skill");
}
