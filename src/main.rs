use relm::Widget;
mod events;
mod icons;
mod widgets;

fn main() {
    widgets::win::Win::run(()).unwrap();
}
