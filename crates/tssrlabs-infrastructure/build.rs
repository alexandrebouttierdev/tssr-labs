//! Recompile la crate quand le contenu embarqué change.

fn main() {
    println!("cargo:rerun-if-changed=../../content");
}
