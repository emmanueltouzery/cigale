use super::datepicker::DatePickerMsg::DayPicked as DatePickerDayPickedMsg;
use super::datepicker::*;
use super::event::EventListItem;
use crate::events::events::Event;
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
        self.update_events();

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
            events: crate::events::events::get_all_events(&Local::today()),
            current_event: None,
        }
    }

    fn update_events(&self) {
        for child in self.event_list.get_children() {
            self.event_list.remove(&child);
        }
        for event in &self.model.events {
            let _child = self.event_list.add_widget::<EventListItem>(event.clone());
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::EventSelected => {
                let selected_index_maybe = self
                    .event_list
                    .get_selected_row()
                    .map(|r| r.get_index() as usize);
                self.model.current_event = selected_index_maybe
                    .and_then(|idx| self.model.events.get(idx))
                    .map(|evt| evt.clone());
            }
            Msg::DayChange(day) => {
                self.model.events = crate::events::events::get_all_events(&day);
                self.update_events();
            }
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
