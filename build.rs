use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/icons.gresource");
    let status = Command::new("glib-compile-resources")
        .arg("src/icons.gresource")
        .arg("--target=src/icons.bin")
        .spawn()
        .expect("Failed running glib-compile-resources")
        .wait()
        .unwrap();
    assert!(status.success());
}
