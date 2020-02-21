use crate::events::events::ConfigType;
use gtk::prelude::*;
use relm::{ContainerWidget, Widget};
use relm_derive::{widget, Msg};
use std::collections::{HashMap, HashSet};

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

    fn update(&mut self, _msg: ProviderItemMsg) {}

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

#[derive(Msg, Debug)]
pub enum Msg {
    Next,
    AddConfig(&'static str, String, HashMap<&'static str, String>),
    ProviderNameChanged,
}

pub struct Model {
    relm: relm::Relm<AddEventSourceDialog>,
    entry_components: Option<HashMap<&'static str, gtk::Widget>>,
    existing_provider_names: HashSet<String>,
    next_btn: gtk::Button,
    dialog: gtk::Dialog,
}

pub struct AddEventSourceDialogParams {
    pub existing_provider_names: HashSet<String>,
    pub next_btn: gtk::Button,
    pub dialog: gtk::Dialog,
}

#[widget]
impl Widget for AddEventSourceDialog {
    fn init_view(&mut self) {
        relm::connect!(
            self.model.relm,
            &self.model.next_btn,
            connect_clicked(_),
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

    fn model(relm: &relm::Relm<Self>, dialog_params: AddEventSourceDialogParams) -> Model {
        Model {
            relm: relm.clone(),
            entry_components: None,
            existing_provider_names: dialog_params.existing_provider_names,
            next_btn: dialog_params.next_btn,
            dialog: dialog_params.dialog,
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
            Msg::Next => {
                if self.model.entry_components.is_none() {
                    // we're at the first step: display the second step
                    self.populate_second_step();
                    self.wizard_stack.set_visible_child_name("step2");

                    self.model.next_btn.set_label("Add");
                    self.model.next_btn.set_sensitive(false); // must enter an event source name
                } else {
                    // we're at the second step: add the event source
                    let entry_values = self
                        .model
                        .entry_components
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|(k, v)| (*k, AddEventSourceDialog::get_entry_val(v)))
                        .collect();
                    let ep = &crate::events::events::get_event_providers()
                        [self.get_provider_index_if_step2()];
                    self.model.relm.stream().emit(Msg::AddConfig(
                        ep.name(),
                        self.provider_name_entry
                            .get_text()
                            .map(|t| t.to_string())
                            .unwrap_or("".to_string()),
                        entry_values,
                    ));
                    self.model.dialog.emit_close();
                }
            }
            Msg::ProviderNameChanged => {
                let txt = self.provider_name_entry.get_text();
                let provider_name = txt.as_ref().map(|t| t.as_str()).unwrap_or("");
                let form_is_valid = provider_name.len() > 0
                    && !self.model.existing_provider_names.contains(provider_name);
                self.model.next_btn.set_sensitive(form_is_valid);
            }
            Msg::AddConfig(_, _, _) => {
                // this is meant for wintitlebar... we emit here, not interested by it ourselves
            }
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
                    hexpand: true,
                    cell: {
                        left_attach: 2,
                        top_attach: 0,
                        width: 1,
                        height: 1
                    },
                    changed() => Msg::ProviderNameChanged
                }
            }
        },
    }
}
