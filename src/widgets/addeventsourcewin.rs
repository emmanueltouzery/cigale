use crate::events::events::{ConfigType, EventProvider};
use gtk::prelude::*;
use relm::{init, Component, ContainerWidget, Widget};
use relm_derive::{widget, Msg};
use std::collections::HashMap;

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
            margin_top: 5,
            margin_bottom: 5,
            margin_start: 5,
            margin_end: 5,
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
    AddConfig((&'static str, String, HashMap<&'static str, String>)),
}

pub struct Model {
    relm: relm::Relm<AddEventSourceWin>,
    titlebar: Component<TitleBar>,
    entry_components: Option<HashMap<&'static str, gtk::Widget>>,
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
            entry_components: None,
        }
    }

    fn get_entry_val(entry: &gtk::Widget) -> String {
        let text_entry = entry.clone().dynamic_cast::<gtk::Entry>().ok();
        match text_entry {
            Some(e) => e.get_text().unwrap().to_string(),
            None => entry
                .clone()
                .dynamic_cast::<gtk::FileChooserButton>()
                .unwrap()
                .get_filename()
                .and_then(|f| f.to_str().map(|s| s.to_string()))
                .unwrap_or("".to_string()),
        }
    }

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Close => self.window.close(),
            Msg::Next => {
                if self.model.entry_components.is_some() {
                    let entry_values = self
                        .model
                        .entry_components
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|(k, v)| (*k, AddEventSourceWin::get_entry_val(v)))
                        .collect();
                    let ep = &crate::events::events::get_event_providers()
                        [self.get_provider_index_if_step2()];
                    self.model.relm.stream().emit(Msg::AddConfig((
                        ep.name(),
                        self.provider_name_entry
                            .get_text()
                            .map(|t| t.to_string())
                            .unwrap_or("".to_string()),
                        entry_values,
                    )));
                } else {
                    self.populate_second_step();
                    self.wizard_stack.set_visible_child_name("step2");
                }
            }
            Msg::AddConfig(_) => {}
        }
    }

    fn get_provider_index_if_step2(&self) -> usize {
        self.provider_list
            .get_selected_row()
            .map(|r| r.get_index() as usize)
            .unwrap()
    }

    fn populate_second_step(&mut self) {
        let provider =
            &crate::events::events::get_event_providers()[self.get_provider_index_if_step2()];
        self.config_fields_grid.attach(
            &gtk::Image::new_from_pixbuf(Some(&crate::icons::fontawesome_image(
                provider.default_icon(),
                32,
            ))),
            0,
            0,
            1,
            1,
        );
        let mut i = 1;
        let mut entry_components = HashMap::new();
        for field in provider.get_config_fields() {
            self.config_fields_grid.attach(
                &gtk::LabelBuilder::new().label(field.0).build(),
                1,
                i,
                1,
                1,
            );
            let entry_widget = &match field.1 {
                ConfigType::Text => gtk::Entry::new().upcast::<gtk::Widget>(),
                ConfigType::Path => {
                    gtk::FileChooserButton::new("Pick file", gtk::FileChooserAction::Open)
                        .upcast::<gtk::Widget>()
                }
            };
            entry_components.insert(field.0, entry_widget.clone());
            self.config_fields_grid.attach(entry_widget, 2, i, 1, 1);
            i += 1;
        }
        self.model.entry_components = Some(entry_components);
        self.config_fields_grid.show_all();
    }

    view! {
        #[name="window"]
        gtk::Window {
            delete_event(_, _) => (Msg::Close, Inhibit(false)),
            property_width_request: 350,
            property_height_request: 200,
            titlebar: Some(self.model.titlebar.widget()),
                #[name="wizard_stack"]
                gtk::Stack {
                    gtk::ScrolledWindow {
                        #[name="provider_list"]
                        gtk::ListBox {}
                    },
                    #[name="config_fields_grid"]
                    gtk::Grid {
                        margin_top: 20,
                        margin_bottom: 10,
                        margin_start: 10,
                        margin_end: 10,
                        row_spacing: 5,
                        column_spacing: 10,
                        child: {
                            name: Some("step2")
                        },
                        gtk::Label {
                            label: "Provider name:",
                            cell: {
                                left_attach: 1,
                                top_attach: 0,
                                width: 1,
                                height: 1
                            }
                        },
                        #[name="provider_name_entry"]
                        gtk::Entry {
                            cell: {
                                left_attach: 2,
                                top_attach: 0,
                                width: 1,
                                height: 1
                            }
                        }
                    }
                },
        }
    }
}
