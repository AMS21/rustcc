#![allow(clippy::panic)]
#![allow(clippy::expect_used)]

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use bindgen::{Builder, EnumVariation, FieldVisibilityKind};
use semver::Version; // For validating minimum supported LLVM version

// Candidate llvm-config executable basenames we will probe on PATH (in order).
// Non-Windows platforms sometimes install multiple versioned llvm-config*
// binaries. Keep newest supported versions first so we prefer them when
// multiple are installed. The unversioned "llvm-config" is checked last as a
// fallback. On Windows distributions typically only provide an unversioned
// llvm-config.exe, so we skip probing version-suffixed names there.
// NOTE: Usability additionally requires the discovered llvm-config reports a
// version >= MIN_VERSION       enforced in `is_usable_llvm_config`.
#[cfg(not(windows))]
const LLVM_CONFIG_CANDIDATES: &[&str] = &[
    "llvm-config-22",
    "llvm-config-21",
    "llvm-config-20",
    "llvm-config-19",
    "llvm-config-18",
    "llvm-config-17",
    "llvm-config-16",
    "llvm-config-15",
    "llvm-config-14",
    "llvm-config-13",
    "llvm-config-12",
    "llvm-config-11",
    "llvm-config-10",
    "llvm-config-9",
    "llvm-config-8",
    "llvm-config-7",
    "llvm-config-6.0",
    "llvm-config-5.0",
    "llvm-config", // fallback
];
#[cfg(windows)]
const LLVM_CONFIG_CANDIDATES: &[&str] = &["llvm-config.exe"]; // Only unversioned on Windows

// Minimum supported LLVM major version
const LLVM_MINIMUM_VERSION: Version = Version::new(5, 0, 0);

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTCC_LLVM_CONFIG_PATH");

    // Decide whether we can reuse a system LLVM instead of building the submodule.
    let llvm_config_path = find_llvm_config();

    if let Some(p) = llvm_config_path.as_ref() {
        link_with_system_llvm(p);
        return;
    }

    // If we get here without returning, we couldn't find a usable system
    // llvm-config. Fail with a helpful message.

    panic!(
        "No usable llvm-config found. Please install a system LLVM (>= {LLVM_MINIMUM_VERSION}) or \
         set RUSTCC_LLVM_CONFIG_PATH."
    );
}

/// Try to locate a usable `llvm-config` either from `RUSTCC_LLVM_CONFIG_PATH`
/// or `PATH`. Returns `Some(path)` if successful.
fn find_llvm_config() -> Option<PathBuf> {
    // Try the explicit path first
    if let Ok(p) = env::var("RUSTCC_LLVM_CONFIG_PATH") {
        let path = PathBuf::from(&p);
        if is_usable_llvm_config(&path) {
            return Some(path);
        }
        println!(
            "cargo:warning=RUSTCC_LLVM_CONFIG_PATH set ('{p}') but unusable (missing or --version \
             failed)"
        );
    }

    // Probe PATH for candidates
    for candidate in LLVM_CONFIG_CANDIDATES {
        let path = PathBuf::from(candidate);

        if is_usable_llvm_config(&path) {
            return Some(path);
        }
    }

    None
}

fn is_usable_llvm_config(path: &Path) -> bool {
    let Ok(output) = Command::new(path).arg("--version").output() else {
        return false;
    };
    if !output.status.success() {
        return false;
    }

    let version_string = String::from_utf8_lossy(&output.stdout).trim().to_owned();

    // llvm-config sometimes prints versions with suffixes like "17.0.0git".
    let cleaned_version: String = version_string
        .chars()
        .scan(true, |keep, ch| {
            if *keep && (ch.is_ascii_alphanumeric() || ch == '.' || ch == '+') {
                Some(ch)
            } else {
                *keep = false;
                None
            }
        })
        .collect();

    let Ok(parsed) = Version::parse(&cleaned_version) else {
        return false;
    };

    if parsed < LLVM_MINIMUM_VERSION {
        println!(
            "cargo:warning=Found llvm-config at '{}' but version {} is below minimum {}",
            path.display(),
            parsed,
            LLVM_MINIMUM_VERSION
        );
        return false;
    }

    true
}

fn link_with_system_llvm(llvm_config: &Path) {
    emit_from_llvm_config(llvm_config, false);
}

fn emit_from_llvm_config(llvm_config: &Path, static_link: bool) {
    let libdir = run(llvm_config, &["--libdir"]);
    println!("cargo:rustc-link-search=native={libdir}");

    let libs = if static_link {
        run(llvm_config, &["--link-static", "--libs", "engine"])
    } else {
        run(llvm_config, &["--libs", "engine"])
    };
    emit_link_libs(&libs, static_link);

    let system_libs = run(llvm_config, &["--system-libs"]);
    emit_system_libs(&system_libs);

    let include_dir = run(llvm_config, &["--includedir"]);
    generate_bindings(&include_dir);
}

