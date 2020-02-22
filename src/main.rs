use relm::Widget;
mod config;
mod events;
mod icons;
mod widgets;

fn main() {
    env_logger::init();
    widgets::win::Win::run(()).unwrap();
}
