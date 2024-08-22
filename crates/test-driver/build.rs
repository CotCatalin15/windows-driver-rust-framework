fn main() -> Result<(), wdk_build::ConfigError> {
    wdk_build::Config::default().configure_binary_build()
}
