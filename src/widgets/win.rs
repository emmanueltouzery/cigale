use super::events::EventView;
use super::eventsources::EventSources;
use super::wintitlebar::WinTitleBar;
use crate::config::Config;
use gtk::prelude::*;
use relm::{Component, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    Quit,
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
        self.model
            .titlebar
            .emit(super::wintitlebar::Msg::MainWindowStackReady(
                self.main_window_stack.clone(),
            ));
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
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            titlebar: Some(self.model.titlebar.widget()),
            #[name="main_window_stack"]
            gtk::Stack {
                EventView(self.model.config.clone()) {
                    child: {
                        name: Some("events"),
                        icon_name: Some("view-list-symbolic")
                    },
                },
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
