use gtk::prelude::*;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    ScreenChanged,
    MainWindowStackReady(gtk::Stack),
}

pub struct Model {
    relm: relm::Relm<WinTitleBar>,
    displaying_event_sources: bool,
    main_window_stack: Option<gtk::Stack>,
}

#[widget]
impl Widget for WinTitleBar {
    fn init_view(&mut self) {
        self.new_event_source_btn
            .get_style_context()
            .add_class("suggested-action");
    }

    fn model(relm: &relm::Relm<Self>, _: ()) -> Model {
        Model {
            relm: relm.clone(),
            displaying_event_sources: false,
            main_window_stack: None,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::MainWindowStackReady(stack) => {
                self.model.main_window_stack = Some(stack.clone());
                self.main_window_stack_switcher
                    .set_stack(self.model.main_window_stack.as_ref());
                relm::connect!(
                    self.model.relm,
                    &stack,
                    connect_property_visible_child_name_notify(_),
                    Msg::ScreenChanged
                );
            }
            Msg::ScreenChanged => {
                self.model.displaying_event_sources = self
                    .model
                    .main_window_stack
                    .as_ref()
                    .unwrap()
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
        }
    }
}
