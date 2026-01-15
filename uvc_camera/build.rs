use std::process::Command;

fn check_pkg_config(lib: &str) -> bool {
    Command::new("pkg-config")
        .args(["--exists", lib])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn main() {
    let required_libs = [
        ("libavcodec", "ffmpeg libraries"),
        ("libavformat", "ffmpeg libraries"),
        ("libavutil", "ffmpeg libraries"),
        ("libswscale", "ffmpeg libraries"),
        ("dav1d", "libdav1d (AV1 software decoder)"),
    ];

    let mut missing = Vec::new();

    for (lib, description) in &required_libs {
        if !check_pkg_config(lib) {
            missing.push((*lib, *description));
        }
    }

    if !missing.is_empty() {
        eprintln!();
        eprintln!("╔══════════════════════════════════════════════════════════════════╗");
        eprintln!("║              MISSING REQUIRED SYSTEM LIBRARIES                   ║");
        eprintln!("╠══════════════════════════════════════════════════════════════════╣");
        for (lib, desc) in &missing {
            eprintln!("║  - {lib:<15} ({desc})", lib = lib, desc = desc);
        }
        eprintln!("╠══════════════════════════════════════════════════════════════════╣");
        eprintln!("║ Install the missing libraries:                                   ║");
        eprintln!("║                                                                  ║");
        eprintln!("║ Ubuntu/Debian:                                                   ║");
        eprintln!("║   sudo apt install libavcodec-dev libavformat-dev \\              ║");
        eprintln!("║                    libavutil-dev libswscale-dev libdav1d-dev     ║");
        eprintln!("║                                                                  ║");
        eprintln!("║ Fedora:                                                          ║");
        eprintln!("║   sudo dnf install ffmpeg-devel libdav1d-devel                   ║");
        eprintln!("║                                                                  ║");
        eprintln!("║ Arch Linux:                                                      ║");
        eprintln!("║   sudo pacman -S ffmpeg dav1d                                    ║");
        eprintln!("║                                                                  ║");
        eprintln!("║ macOS (Homebrew):                                                ║");
        eprintln!("║   brew install ffmpeg dav1d                                      ║");
        eprintln!("╚══════════════════════════════════════════════════════════════════╝");
        eprintln!();
        panic!("Missing required system libraries - see above for installation instructions");
    }

    println!("cargo::rerun-if-env-changed=PKG_CONFIG_PATH");
}
