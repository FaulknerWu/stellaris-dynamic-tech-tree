#![forbid(unsafe_code)]

fn main() {
    if std::env::args().skip(1).any(|argument| argument == "--x11") {
        #[cfg(target_os = "linux")]
        std::env::set_var("GDK_BACKEND", "x11");
    }
    dtt_desktop::run();
}
