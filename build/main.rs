mod hooks;

fn main() {
    println!("cargo::rerun-if-changed=build");
    println!("cargo::rerun-if-changed=.githooks");

    hooks::install();
}
