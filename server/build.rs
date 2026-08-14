fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        let icon_path = std::path::Path::new("assets").join("icon.ico");
        if icon_path.exists() {
            use winres::WindowsResource;
            WindowsResource::new()
                .set_icon("assets/icon.ico")
                .compile()?;
        }
    }
    Ok(())
}
