fn main() {
    // only run if target os is windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        return;
    }

    // only build the resource for release builds
    // as calling rc.exe might be slow
    #[cfg(target_os = "windows")]
    if std::env::var("PROFILE").unwrap() == "release" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("./branding/squiidsquare.ico");
        res.compile().unwrap();
    }
}
