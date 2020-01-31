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
                    gtk::Label {
                        child: {
                            fill: true,
                            expand: true,
                            padding: 10,
                        },
                        halign: gtk::Align::Start,
                        valign: gtk::Align::Start,
                        markup: self.model
                            .current_event
                            .as_ref()
                            .map(|e| e.event_contents.as_str())
                            .unwrap_or("No current event")
                    }
                },
            },
            // Use a tuple when you want to both send a message and return a value to
            // the GTK+ callback.
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}
