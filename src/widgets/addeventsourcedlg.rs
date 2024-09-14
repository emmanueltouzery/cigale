use crate::config::Config;
use crate::events::events::{get_event_providers, ConfigType, EventProvider};
use crate::icons::*;
use gtk::builders::*;
use gtk::prelude::*;
use relm::{ContainerWidget, Widget};
use relm_derive::{widget, Msg};
use std::collections::{HashMap, HashSet};

/// event provider list item

#[derive(Msg)]
pub enum ProviderItemMsg {}

pub struct ProviderItemModel {
    name: &'static str,
    icon: Icon,
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
                icon_name: Some(self.model.icon.name()),
                // https://github.com/gtk-rs/gtk/issues/837
                icon_size: gtk::IconSize::Dnd,
            },
            gtk::Label {
                text: self.model.name
            }
        }
    }
}

/// dialog

#[derive(Msg, Debug)]
pub enum Msg {
    Next,
    EditSave,
    AddConfig(&'static str, String, HashMap<&'static str, String>),
    EditConfig(String, &'static str, String, HashMap<&'static str, String>),
    SourceNameChanged,
    FormChanged,
}

pub struct Model {
    relm: relm::Relm<AddEventSourceDialog>,
    entry_components: Option<HashMap<&'static str, gtk::Widget>>,
    existing_source_names: HashSet<String>,
    existing_source_names_sanitized: HashSet<String>,
    next_btn: gtk::Button,
    dialog: gtk::Dialog,
    edit_model: Option<EventSourceEditModel>,
    event_provider: Option<Box<dyn EventProvider>>,
}

#[derive(Clone)]
pub struct EventSourceEditModel {
    pub event_provider_name: &'static str,
    pub event_source_name: String,
    pub event_source_values: HashMap<&'static str, String>,
}

pub struct AddEventSourceDialogParams {
    pub existing_source_names: HashSet<String>,
    pub next_btn: gtk::Button,
    pub dialog: gtk::Dialog,
    pub edit_model: Option<EventSourceEditModel>,
}

#[widget]
impl Widget for AddEventSourceDialog {
    fn init_view(&mut self) {
        match self.model.edit_model {
            None => self.init_add(),
            _ => self.init_edit(),
        }
    }

    fn init_add(&mut self) {
        relm::connect!(
            self.model.relm,
            &self.model.next_btn,
            connect_clicked(_),
            Msg::Next
        );
        for provider in get_event_providers() {
            let _child = self
                .widgets
                .provider_list
                .add_widget::<ProviderItem>(ProviderItemModel {
                    name: provider.name(),
                    icon: provider.default_icon(),
                });
        }
        // select the first event provider by default
        self.widgets.provider_list.select_row(Some(
            &self.widgets.provider_list.children()[0]
                .clone()
                .dynamic_cast::<gtk::ListBoxRow>()
                .unwrap(),
        ));
    }

    fn init_edit(&mut self) {
        // i'd rather be given the unwrapped model by the caller,
        // but rustc bugs me about multiple borrows of self.
        let edit_model = self.model.edit_model.clone().unwrap(); // annoying to clone
        relm::connect!(
            self.model.relm,
            &self.model.next_btn,
            connect_clicked(_),
            Msg::EditSave
        );
        let ep = get_event_providers()
            .into_iter()
            .find(|ep| ep.name() == edit_model.event_provider_name)
            .unwrap();
        self.populate_second_step(
            ep,
            &edit_model.event_source_name,
            &edit_model.event_source_values,
        );
        self.widgets.wizard_stack.set_visible_child_name("step2");
        self.model.next_btn.set_label("Save");
    }

    fn model(relm: &relm::Relm<Self>, dialog_params: AddEventSourceDialogParams) -> Model {
        Model {
            relm: relm.clone(),
            entry_components: None,
            existing_source_names_sanitized: dialog_params
                .existing_source_names
                .iter()
                .map(|s| Config::sanitize_for_filename(s).to_string())
                .collect(),
            existing_source_names: dialog_params.existing_source_names,
            next_btn: dialog_params.next_btn,
            dialog: dialog_params.dialog,
            edit_model: dialog_params.edit_model,
            event_provider: None,
        }
    }

