fn main() {
  tauri_build::build();
  println!("cargo:rustc-link-lib=user32");
  println!("cargo:rustc-link-lib=shell32");
}
