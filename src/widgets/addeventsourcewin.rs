use gtk::prelude::*;
use relm::{init, Component, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum HeaderMsg {
    Close,
    Next,
}

pub struct HeaderModel {
    relm: relm::Relm<TitleBar>,
}

#[widget]
impl Widget for TitleBar {
    fn init_view(&mut self) {
        self.next_btn
            .get_style_context()
            .add_class("suggested-action");
    }

    fn model(relm: &relm::Relm<Self>, _: ()) -> HeaderModel {
        HeaderModel { relm: relm.clone() }
    }

    fn update(&mut self, msg: HeaderMsg) {
        match msg {
            _ => {}
        }
    }

    view! {
        gtk::HeaderBar {
            delete_event(_, _) => (HeaderMsg::Close, Inhibit(false)),
            title: Some("Add event source"),
            gtk::Button {
                label: "Close",
                clicked() => HeaderMsg::Close,
            },
            #[name="next_btn"]
            gtk::Button {
                label: "Next",
                child: {
                    pack_type: gtk::PackType::End
                },
                clicked() => HeaderMsg::Next,
            },
        }
    }
}

#[derive(Msg)]
pub enum Msg {
    Close,
}

pub struct Model {
    relm: relm::Relm<AddEventSourceWin>,
    titlebar: Component<TitleBar>,
}

#[widget]
impl Widget for AddEventSourceWin {
    fn init_view(&mut self) {
        let titlebar = &self.model.titlebar;
        relm::connect!(
            titlebar@HeaderMsg::Close,
            self.model.relm,
            Msg::Close
        );
    }

    fn model(relm: &relm::Relm<Self>, _: ()) -> Model {
        Model {
            relm: relm.clone(),
            titlebar: init::<TitleBar>(()).expect("Error building the titlebar"),
        }
    }

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Close => self.window.close(),
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            delete_event(_, _) => (Msg::Close, Inhibit(false)),
            titlebar: Some(self.model.titlebar.widget())
        }
    }
}
