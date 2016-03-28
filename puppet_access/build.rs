fn main() {
    println!("cargo:include=/usr/local/include");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
}
