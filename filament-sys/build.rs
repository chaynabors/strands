use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const FILAMENT_VERSION: &str = std::env!("CARGO_PKG_VERSION");

fn main() {
    let spec_file = format!("spec/filament-{}.md", FILAMENT_VERSION);

    let header_mappings = [
        ("## Appendix B: Core Header", "include/filament.h"),
        (
            "## Appendix C: Standard Library Header",
            "include/filament_std.h",
        ),
    ];

    // Sync C Headers from Specification
    if let Err(e) = sync_headers_from_spec(&spec_file, &header_mappings) {
        panic!("Failed to sync headers from spec: {}", e);
    }

    // Generate Rust Bindings
    generate_bindings();
}

/// Parses the Markdown specification and writes the C code blocks to files.
fn sync_headers_from_spec(spec_path: &str, mappings: &[(&str, &str)]) -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", spec_path);

    let content = fs::read_to_string(spec_path)?;
    let lines: Vec<&str> = content.lines().collect();

    fs::create_dir_all("include")?;

    for (section_title, output_filename) in mappings {
        let mut found_section = false;
        let mut inside_code_block = false;
        let mut captured_code = String::new();

        captured_code.push_str(
            "// --------------------------------------------------------------------------\n",
        );
        captured_code.push_str("// AUTO-GENERATED FILE. DO NOT EDIT.\n");
        captured_code.push_str(&format!("// Source: {}\n", spec_path));
        captured_code.push_str(
            "// --------------------------------------------------------------------------\n\n",
        );

        for line in &lines {
            if !found_section {
                if line.trim_end() == *section_title {
                    found_section = true;
                }
                continue;
            }

            if !inside_code_block {
                if line.trim().starts_with("```") {
                    inside_code_block = true;
                }
                continue;
            }

            if inside_code_block {
                if line.trim().starts_with("```") {
                    break;
                }
                captured_code.push_str(line);
                captured_code.push('\n');
            }
        }

        if found_section {
            let mut file = fs::File::create(output_filename)?;
            file.write_all(captured_code.as_bytes())?;
            println!(
                "cargo:warning=Updated {} from specification.",
                output_filename
            );
        } else {
            println!(
                "cargo:warning=Section '{}' not found in specification.",
                section_title
            );
        }
    }

    Ok(())
}

/// Generates Rust FFI bindings using a composite wrapper.
fn generate_bindings() {
    let wrapper_path = "include/filament_wrapper.h";

    // Create a temporary wrapper header that includes everything
    let mut wrapper_file = fs::File::create(wrapper_path).expect("Could not create wrapper.h");
    writeln!(wrapper_file, "#include \"filament.h\"").unwrap();
    writeln!(wrapper_file, "#include \"filament_std.h\"").unwrap();

    println!("cargo:rerun-if-changed={}", wrapper_path);

    let bindings = bindgen::Builder::default()
        .header(wrapper_path)
        .clang_arg("-Iinclude") // Ensure clang can find the headers
        .use_core()
        .ctypes_prefix("core::ffi")
        .derive_default(true)
        .derive_eq(false)
        .derive_partialeq(false)
        .derive_hash(false)
        .derive_debug(true)
        .derive_copy(true)
        .bitfield_enum("FilamentEventFlags")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .allowlist_function("filament_.*")
        .allowlist_type("Filament.*")
        .allowlist_type("FILAMENT_.*")
        .allowlist_var("FILAMENT_.*")
        .layout_tests(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    fs::remove_file(wrapper_path).ok();
}
