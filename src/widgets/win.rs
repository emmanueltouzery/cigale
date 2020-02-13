use super::events::EventView;
use super::eventsources::EventSources;
use crate::config::Config;
use gtk::prelude::*;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    Quit,
}

pub struct Model {
    config: Config,
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
    }

    fn model(config: Config) -> Model {
        Model { config: config }
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
            decorated: false, // we have a custom header bar with tabs
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                #[name="header_bar"]
                gtk::HeaderBar {
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
                            icon_name: Some("view-list-symbolic")
                        },
                    },
                    EventSources(self.model.config.clone()) {
                        child: {
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
