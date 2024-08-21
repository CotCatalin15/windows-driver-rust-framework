fn main() -> Result<(), wdk_build::ConfigError> {
    wdk_build::Config::default().configure_library_build()
}
