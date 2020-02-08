use relm::Widget;
mod config;
mod events;
mod icons;
mod widgets;

fn main() {
    let config = config::read_config().expect("Failed to read config");
    widgets::win::Win::run(config).unwrap();
}
