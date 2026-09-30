#![forbid(unsafe_code)]

fn main() -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    if std::env::args_os()
        .skip(1)
        .any(|argument| argument == "--x11")
        && std::env::var_os("GDK_BACKEND").as_deref() != Some(std::ffi::OsStr::new("x11"))
    {
        use std::os::unix::process::CommandExt;

        // Set the backend before GTK starts without mutating this process's environment.
        // The inherited x11 value prevents the replacement process from relaunching again.
        let error = std::process::Command::new(std::env::current_exe()?)
            .args(std::env::args_os().skip(1))
            .env("GDK_BACKEND", "x11")
            .exec();
        return Err(error);
    }
    dtt_desktop::run();
    Ok(())
}
