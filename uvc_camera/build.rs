use std::process::Command;

const REQUIRED_LIBS: &[&str] = &[
    "libavutil",
    "libavformat",
    "libavcodec",
    "libswscale",
    "libavfilter",
    "libswresample",
    "libavdevice",
];

fn main() {
    let mut missing_libs = Vec::new();
    let mut missing_clang = false;

    // Check for FFmpeg libraries
    for lib in REQUIRED_LIBS {
        let status = Command::new("pkg-config")
            .args(["--exists", lib])
            .status();

        match status {
            Ok(s) if s.success() => {}
            _ => missing_libs.push(*lib),
        }
    }

    // Check for libclang (needed by bindgen)
    let clang_check = Command::new("llvm-config")
        .arg("--version")
        .output();

    if clang_check.is_err() || !clang_check.unwrap().status.success() {
        // Also check if libclang.so exists in common paths
        let common_paths = [
            "/usr/lib/llvm-14/lib/libclang.so",
            "/usr/lib/llvm-15/lib/libclang.so",
            "/usr/lib/llvm-16/lib/libclang.so",
            "/usr/lib/llvm-17/lib/libclang.so",
            "/usr/lib/llvm-18/lib/libclang.so",
            "/usr/lib/x86_64-linux-gnu/libclang-14.so",
            "/usr/lib/x86_64-linux-gnu/libclang-15.so",
            "/usr/lib/x86_64-linux-gnu/libclang-16.so",
            "/usr/lib/x86_64-linux-gnu/libclang-17.so",
            "/usr/lib/x86_64-linux-gnu/libclang-18.so",
            "/usr/lib/libclang.so",
        ];

        missing_clang = !common_paths.iter().any(|p| std::path::Path::new(p).exists());
    }

    if !missing_libs.is_empty() || missing_clang {
        eprintln!("\n");
        eprintln!("╔══════════════════════════════════════════════════════════════════╗");
        eprintln!("║                     MISSING BUILD DEPENDENCIES                   ║");
        eprintln!("╠══════════════════════════════════════════════════════════════════╣");

        if !missing_libs.is_empty() {
            eprintln!("║ Missing FFmpeg libraries:                                        ║");
            for lib in &missing_libs {
                eprintln!("║   - {:<60}║", lib);
            }
            eprintln!("║                                                                  ║");
        }

        if missing_clang {
            eprintln!("║ Missing libclang (required by bindgen):                          ║");
            eprintln!("║   - libclang-dev                                                 ║");
            eprintln!("║                                                                  ║");
        }

        eprintln!("║ Please install with:                                             ║");
        eprintln!("║   sudo apt install libavutil-dev libavformat-dev libavcodec-dev \\║");
        eprintln!("║       libswscale-dev libavfilter-dev libswresample-dev \\         ║");
        eprintln!("║       libavdevice-dev libclang-dev                               ║");
        eprintln!("╚══════════════════════════════════════════════════════════════════╝");
        eprintln!("\n");
        std::process::exit(1);
    }
}
