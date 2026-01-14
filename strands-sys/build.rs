use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let header_core = "include/strands_core.h";
    let header_std = "include/strands_std.h";
    let header_structs = "include/strands_std_structs.h";
    let spec_file = "spec/strands.md";

    generate_bindings(header_core);

    if let Err(e) = update_spec_documentation(spec_file, header_core, header_std, header_structs) {
        println!("cargo:warning=Failed to update spec documentation: {}", e);
    }
}

/// Generates Rust FFI bindings from the C headers using bindgen.
///
/// This configures strict allow-lists to ensure only `Strands`-related symbols
/// are exposed to the Rust crate, and treats `StrandsEventFlags` as a bitfield.
/// The output is written to `bindings.rs` in the cargo build directory.
fn generate_bindings(entry_header: &str) {
    println!("cargo:rerun-if-changed={}", entry_header);
    // Note: We don't explicitly list dependent headers here because
    // bindgen::CargoCallbacks handles dependency discovery automatically.

    let bindings = bindgen::Builder::default()
        .header(entry_header)
        .use_core()
        .ctypes_prefix("core::ffi")
        .derive_default(true)
        .derive_eq(false)
        .derive_partialeq(false)
        .derive_hash(false)
        .derive_debug(true)
        .derive_copy(true)
        .bitfield_enum("StrandsEventFlags")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .allowlist_function("strands_.*")
        .allowlist_type("Strands.*")
        .allowlist_type("STRANDS_.*")
        .allowlist_var("STRANDS_.*")
        .layout_tests(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

/// Updates the Markdown specification to ensure it matches the actual C headers.
///
/// This function treats the files in `include/` as the source of truth. It reads
/// `spec/strands.md`, locates the Appendix sections, and injects the raw content
/// of the header files into the corresponding code blocks.
fn update_spec_documentation(
    spec_path: &str,
    core_path: &str,
    std_path: &str,
    structs_path: &str,
) -> std::io::Result<()> {
    println!("cargo:rerun-if-changed={}", spec_path);
    println!("cargo:rerun-if-changed={}", core_path);
    println!("cargo:rerun-if-changed={}", std_path);
    println!("cargo:rerun-if-changed={}", structs_path);

    let content = fs::read_to_string(spec_path)?;
    let mut lines = content.lines();
    let mut new_output = String::with_capacity(content.len());

    let core_content = fs::read_to_string(core_path)?;
    let std_content = fs::read_to_string(std_path)?;
    let structs_content = fs::read_to_string(structs_path)?;

    let replacements = [
        ("## Appendix A: Core Header", core_content),
        ("## Appendix B: Standard Library Header", std_content),
        ("## Appendix C: Canonical Binary Layouts", structs_content),
    ];

    while let Some(line) = lines.next() {
        new_output.push_str(line);
        new_output.push('\n');

        for (header, file_content) in &replacements {
            if line.trim_end() == *header {
                // Advance to the code block start
                for sub_line in lines.by_ref() {
                    new_output.push_str(sub_line);
                    new_output.push('\n');
                    if sub_line.trim().starts_with("```") {
                        break;
                    }
                }

                new_output.push_str(file_content);
                if !file_content.ends_with('\n') {
                    new_output.push('\n');
                }

                // Skip existing markdown content until the code block ends
                for skip_line in lines.by_ref() {
                    if skip_line.trim().starts_with("```") {
                        new_output.push_str(skip_line);
                        new_output.push('\n');
                        break;
                    }
                }
                break;
            }
        }
    }

    fs::write(spec_path, new_output)?;

    Ok(())
}
