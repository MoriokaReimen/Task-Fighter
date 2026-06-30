fn main() {
    /* Set icon for windows build */
    if cfg!(target_os = "windows") {
        winres::WindowsResource::new()
            .set_icon("./assets/icon.ico")
            .compile()
            .unwrap();
    }
}
