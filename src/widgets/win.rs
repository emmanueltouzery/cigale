use super::events::EventView;
use super::eventsources::EventSources;
use crate::config::Config;
use gtk::prelude::*;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    Quit,
    ScreenChanged,
}

pub struct Model {
    relm: relm::Relm<Win>,
    config: Config,
    displaying_event_sources: bool,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        match self.load_style() {
            Err(err) => println!("Error loading the CSS: {}", err),
            _ => {}
        }

        self.main_window_stack_switcher
            .set_stack(Some(&self.main_window_stack));
        self.new_event_source_btn
            .get_style_context()
            .add_class("suggested-action");
        relm::connect!(
            self.model.relm,
            self.main_window_stack,
            connect_property_visible_child_name_notify(_),
            Msg::ScreenChanged
        );
    }

    fn model(relm: &relm::Relm<Self>, config: Config) -> Model {
        Model {
            relm: relm.clone(),
            config,
            displaying_event_sources: false,
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
            Msg::ScreenChanged => {
                self.model.displaying_event_sources = self
                    .main_window_stack
                    .get_visible_child_name()
                    .as_ref()
                    .map(|s| s.as_str())
                    == Some("event-sources");
                self.header_bar.set_subtitle(
                    Some("Event Sources").filter(|_| self.model.displaying_event_sources),
                );
                self.new_event_source_btn
                    .set_visible(self.model.displaying_event_sources);
            }
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            decorated: false, // we have a custom header bar with tabs
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                #[name="header_bar"]
                gtk::HeaderBar {
                    #[name="new_event_source_btn"]
                    gtk::Button {
                        label: "New",
                        visible:false
                    },
                    show_close_button: true,
                    title: Some("Cigale"),
                    #[name="main_window_stack_switcher"]
                    gtk::StackSwitcher {
                        child: {
                            pack_type: gtk::PackType::End
                        }
                    }
                },
                #[name="main_window_stack"]
                gtk::Stack {
                    child: {
                        fill: true,
                        expand: true,
                    },
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
            },
            // Use a tuple when you want to both send a message and return a value to
            // the GTK+ callback.
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}
