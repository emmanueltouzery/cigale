use super::datepicker::DatePickerMsg::DayPicked as DatePickerDayPickedMsg;
use super::datepicker::*;
use super::event::EventListItem;
use crate::events::events::Event;
use chrono::prelude::*;
use gtk::prelude::*;
use relm::{Channel, ContainerWidget, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    Quit,
    EventSelected,
    DayChange(Date<Local>),
    GotEvents(Result<Vec<Event>, String>),
}

pub struct Model {
    relm: relm::Relm<Win>,
    // events will be None while we're loading
    events: Option<Result<Vec<Event>, String>>,
    current_event: Option<Event>,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        match self.load_style() {
            Err(err) => println!("Error loading the CSS: {}", err),
            _ => {}
        }

        self.header_label
            .get_style_context()
            .add_class("event_header_label");
        self.update_events();

        relm::connect!(
            self.model.relm,
            self.event_list,
            connect_row_selected(_, _),
            Msg::EventSelected
        );
    }

    fn model(relm: &relm::Relm<Self>, _: ()) -> Model {
        Win::fetch_events(relm, &Local::today());
        Model {
            relm: relm.clone(),
            events: None,
            current_event: None,
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
        gtk::StyleContext::add_provider_for_screen(&screen, &css, 0);
        Ok(())
    }

    fn fetch_events(relm: &relm::Relm<Self>, day: &Date<Local>) {
        let dday = day.clone();
        let stream = relm.stream().clone();
        let (_channel, sender) = Channel::new(move |events| {
            stream.emit(Msg::GotEvents(events));
        });
        std::thread::spawn(move || {
            sender
                .send(crate::events::events::get_all_events(&dday).map_err(|e| e.to_string()))
                .unwrap_or_else(|err| println!("Thread communication error: {}", err));
        });
    }

    fn update_events(&mut self) {
        self.model.current_event = None;
        for child in self.event_list.get_children() {
            self.event_list.remove(&child);
        }
        match &self.model.events {
            Some(Ok(events)) => {
                for event in events {
                    let _child = self.event_list.add_widget::<EventListItem>(event.clone());
                }
            }
            Some(Err(err)) => {
                let info_contents = self
                    .info_bar
                    .get_content_area()
                    .unwrap()
                    .dynamic_cast::<gtk::Box>() // https://github.com/gtk-rs/gtk/issues/947
                    .unwrap();
                for child in info_contents.get_children() {
                    info_contents.remove(&child);
                }
                info_contents.add(&gtk::Label::new(Some(err.to_string().as_str())));
                info_contents.show_all();
            }
            None => {}
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::EventSelected => match &self.model.events {
                Some(Ok(events)) => {
                    let selected_index_maybe = self
                        .event_list
                        .get_selected_row()
                        .map(|r| r.get_index() as usize);
                    self.model.current_event = selected_index_maybe
                        .and_then(|idx| events.get(idx))
                        .map(|evt| evt.clone());
                }
                _ => {}
            },
            Msg::DayChange(day) => {
                self.model.events = None;
                self.update_events();
                Win::fetch_events(&self.model.relm, &day);
            }
            Msg::GotEvents(events) => {
                self.model.events = Some(events);
                self.update_events();
            }
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                gtk::Box {
                    orientation: gtk::Orientation::Horizontal,
                    DatePicker {
                        DatePickerDayPickedMsg(d) => Msg::DayChange(d)
                    },
                    gtk::Spinner {
                        property_active: self.model.events.is_none()
                    }
                },
                #[name="info_bar"]
                gtk::InfoBar {
                    revealed: self.model.events.as_ref()
                                               .filter(|r| r.is_err())
                                               .is_some(),
                    message_type: gtk::MessageType::Error,
                },
                gtk::Box {
                    orientation: gtk::Orientation::Horizontal,
                    child: {
                        fill: true,
                        expand: true,
                    },
                    gtk::ScrolledWindow {
                        halign: gtk::Align::Start,
                        property_width_request: 350,
                        gtk::Box {
                            #[name="event_list"]
                            gtk::ListBox {
                                child: {
                                    fill: true,
                                    expand: true,
                                }
                            }
                        }
                    },
                    gtk::Box {
                        orientation: gtk::Orientation::Vertical,
                        valign: gtk::Align::Start,
                        child: {
                            fill: true,
                            expand: true,
                        },
                        #[name="header_label"]
                        gtk::Label {
                            child: {
                                padding: 10,
                                fill: true,
                                expand: true,
                            },
                            halign: gtk::Align::Start,
                            valign: gtk::Align::Start,
                            line_wrap: true,
                            selectable: true,
                            markup: self.model
                                        .current_event
                                        .as_ref()
                                        .map(|e| e.event_contents_header.as_str())
                                        .unwrap_or("No current event")
                        },
                        gtk::ScrolledWindow {
                            child: {
                                expand: true,
                            },
                            propagate_natural_height: true,
                            gtk::Box {
                                // two labels: one in case we have markup, one in case we have plain text.
                                // I used to have a single label for both,  using use_markup and text, and it worked,
                                // but there was no guarantee on the other in which both fields were updated. If the text
                                // was updated before 'use_markup', i could get text interpreted as markup which was not markup,
                                // then GtkLabel would fail and never recover displaying markup.
                                gtk::Label {
                                    // text label, not used when we display markup
                                    child: {
                                        fill: true,
                                        expand: true,
                                        padding: 10,
                                    },
                                    halign: gtk::Align::Start,
                                    valign: gtk::Align::Start,
                                    selectable: true,
                                    visible: self.model.current_event.as_ref()
                                                .filter(|e| e.event_contents_body.is_markup())
                                                .is_none(),
                                    text: self.model
                                                .current_event
                                                .as_ref()
                                                 .filter(|e| !e.event_contents_body.is_markup())
                                                .map(|e| e.event_contents_body.as_str())
                                                .unwrap_or(""),
                                },
                                gtk::Label {
                                    // markup label, not used when we display text
                                    child: {
                                        fill: true,
                                        expand: true,
                                        padding: 10,
                                    },
                                    halign: gtk::Align::Start,
                                    valign: gtk::Align::Start,
                                    selectable: true,
                                    visible: self.model.current_event.as_ref()
                                                .filter(|e| e.event_contents_body.is_markup())
                                                .is_some(),
                                    markup: self.model.current_event.as_ref()
                                                .filter(|e| e.event_contents_body.is_markup())
                                                .map(|e| e.event_contents_body.as_str())
                                                .unwrap_or(""),
                                }
                            }
                        }
                    }
                },
            },
            // Use a tuple when you want to both send a message and return a value to
            // the GTK+ callback.
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}
