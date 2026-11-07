fn main() {
    // "@kaamos" → Kaamos' ui/ directory; std widgets in the dark Fluent style.
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-dark".into())
        .with_library_paths(kaamos_ui::library_paths());
    slint_build::compile_with_config("ui/main.slint", config)
        .expect("ui/main.slint: Slint compilation failed");
}
