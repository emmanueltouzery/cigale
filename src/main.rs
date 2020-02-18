use relm::Widget;
mod config;
mod events;
mod icons;
mod widgets;

fn main() {
    widgets::win::Win::run(()).unwrap();
}
