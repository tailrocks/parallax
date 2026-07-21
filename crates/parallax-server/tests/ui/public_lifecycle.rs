use parallax_server::{Config, ServerHandle, start};

fn accepts(_: Option<ServerHandle>) {}

fn main() {
    let _ = (Config::default(), start);
    accepts(None);
}