fn emit_link_libs(libs: &str, static_link: bool) {
    for token in libs.split_whitespace() {
        if let Some(stripped) = token.strip_prefix("-l") {
            if static_link {
                println!("cargo:rustc-link-lib=static={stripped}");
            } else {
                println!("cargo:rustc-link-lib={stripped}");
            }
        } else {
            let path = Path::new(token);
            // Handle absolute or relative library file paths returned by llvm-config
            // - On Unix static builds this may be ".a"
            // - On Windows MSVC builds this is typically ".lib" (static or import libs)
            if let Some(ext) = path.extension()
                && (ext.eq_ignore_ascii_case("a") || ext.eq_ignore_ascii_case("lib"))
            {
                let mut lib_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or(token);
                // Strip common "lib" prefix if present (mainly for Unix-style names)
                if let Some(stripped) = lib_name.strip_prefix("lib") {
                    lib_name = stripped;
                }

                if static_link {
                    println!("cargo:rustc-link-lib=static={lib_name}");
                } else {
                    println!("cargo:rustc-link-lib={lib_name}");
                }

                if let Some(parent) = path.parent()
                    && !parent.as_os_str().is_empty()
                {
                    println!("cargo:rustc-link-search=native={}", parent.display());
                }
            }
        }
    }
}

#[cfg(windows)]
const IGNORED_SYSTEM_LIBS: &[&str] = &["libxml2s.lib", "xml2s.lib"];

fn emit_system_libs(system_libs: &str) {
    for token in system_libs.split_whitespace() {
        // Skip libxml2s which is not needed and not inside the prebuild archive.
        #[cfg(windows)]
        if IGNORED_SYSTEM_LIBS.contains(&token.to_ascii_lowercase().as_str()) {
            continue;
        }

        if let Some(stripped) = token.strip_prefix("-l") {
            // Typical Unix-style system libs (e.g., -ldl -lpthread)
            println!("cargo:rustc-link-lib={stripped}");
            continue;
        }

        let path = Path::new(token);
        // Accept Windows-style tokens like "Advapi32.lib" or absolute paths to .lib
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lib") || ext.eq_ignore_ascii_case("a"))
        {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                // Normalize common Unix-style prefix before checks
                let name = stem.strip_prefix("lib").unwrap_or(stem);

                println!("cargo:rustc-link-lib={name}");
            }
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                println!("cargo:rustc-link-search=native={}", parent.display());
            }
            continue;
        }

        // Bare names without -l or extension (e.g., "shell32" on Windows)
        if !token.is_empty() {
            // Windows-specific: ignore libxml2s which is not needed and not inside the
            // prebuild archive.
            #[cfg(windows)]
            if IGNORED_SYSTEM_LIBS.contains(&token.to_ascii_lowercase().as_str()) {
                continue;
            }

            println!("cargo:rustc-link-lib={token}");
        }
    }
}

fn generate_bindings(include_path: &str) {
    let llvm_c_dir = Path::new(include_path).join("llvm-c");
    assert!(
        llvm_c_dir.exists(),
        "No llvm-c directory found in include path '{include_path}'"
    );

    let mut headers = collect_header_files(&llvm_c_dir);
    headers.sort(); // deterministic order (stable order for bindgen)
    let headers = headers;

    assert!(
        !headers.is_empty(),
        "No llvm-c headers found in '{}'",
        llvm_c_dir.display()
    );

    let bindings = Builder::default()
        .headers(headers.iter().map(|p| p.to_string_lossy()))
        .allowlist_item("LLVM.*")
        .clang_args(["-I", include_path])
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .default_visibility(FieldVisibilityKind::PublicCrate)
        .derive_eq(true)
        .derive_hash(true)
        .derive_ord(true)
        .derive_partialeq(true)
        .derive_partialord(true)
        .enable_function_attribute_detection()
        .generate_comments(false) // Disable comments since they contain invalid doc-tests
        .generate_cstr(true)
        .impl_debug(true)
        .impl_partialeq(true)
        .merge_extern_blocks(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .wrap_unsafe_ops(true)
        .generate()
        .expect("Unable to generate LLVM bindings");

    let out_path =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not defined")).join("bindings.rs");

    bindings
        .write_to_file(&out_path)
        .expect("Unable to write LLVM bindings");
}

/// Iteratively traverse `root` collecting all `*.h` (case-insensitive) files.
/// Unreadable directories are skipped silently.
fn collect_header_files(root: &Path) -> Vec<PathBuf> {
    let mut headers = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read_dir) = dir.read_dir() else {
            println!(
                "cargo:warning=Skipping unreadable directory '{}'",
                dir.display()
            );
            continue;
        };

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file()
                && path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("h"))
            {
                headers.push(path);
            }
        }
    }

    headers
}

fn run(executable: &Path, args: &[&str]) -> String {
    let out = Command::new(executable)
        .args(args)
        .output()
        .expect("Failed to run command (executable missing?)");

    if out.status.success() {
        String::from_utf8(out.stdout)
            .expect("Command output not UTF-8")
            .trim()
            .to_owned()
    } else {
        panic!(
            "Command '{} {}' failed with status {}\n--- stdout:\n{}\n--- stderr:\n{}",
            executable.display(),
            args.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
}
