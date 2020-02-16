use super::events::EventView;
use super::eventsources::EventSources;
use super::wintitlebar::Msg as WinTitleBarMsg;
use super::wintitlebar::WinTitleBar;
use crate::config::Config;
use gtk::prelude::*;
use relm::{Component, Widget};
use relm_derive::{widget, Msg};
use std::collections::HashMap;

#[derive(Msg)]
pub enum Msg {
    Quit,
    AddConfig((&'static str, String, HashMap<&'static str, String>)),
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
        relm::connect!(titlebar@WinTitleBarMsg::AddConfig((ref providername, ref name, ref cfg)),
                               self.model.relm, Msg::AddConfig((providername, name.clone(), cfg.clone())));
    }

    fn model(relm: &relm::Relm<Self>, config: Config) -> Model {
        let titlebar = relm::init::<WinTitleBar>(()).expect("win title bar init");
        Model {
            relm: relm.clone(),
            config,
            titlebar,
        }
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

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::AddConfig((providername, name, contents)) => {
                let providers = &crate::events::events::get_event_providers();
                let ep = providers
                    .iter()
                    .find(|ep| ep.name() == providername)
                    .unwrap();
                ep.add_config_values(&mut self.model.config, name, contents);
                crate::config::save_config(&self.model.config).unwrap_or_else(|e| {
                    let dialog = gtk::MessageDialog::new(
                        Some(&self.window),
                        gtk::DialogFlags::all(),
                        gtk::MessageType::Error,
                        gtk::ButtonsType::Close,
                        &format!("Error saving the configuration: {}", e),
                    );
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
