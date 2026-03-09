use std::{fs, path::Path};

/// Valid platforms
const VALID_PLATFORMS: [&str; 3] = ["plat_qemu_riscv", "plat_qemu_x86_64", "plat_vf2"];

fn main() {
    println!("cargo::rustc-check-cfg=cfg(plat_qemu_riscv)");
    println!("cargo::rustc-check-cfg=cfg(plat_qemu_x86_64)");
    println!("cargo::rustc-check-cfg=cfg(plat_vf2)");

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").ok().unwrap_or_default();

    let cpus = option_env!("SMP");

    if let Some(cpus) = cpus {
        let cpus = cpus.parse::<usize>().unwrap();
        let config_file = Path::new("src/lib.rs");
        let config = fs::read_to_string(config_file).unwrap();
        let cpus = format!("pub const CPU_NUM: usize = {};\n", cpus);
        let mut new_config = String::new();
        for line in config.lines() {
            if line.starts_with("pub const CPU_NUM: usize = ") {
                new_config.push_str(cpus.as_str());
            } else {
                new_config.push_str(line);
                new_config.push('\n');
            }
        }
        fs::write(config_file, new_config).unwrap();
    }

    println!("cargo:rerun-if-changed=src/lib.rs");
    let platform = std::env::var("PLATFORM").ok().unwrap_or_else(|| {
        match target_arch.as_str() {
            "x86_64" => "plat_qemu_x86_64".to_string(),
            "riscv64" => "plat_qemu_riscv".to_string(),
            _ => "plat_qemu_riscv".to_string(),
        }
    });
    
    // Validate platform
    if !VALID_PLATFORMS.contains(&platform.as_str()) {
        panic!("Invalid PLATFORM='{}'. Valid values are: {:?}", platform, VALID_PLATFORMS);
    }

    match target_arch.as_str() {
        "x86_64" | "riscv64" => {}
        other => panic!("Unsupported target architecture '{}'. Expected x86_64 or riscv64", other),
    }

    let is_valid_combo = match target_arch.as_str() {
        "x86_64" => platform == "plat_qemu_x86_64",
        "riscv64" => matches!(platform.as_str(), "plat_qemu_riscv" | "plat_vf2"),
        _ => false,
    };
    if !is_valid_combo {
        panic!(
            "Invalid ARCH/PLATFORM combination: target_arch='{}', PLATFORM='{}'",
            target_arch, platform
        );
    }
    
    println!("cargo::rustc-cfg={}", platform.as_str());
}
