use crate::events::events::ConfigType;
use gtk::prelude::*;
use relm::{init, Component, ContainerWidget, Widget};
use relm_derive::{widget, Msg};

/// titlebar

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
            HeaderMsg::Next => self.next_btn.set_label("Add"),
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

/// event provider list item

#[derive(Msg)]
pub enum ProviderItemMsg {}

pub struct ProviderItemModel {
    name: &'static str,
    icon: &'static str,
}

#[widget]
impl Widget for ProviderItem {
    fn init_view(&mut self) {}

    fn model(model: ProviderItemModel) -> ProviderItemModel {
        model
    }

    fn update(&mut self, msg: ProviderItemMsg) {}

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Horizontal,
            gtk::Image {
                child: {
                    padding: 10,
                },
                from_pixbuf: Some(&crate::icons::fontawesome_image(
                    self.model.icon, 32))
            },
            gtk::Label {
                text: self.model.name
            }
        }
    }
}

/// window

#[derive(Msg)]
pub enum Msg {
    Close,
    Next,
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
        relm::connect!(
            titlebar@HeaderMsg::Next,
            self.model.relm,
            Msg::Next
        );
        for provider in crate::events::events::get_event_providers() {
            let _child = self
                .provider_list
                .add_widget::<ProviderItem>(ProviderItemModel {
                    name: provider.name(),
                    icon: provider.default_icon(),
                });
        }
        // select the first event provider by default
        self.provider_list.select_row(Some(
            &self.provider_list.get_children()[0]
                .clone()
                .dynamic_cast::<gtk::ListBoxRow>()
                .unwrap(),
        ));
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
            Msg::Next => {
                self.populate_second_step();
                self.wizard_stack.set_visible_child_name("step2")
            }
        }
    }

    fn populate_second_step(&mut self) {
        let selected_index = self
            .provider_list
            .get_selected_row()
            .map(|r| r.get_index() as usize)
            .unwrap();
        let provider = &crate::events::events::get_event_providers()[selected_index];
        let grid = gtk::GridBuilder::new().build();
        let mut i = 0;
        for field in provider.get_config_fields() {
            grid.attach(&gtk::LabelBuilder::new().label(field.0).build(), 0, i, 1, 1);
            grid.attach(
                &match field.1 {
                    ConfigType::Text => gtk::Entry::new().upcast::<gtk::Widget>(),
                    ConfigType::Path => {
                        gtk::FileChooserButton::new("Pick file", gtk::FileChooserAction::Open)
                            .upcast::<gtk::Widget>()
                    }
                },
                1,
                i,
                1,
                1,
            );
            i += 1;
        }
        grid.show_all();
        self.wizard_stack.add_named(&grid, "step2");
    }

    view! {
        #[name="window"]
        gtk::Window {
            delete_event(_, _) => (Msg::Close, Inhibit(false)),
            property_width_request: 350,
            property_height_request: 250,
            titlebar: Some(self.model.titlebar.widget()),
                #[name="wizard_stack"]
                gtk::Stack {
                    gtk::ScrolledWindow {
                        #[name="provider_list"]
                        gtk::ListBox {}
                    },
                }
        }
    }
}
