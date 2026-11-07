fn main() {
    // "@kit" → slint-kit's ui/ directory; std widgets in the dark Fluent style.
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-dark".into())
        .with_library_paths(slint_kit_ui::library_paths());
    slint_build::compile_with_config("ui/main.slint", config)
        .expect("ui/main.slint: Slint compilation failed");
}