    fn get_entry_val(&self, field_name: &'static str, entry: &gtk::Widget) -> String {
        let field_type = self
            .model
            .event_provider
            .as_ref()
            .unwrap()
            .get_config_fields()
            .iter()
            .find(|f| f.0 == field_name)
            .unwrap()
            .1;
        match field_type {
            ConfigType::File | ConfigType::Folder => entry
                .clone()
                .dynamic_cast::<gtk::FileChooserButton>()
                .unwrap()
                .filename()
                .and_then(|f| f.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "".to_string()),
            ConfigType::Text(_) | ConfigType::Password => entry
                .clone()
                .dynamic_cast::<gtk::Entry>()
                .unwrap()
                .text()
                .to_string(),
            ConfigType::Combo => entry
                .clone()
                .dynamic_cast::<gtk::ComboBoxText>()
                .unwrap()
                .active_text()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string()),
        }
    }

    fn get_entry_values(&self) -> HashMap<&'static str, String> {
        self.model
            .entry_components
            .as_ref()
            .unwrap()
            .iter()
            .map(|(k, v)| (*k, self.get_entry_val(k, v)))
            .collect()
    }

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Next => {
                if self.model.entry_components.is_none() {
                    // we're at the first step: display the second step
                    let provider = crate::events::events::get_event_providers()
                        .remove(self.get_provider_index_if_step2());
                    self.populate_second_step(provider, &"".to_string(), &HashMap::new());
                    self.widgets.wizard_stack.set_visible_child_name("step2");

                    self.model.next_btn.set_label("Add");
                    self.model.next_btn.set_sensitive(false); // must enter an event source name
                } else {
                    // we're at the second step: add the event source
                    let ep = &crate::events::events::get_event_providers()
                        [self.get_provider_index_if_step2()];
                    self.model.relm.stream().emit(Msg::AddConfig(
                        ep.name(),
                        self.widgets.provider_name_entry.text().to_string(),
                        self.get_entry_values(),
                    ));
                    self.model.dialog.emit_close();
                }
            }
            Msg::EditSave => {
                self.model.relm.stream().emit(Msg::EditConfig(
                    self.model
                        .edit_model
                        .as_ref()
                        .unwrap()
                        .event_source_name
                        .clone(),
                    self.model.edit_model.as_ref().unwrap().event_provider_name,
                    self.widgets.provider_name_entry.text().to_string(),
                    self.get_entry_values(),
                ));
                self.model.dialog.emit_close();
            }
            Msg::SourceNameChanged => {
                let txt = self.widgets.provider_name_entry.text();
                let source_name = txt.as_str();
                let form_is_valid = !source_name.is_empty()
                    && !self.model.existing_source_names.contains(source_name)
                    && !self
                        .model
                        .existing_source_names_sanitized
                        .contains(&Config::sanitize_for_filename(source_name).to_string());
                self.model.next_btn.set_sensitive(form_is_valid);
            }
            Msg::AddConfig(_, _, _) => {
                // this is meant for wintitlebar... we emit here, not interested by it ourselves
            }
            Msg::EditConfig(_, _, _, _) => {
                // this is meant for wintitlebar... we emit here, not interested by it ourselves
            }
            Msg::FormChanged => {
                self.update_form();
            }
        }
    }

    fn get_provider_index_if_step2(&self) -> usize {
        self.widgets
            .provider_list
            .selected_row()
            .map(|r| r.index() as usize)
            .unwrap()
    }

    fn update_form(&mut self) {
        // for now combo boxes can be loaded when other fields
        // are updated. we use that for git author names, which
        // depends on the git repo path.
        let fields = self
            .model
            .event_provider
            .as_ref()
            .unwrap()
            .get_config_fields();
        // assumes only one combo max
        let combo_pos = fields.iter().position(|e| e.1 == ConfigType::Combo);
        if let Some(idx) = combo_pos {
            let field_name = fields[idx].0;
            let combo_widget = self.model.entry_components.as_ref().unwrap()[field_name].clone();
            self.refresh_combo(combo_widget, field_name, &self.get_entry_values());
        }
    }

    fn refresh_combo(
        &self,
        combo_widget: gtk::Widget,
        field_name: &'static str,
        entry_values: &HashMap<&'static str, String>,
    ) -> Vec<String> {
        let combo = combo_widget
            .dynamic_cast::<gtk::ComboBoxText>()
            .expect("upcast combobox");
        combo.remove_all();
        let values = self
            .model
            .event_provider
            .as_ref()
            .expect("reading event provider from model")
            .field_values(entry_values, field_name)
            .unwrap_or_else(|e| {
                println!("fetching field values failed: {}", e);
                Vec::new()
            });
        for value in &values {
            combo.append_text(value);
        }
        values
    }

    fn populate_second_step(
        &mut self,
        provider: Box<dyn EventProvider>,
        event_source_name: &str,
        event_source_values: &HashMap<&'static str, String>,
    ) {
        self.model.event_provider = Some(provider);
        let p = self.model.event_provider.as_ref().unwrap();
        self.widgets.provider_name_entry.set_text(event_source_name);
        self.widgets.config_fields_grid.attach(
            &gtk::Image::from_icon_name(Some(p.default_icon().name()), gtk::IconSize::Dnd),
            0,
            0,
            1,
            1,
        );
        let mut i = 1;
        let mut entry_components = HashMap::new();
        for field in p.get_config_fields() {
            let field_val = event_source_values.get(field.0).map(|s| s.as_str());
            self.widgets.config_fields_grid.attach(
                &LabelBuilder::new()
                    .label(field.0)
                    .halign(gtk::Align::End)
                    .build(),
                1,
                i,
                1,
                1,
            );
            let entry_widget = &match field.1 {
                ConfigType::Text(def) => EntryBuilder::new()
                    .text(field_val.unwrap_or(def))
                    .build()
                    .upcast::<gtk::Widget>(),
                ConfigType::File => {
                    let btn =
                        gtk::FileChooserButton::new("Pick file", gtk::FileChooserAction::Open);
                    if let Some(u) = event_source_values.get(field.0) {
                        btn.set_filename(u);
                    }
                    relm::connect!(self.model.relm, btn, connect_file_set(_), Msg::FormChanged);
                    btn.upcast::<gtk::Widget>()
                }
                ConfigType::Folder => {
                    let btn = gtk::FileChooserButton::new(
                        "Pick folder",
                        gtk::FileChooserAction::SelectFolder,
                    );
                    if let Some(u) = event_source_values.get(field.0) {
                        btn.set_filename(u);
                    }
                    relm::connect!(self.model.relm, btn, connect_file_set(_), Msg::FormChanged);
                    btn.upcast::<gtk::Widget>()
                }
                ConfigType::Password => EntryBuilder::new()
                    .text(field_val.unwrap_or(""))
                    .visibility(false) // password field
                    .secondary_icon_name(Icon::EXCLAMATION_TRIANGLE.name())
                    .secondary_icon_tooltip_text("Passwords are not encrypted in the config file")
                    .build()
                    .upcast::<gtk::Widget>(),
                ConfigType::Combo => {
                    let combo = gtk::ComboBoxText::new();
                    let combo_items = self.refresh_combo(
                        combo.clone().upcast::<gtk::Widget>(),
                        field.0,
                        event_source_values,
                    );
                    combo.set_active(
                        combo_items
                            .iter()
                            .position(|i| i == field_val.unwrap_or(""))
                            .map(|p| p as u32),
                    );
                    combo.upcast::<gtk::Widget>()
                }
            };
            entry_components.insert(field.0, entry_widget.clone());
            self.widgets
                .config_fields_grid
                .attach(entry_widget, 2, i, 1, 1);
            i += 1;
        }
        self.model.entry_components = Some(entry_components);
        self.widgets.config_fields_grid.show_all();
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
                    label: "Provider name",
                    halign: gtk::Align::End,
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
                    changed() => Msg::SourceNameChanged
                }
            }
        },
    }
}
