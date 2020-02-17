use super::addeventsourcewin::AddEventSourceWin;
use super::addeventsourcewin::Msg as AddEventSourceWinMsg;
use gtk::prelude::*;
use relm::{init, Component, Widget};
use relm_derive::{widget, Msg};
use std::collections::HashMap;

#[derive(Msg)]
pub enum Msg {
    ScreenChanged,
    MainWindowStackReady(gtk::Stack),
    NewEventSourceClick,
    AddConfig(&'static str, String, HashMap<&'static str, String>),
}

pub struct Model {
    relm: relm::Relm<WinTitleBar>,
    displaying_event_sources: bool,
    main_window_stack: Option<gtk::Stack>,
    add_event_source_win: Option<Component<AddEventSourceWin>>,
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
            add_event_source_win: None,
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
            Msg::NewEventSourceClick => {
                self.model.add_event_source_win = Some(
                    init::<AddEventSourceWin>(())
                        .expect("error initializing the add event source modal"),
                );
                let src = self.model.add_event_source_win.as_ref().unwrap();
                relm::connect!(src@AddEventSourceWinMsg::AddConfig(ref providername, ref name, ref cfg),
                               self.model.relm, Msg::AddConfig(providername, name.clone(), cfg.clone()));
                let main_win = self
                    .model
                    .main_window_stack
                    .as_ref()
                    .unwrap()
                    .get_toplevel()
                    .and_then(|w| w.dynamic_cast::<gtk::Window>().ok());
                self.model
                    .add_event_source_win
                    .as_ref()
                    .unwrap()
                    .widget()
                    .set_transient_for(main_win.as_ref());
            }
            Msg::AddConfig(_, _, _) => {}
        }
    }

    view! {
        #[name="header_bar"]
        gtk::HeaderBar {
            #[name="new_event_source_btn"]
            gtk::Button {
                label: "New",
                visible:false,
                clicked() => Msg::NewEventSourceClick,
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
