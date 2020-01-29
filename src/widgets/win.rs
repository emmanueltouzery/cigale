use super::datepicker::DatePickerMsg::DayPicked as DatePickerDayPickedMsg;
use super::datepicker::*;
use super::event::EventListItem;
use crate::events::events::{Event, EventType};
use chrono::prelude::*;
use gtk::prelude::*;
use relm::{ContainerWidget, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    Quit,
    EventSelected,
    DayChange(Date<Local>),
}

pub struct Model {
    relm: relm::Relm<Win>,
    events: Vec<Event>,
    current_event: Option<Event>,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        for event in &self.model.events {
            let _child = self.event_list.add_widget::<EventListItem>(event.clone());
        }

        relm::connect!(
            self.model.relm,
            self.event_list,
            connect_row_selected(_, _),
            Msg::EventSelected
        );
    }

    fn model(relm: &relm::Relm<Self>, _: ()) -> Model {
        Model {
            relm: relm.clone(),
            events: vec![
                Event::new(
                    EventType::Git,
                    "12:56".to_string(),
                    "Emmanuel Touzery, Jane Doe".to_string(),
                    "Commit message <b>details</b>".to_string(),
                    Some("42 messages, lasted 2:30".to_string()),
                ),
                Event::new(
                    EventType::Email,
                    "13:42".to_string(),
                    "important email".to_string(),
                    "Hello John, Goodbye John".to_string(),
                    Some("to: John Doe (john@example.com)".to_string()),
                ),
            ],
            current_event: None,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::EventSelected => {
                self.model.current_event = Some(
                    self.model
                        .events
                        .get(self.event_list.get_selected_row().unwrap().get_index() as usize)
                        .unwrap()
                        .clone(),
                )
            }
            Msg::DayChange(day) => println!("Day change {}", day.format("%A, %Y-%m-%d")),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                DatePicker {
                    DatePickerDayPickedMsg(d) => Msg::DayChange(d)
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
