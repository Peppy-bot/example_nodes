use std::process::Command;

fn main() {
    // Check for nasm which is required by ffmpeg-sys-next for x86 assembly optimizations
    let nasm_check = Command::new("nasm").arg("-v").output();

    match nasm_check {
        Ok(output) if output.status.success() => {
            // nasm is available
        }
        _ => {
            eprintln!();
            eprintln!("╔══════════════════════════════════════════════════════════════════╗");
            eprintln!("║                     MISSING DEPENDENCY: nasm                     ║");
            eprintln!("╠══════════════════════════════════════════════════════════════════╣");
            eprintln!("║ The 'nasm' assembler is required to build FFmpeg.                ║");
            eprintln!("║                                                                  ║");
            eprintln!("║ Install it with:                                                 ║");
            eprintln!("║   Ubuntu/Debian:  sudo apt install nasm                          ║");
            eprintln!("║   Fedora:         sudo dnf install nasm                          ║");
            eprintln!("║   Arch:           sudo pacman -S nasm                            ║");
            eprintln!("║   macOS:          brew install nasm                              ║");
            eprintln!("╚══════════════════════════════════════════════════════════════════╝");
            eprintln!();
            panic!("nasm not found - please install it and retry the build");
        }
    }
}
