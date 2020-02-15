use gtk::prelude::*;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    Quit,
}

pub struct Model {
    relm: relm::Relm<AddEventSourceWin>,
}

#[widget]
impl Widget for AddEventSourceWin {
    fn model() -> () {}

    fn update(&mut self, _msg: Msg) {}

    view! {
        gtk::Window {
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}
