use wdk_build::DriverConfig;

fn main() -> Result<(), wdk_build::ConfigError> {
    let mut config = wdk_build::Config::from_env_auto()?;

    config.driver_config = DriverConfig::WDM();

    config.configure_binary_build();
    Ok(())
}
