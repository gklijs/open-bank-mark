use env_logger::Builder;
use log::{LevelFilter};

pub fn setup_logger(rust_log: Option<&str>) {
    let mut builder = Builder::new();
    builder.filter(None, LevelFilter::Warn);

    rust_log.map(|conf| builder.parse_filters(conf));

    builder.init();
}
