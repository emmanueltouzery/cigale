use super::addeventsourcedlg::EventSourceEditModel;
use super::addeventsourcedlg::Msg as AddEventSourceDialogMsg;
use super::events::EventView;
use super::eventsources::EventSources;
use super::eventsources::Msg as EventSourcesMsg;
use super::wintitlebar::Msg as WinTitleBarMsg;
use super::wintitlebar::WinTitleBar;
use crate::config::Config;
use crate::events::events::EventProvider;
use gtk::prelude::*;
use relm::{Component, Widget};
use relm_derive::{widget, Msg};
use std::collections::{HashMap, HashSet};

#[derive(Msg)]
pub enum Msg {
    Quit,
    AddConfig(&'static str, String, HashMap<&'static str, String>),
    EditConfig(String, &'static str, String, HashMap<&'static str, String>),
    EditEventSource(&'static str, String),
    RemoveEventSource(&'static str, String),
}

pub struct Model {
    relm: relm::Relm<Win>,
    config: Config,
    titlebar: Component<WinTitleBar>,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        match self.load_style() {
            Err(err) => println!("Error loading the CSS: {}", err),
            _ => {}
        }
        let titlebar = &self.model.titlebar;
        titlebar.emit(super::wintitlebar::Msg::MainWindowStackReady(
            self.main_window_stack.clone(),
        ));
        relm::connect!(titlebar@WinTitleBarMsg::AddConfig(ref providername, ref name, ref cfg),
                               self.model.relm, Msg::AddConfig(providername, name.clone(), cfg.clone()));
        let event_sources = &self.event_sources;
        relm::connect!(event_sources@EventSourcesMsg::RemoveEventSource(ref providername, ref name),
                               self.model.relm, Msg::RemoveEventSource(providername, name.clone()));
        relm::connect!(event_sources@EventSourcesMsg::EditEventSource(ref providername, ref name),
                               self.model.relm, Msg::EditEventSource(providername, name.clone()));
    }

    fn model(relm: &relm::Relm<Self>, _: ()) -> Model {
        let config = Config::read_config().unwrap_or_else(|e| {
            let dialog = gtk::MessageDialog::new(
                None::<&gtk::Window>,
                gtk::DialogFlags::all(),
                gtk::MessageType::Error,
                gtk::ButtonsType::Close,
                &format!("Error loading the configuration"),
            );
            dialog.set_property_secondary_text(Some(&format!(
                "{}: {:}",
                Config::config_path()
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or("".to_string()),
                e
            )));
            let _r = dialog.run();
            dialog.destroy();
            Config::default_config()
        });
        let titlebar = relm::init::<WinTitleBar>(Win::config_source_names(&config))
            .expect("win title bar init");
        Model {
            relm: relm.clone(),
            config,
            titlebar,
        }
    }

    fn config_source_names(config: &Config) -> HashSet<String> {
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

    fn load_style(&self) -> Result<(), Box<dyn std::error::Error>> {
        let screen = self.window.get_screen().unwrap();
        let css = gtk::CssProvider::new();

        // TODO embed the css in the binary?
        let mut path = std::path::PathBuf::new();
        path.push("resources");
        path.push("style.css");
        let path_str = path.to_str().ok_or("Invalid path")?;
        css.load_from_path(path_str)?;
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        Ok(())
    }

    fn get_event_provider_by_name<'a>(
        providers: &'a Vec<Box<dyn EventProvider>>,
        providername: &'static str,
    ) -> &'a Box<dyn EventProvider> {
        providers
            .iter()
            .find(|ep| ep.name() == providername)
            .unwrap()
    }

    fn save_config(&self) {
        Config::save_config(&self.model.config).unwrap_or_else(|e| {
            let dialog = gtk::MessageDialog::new(
                Some(&self.window),
                gtk::DialogFlags::all(),
                gtk::MessageType::Error,
                gtk::ButtonsType::Close,
                &format!("Error saving the configuration"),
            );
            dialog.set_property_secondary_text(Some(&format!("{}", e)));
            let _r = dialog.run();
            dialog.destroy();
        });
        self.event_sources
            .stream()
            .emit(super::eventsources::Msg::ConfigUpdate(
                self.model.config.clone(),
            ));
        self.events
            .stream()
            .emit(super::events::Msg::ConfigUpdate(self.model.config.clone()));
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
                self.save_config();
            }
            Msg::EditConfig(configname, providername, name, contents) => {
                let ep = Win::get_event_provider_by_name(providers, providername);
                ep.remove_config(&mut self.model.config, configname);
                ep.add_config_values(&mut self.model.config, name, contents);
                self.save_config();
            }
            Msg::RemoveEventSource(ep_name, config_name) => {
                let dialog = gtk::MessageDialog::new(
                    Some(&self.window),
                    gtk::DialogFlags::all(),
                    gtk::MessageType::Warning,
                    gtk::ButtonsType::None,
                    "Remove event source",
                );
                dialog.set_property_secondary_text(Some(&format!(
                    "Are you sure you want to remove the '{}' event source?",
                    config_name
                )));
                dialog.add_button("Cancel", gtk::ResponseType::Cancel);
                let remove_btn = dialog.add_button("Remove", gtk::ResponseType::Yes);
                remove_btn
                    .get_style_context()
                    .add_class("destructive-action");
                let r = dialog.run();
                dialog.destroy();
                if r == gtk::ResponseType::Yes {
                    let ep = Win::get_event_provider_by_name(providers, ep_name);
                    ep.remove_config(&mut self.model.config, config_name);
                    self.save_config();
                }
            }
            Msg::EditEventSource(ep_name, config_name) => {
                let mut config_source_names = Win::config_source_names(&self.model.config);
                config_source_names.remove(&config_name); // allow to use the current config name in the edit dialog
                let ep = Win::get_event_provider_by_name(providers, ep_name);
                let event_source_values = ep.get_config_values(&self.model.config, &config_name);
                let (dialog, dialog_contents) = WinTitleBar::prepare_addedit_eventsource_dlg(
                    &self.window,
                    &config_source_names,
                    Some(EventSourceEditModel {
                        event_provider_name: ep_name,
                        event_source_name: config_name,
                        event_source_values,
                    }),
                );
                relm::connect!(dialog_contents@AddEventSourceDialogMsg::EditConfig(ref configname, ref providername, ref name, ref cfg),
                               self.model.relm, Msg::EditConfig(configname.clone(), providername, name.clone(), cfg.clone()));
                let resp = dialog.run();
                match resp {
                    gtk::ResponseType::Cancel | gtk::ResponseType::DeleteEvent => dialog.destroy(),
                    _ => {}
                }
            }
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            titlebar: Some(self.model.titlebar.widget()),
            #[name="main_window_stack"]
            gtk::Stack {
                #[name="events"]
                EventView(self.model.config.clone()) {
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
        }
    }
}
