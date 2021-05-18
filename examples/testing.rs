use rusty_daw_io::DeviceConfigurator;

fn main() {
    let mut config = DeviceConfigurator::new();
    dbg!(config.servers());
}