fn main() {
    println!("cargo:rerun-if-changed=assets/icon/icon.ico");

    // Embed the icon and version info into the Windows executable
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon/icon.ico")
            .set("ProductName", "Git Commit Rewriter")
            .set("FileDescription", "Git Commit Rewriter");
        if let Err(e) = res.compile() {
            panic!("failed to embed Windows resources: {}", e);
        }
    }
}
