use super::addeventsourcedlg::EventSourceEditModel;
use super::addeventsourcedlg::Msg as AddEventSourceDialogMsg;
use super::events::EventView;
use super::eventsources::EventSources;
use super::eventsources::Msg as EventSourcesMsg;
use super::wintitlebar::Msg as WinTitleBarMsg;
use super::wintitlebar::WinTitleBar;
use crate::config::Config;
use crate::events::events::EventProvider;
use glib::signal::Inhibit;
use gtk::prelude::*;
use gtk::traits::SettingsExt;
use relm::{Component, Widget};
use relm_derive::{widget, Msg};
use std::collections::{HashMap, HashSet};

const CSS_DATA: &[u8] = include_bytes!("../../resources/style.css");

#[derive(Msg)]
pub enum Msg {
    Quit,
    AddConfig(&'static str, String, HashMap<&'static str, String>),
    EditConfig(String, &'static str, String, HashMap<&'static str, String>),
    EditEventSource(&'static str, String),
    RemoveEventSource(&'static str, String),
    KeyPress(gdk::EventKey),
    ConfigUpdated(Box<Config>),
}

pub struct Model {
    relm: relm::Relm<Win>,
    config: Config,
    titlebar: Component<WinTitleBar>,
    accel_group: gtk::AccelGroup,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        if let Err(err) = self.load_style() {
            println!("Error loading the CSS: {}", err);
        }

        self.widgets.window.add_accel_group(&self.model.accel_group);
        let titlebar = &self.model.titlebar;
        titlebar.emit(super::wintitlebar::Msg::MainWindowStackReady(
            self.widgets.main_window_stack.clone(),
        ));
        relm::connect!(titlebar@WinTitleBarMsg::AddConfig(providername, ref name, ref cfg),
                               self.model.relm, Msg::AddConfig(providername, name.clone(), cfg.clone()));
        relm::connect!(titlebar@WinTitleBarMsg::ConfigUpdated(ref cfg),
                       self.model.relm, Msg::ConfigUpdated(cfg.clone()));
        let event_sources = &self.components.event_sources;
        relm::connect!(event_sources@EventSourcesMsg::RemoveEventSource(providername, ref name),
                               self.model.relm, Msg::RemoveEventSource(providername, name.clone()));
        relm::connect!(event_sources@EventSourcesMsg::EditEventSource(providername, ref name),
                               self.model.relm, Msg::EditEventSource(providername, name.clone()));
        self.update_event_sources_need_attention();
    }

    fn model(relm: &relm::Relm<Self>, _: ()) -> Model {
        gtk::IconTheme::default()
            .unwrap()
            .add_resource_path("/icons");
        let config = Config::read_config();
        gtk::Settings::default()
            .unwrap()
            .set_gtk_application_prefer_dark_theme(config.prefer_dark_theme);
        let titlebar = relm::init::<WinTitleBar>(Win::config_source_names(&config))
            .expect("win title bar init");
        let accel_group = gtk::AccelGroup::new();
        Model {
            relm: relm.clone(),
            config,
            titlebar,
            accel_group,
        }
    }

    pub fn config_source_names(config: &Config) -> HashSet<String> {
        crate::events::events::get_event_providers()
            .iter()
            .flat_map(|ep| {
                ep.get_config_names(config)
                    .iter()
                    .map(|n| (*n).clone())
                    .collect::<Vec<String>>()
            })
            .collect()
    }

    // we use the 'needs-attention' hint on the 'event sources'
    // tab when there are no event sources configured, because
    // the app won't be useful until we have event sources.
    fn update_event_sources_need_attention(&self) {
        self.widgets.main_window_stack.set_child_needs_attention(
            &self.widgets.event_sources,
            Self::config_source_names(&self.model.config).is_empty(),
        );
    }

    fn load_style(&self) -> Result<(), Box<dyn std::error::Error>> {
        let screen = self.widgets.window.screen().unwrap();
        let css = gtk::CssProvider::new();
        css.load_from_data(CSS_DATA)?;
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        Ok(())
    }

    fn get_event_provider_by_name<'a>(
        providers: &'a [Box<dyn EventProvider>],
        providername: &'static str,
    ) -> &'a dyn EventProvider {
        providers
            .iter()
            .find(|ep| ep.name() == providername)
            .unwrap()
            .as_ref()
    }

    pub fn save_event_providers(&self) {
        self.model.config.save_config(&self.widgets.window);
        self.propagate_config_change();
    }

    fn propagate_config_change(&self) {
        self.streams
            .event_sources
            .emit(super::eventsources::Msg::ConfigUpdate(Box::new(
                self.model.config.clone(),
            )));
        self.streams
            .events
            .emit(super::events::Msg::ConfigUpdate(Box::new(
                self.model.config.clone(),
            )));
        self.model
            .titlebar
            .stream()
            .emit(WinTitleBarMsg::EventSourceNamesChanged(
                Win::config_source_names(&self.model.config),
            ));
    }

    fn update(&mut self, event: Msg) {
        let providers = &crate::events::events::get_event_providers();
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::AddConfig(providername, name, contents) => {
                let ep = Win::get_event_provider_by_name(providers, providername);
                ep.add_config_values(&mut self.model.config, name, contents);
                self.save_event_providers();
            }
            Msg::EditConfig(configname, providername, name, contents) => {
                let ep = Win::get_event_provider_by_name(providers, providername);
                ep.remove_config(&mut self.model.config, configname);
                ep.add_config_values(&mut self.model.config, name, contents);
                self.save_event_providers();
            }
            Msg::RemoveEventSource(ep_name, config_name) => {
                let dialog = gtk::MessageDialog::new(
                    Some(&self.widgets.window),
                    gtk::DialogFlags::all(),
                    gtk::MessageType::Warning,
                    gtk::ButtonsType::None,
                    "Remove event source",
                );
                dialog.set_secondary_text(Some(&format!(
                    "Are you sure you want to remove the '{}' event source?",
                    config_name
                )));
                dialog.add_button("Cancel", gtk::ResponseType::Cancel);
                let remove_btn = dialog.add_button("Remove", gtk::ResponseType::Yes);
                remove_btn.style_context().add_class("destructive-action");
                let r = dialog.run();
                dialog.close();
                if r == gtk::ResponseType::Yes {
                    let ep = Win::get_event_provider_by_name(providers, ep_name);
                    ep.remove_config(&mut self.model.config, config_name);
                    self.save_event_providers();
                }
            }
            Msg::EditEventSource(ep_name, config_name) => {
                let mut config_source_names = Win::config_source_names(&self.model.config);
                config_source_names.remove(&config_name); // allow to use the current config name in the edit dialog
                let ep = Win::get_event_provider_by_name(providers, ep_name);
                let event_source_values = ep.get_config_values(&self.model.config, &config_name);
                let (dialog, dialog_contents) = WinTitleBar::prepare_addedit_eventsource_dlg(
                    &self.widgets.window,
                    &config_source_names,
                    Some(EventSourceEditModel {
                        event_provider_name: ep_name,
                        event_source_name: config_name,
                        event_source_values,
                    }),
                );
                relm::connect!(dialog_contents@AddEventSourceDialogMsg::EditConfig(ref configname, providername, ref name, ref cfg),
                               self.model.relm, Msg::EditConfig(configname.clone(), providername, name.clone(), cfg.clone()));
                let resp = dialog.run();
                match resp {
                    gtk::ResponseType::Cancel | gtk::ResponseType::DeleteEvent => dialog.close(),
                    _ => {}
                }
            }
            Msg::KeyPress(key) => {
                if key.state().contains(gdk::ModifierType::CONTROL_MASK)
                    && key.state().contains(gdk::ModifierType::MOD1_MASK)
                    && key.keyval() == gdk::keys::constants::y
                {
                    self.components
                        .events
                        .emit(super::events::Msg::CopyAllHeaders);
                }
            }
            Msg::ConfigUpdated(cfg) => {
                self.model.config = *cfg;
                self.propagate_config_change();
            }
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            titlebar: Some(self.model.titlebar.widget()),
            default_width: 1000,
            default_height: 650,
            #[name="main_window_stack"]
            gtk::Stack {
                #[name="events"]
                EventView((self.model.config.clone(), self.model.accel_group.clone())) {
                    child: {
                        name: Some("events"),
                        icon_name: Some("view-list-symbolic")
                    },
                },
                #[name="event_sources"]
                EventSources(self.model.config.clone()) {
                    child: {
                        name: Some("event-sources"),
                        icon_name: Some("document-properties-symbolic")
                    },
                }
            },
            // Use a tuple when you want to both send a message and return a value to
            // the GTK+ callback.
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
            key_press_event(_, key) => (Msg::KeyPress(key.clone()), Inhibit(false)),
        }
    }
}
