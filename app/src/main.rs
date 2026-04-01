fn main() {
    let mut args = std::env::args().skip(1);
    if matches!(args.next().as_deref(), Some("--version")) {
        println!("oh-my-oc {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    println!("oh-my-oc");
}
