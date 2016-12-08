fn main() {
    println!("cargo:rustc-link-search=./src/parser/");
    println!("cargo:rustc-link-lib=static=parse");
    println!("cargo:rustc-link-lib=l");
}
