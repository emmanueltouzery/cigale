use chrono::prelude::*;
use gtk::prelude::*;
use gtk::{ImageExt, Inhibit};
use relm::{ContainerWidget, Widget};
use relm_derive::{widget, Msg};
use DatePickerMsg::DayPicked as PickerDayPickedMsg;

const FONT_AWESOME_SVGS_ROOT: &str = "fontawesome-free-5.12.0-desktop/svgs/solid";

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
        self.event_pane.set_size_request(350, -1);
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
                    "Commit message details".to_string(),
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
                    PickerDayPickedMsg(d) => Msg::DayChange(d)
                },
                gtk::Box {
                    orientation: gtk::Orientation::Horizontal,
                    child: {
                        fill: true,
                        expand: true,
                    },
                    #[name="event_pane"]
                    gtk::ScrolledWindow {
                        halign: gtk::Align::Start,
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
                        },
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

/// EventListItem

#[derive(Msg)]
pub enum EventListItemMsg {}

pub struct EventListItemModel {
    event: Event,
}

#[widget]
impl Widget for EventListItem {
    fn model(event: Event) -> EventListItemModel {
        EventListItemModel { event }
    }

    fn update(&mut self, _event: EventListItemMsg) {}

    view! {
        gtk::EventBox {
            gtk::Box {
                orientation: gtk::Orientation::Horizontal,
                margin_start: 10,
                margin_end: 10,
                margin_top: 10,
                margin_bottom: 10,
                gtk::Box {
                    orientation: gtk::Orientation::Vertical,
                    child: {
                        padding: 2,
                    },
                    gtk::Image {
                        from_pixbuf: Some(&fontawesome_image("code-branch", 40))
                    },
                    gtk::Label {
                        text: self.model.event.event_type.get_desc()
                    },
                },
                gtk::Box {
                    orientation: gtk::Orientation::Vertical,
                    child: {
                        padding: 2,
                        pack_type: gtk::PackType::Start,
                        expand: true,
                        fill: true,
                    },
                    gtk::Box {
                        orientation: gtk::Orientation::Horizontal,
                        child: {
                            pack_type: gtk::PackType::Start,
                            expand: true,
                            fill: true,
                            padding: 5,
                        },
                        gtk::Label {
                            child: {
                                pack_type: gtk::PackType::Start,
                            },
                            // text: format!("<b>{}</b>", event.event_time) // doesn't compile
                            label: ("<b>".to_string() + &self.model.event.event_time + "</b>").as_str(),
                            use_markup: true,
                            // text: self.model.event.event_time.as_str(),
                            halign: gtk::Align::Start
                        },
                        gtk::Label {
                            child: {
                                pack_type: gtk::PackType::End,
                            },
                            text: self.model.event.event_extra_details.as_ref().unwrap().as_str(),
                            halign: gtk::Align::Start
                        },
                    },
                    gtk::Label {
                        child: {
                            expand: true,
                            fill: true,
                            padding: 5
                        },
                        text: self.model.event.event_info.as_str(),
                        halign: gtk::Align::Start
                    }
                }
            }
        }
    }
}

/// DatePicker

#[derive(Msg)]
pub enum DatePickerMsg {
    ButtonClicked,
    DayClicked,
    DayPicked(Date<Local>),
}

pub struct DatePickerModel {
    relm: relm::Relm<DatePicker>,
    calendar_popover: gtk::Popover,
    calendar: gtk::Calendar,
    date: Date<Local>,
}

#[widget]
impl Widget for DatePicker {
    fn init_view(&mut self) {
        self.model
            .calendar_popover
            .set_relative_to(Some(&self.calendar_button));
        self.model.calendar_popover.hide();
        self.model.calendar_popover.add(&self.model.calendar);
        self.model.calendar.show();
        relm::connect!(
            self.model.relm,
            self.model.calendar,
            connect_day_selected(_),
            DatePickerMsg::DayClicked
        );
    }
    fn model(relm: &relm::Relm<Self>, _: ()) -> DatePickerModel {
        DatePickerModel {
            relm: relm.clone(),
            calendar_popover: gtk::Popover::new(None::<&gtk::Button>),
            calendar: gtk::Calendar::new(),
            date: Local::today(),
        }
    }

    fn update(&mut self, event: DatePickerMsg) {
        match event {
            DatePickerMsg::ButtonClicked => {
                if self.model.calendar_popover.is_visible() {
                    self.model.calendar_popover.popdown()
                } else {
                    self.model.calendar_popover.popup()
                }
            }
            DatePickerMsg::DayClicked => {
                let (y, m, d) = self.model.calendar.get_date();
                self.model
                    .relm
                    .stream()
                    .emit(DatePickerMsg::DayPicked(Local.ymd(y as i32, m + 1, d)))
            }
            DatePickerMsg::DayPicked(d) => {
                self.model.date = d;
                self.model.calendar_popover.popdown();
            }
        }
    }

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Horizontal,
            margin_start: 10,
            margin_end: 10,
            margin_top: 10,
            margin_bottom: 10,
            gtk::Label {
                child: {
                    padding: 2,
                },
                label: "Day to display:"
            },
            #[name="calendar_button"]
            gtk::Button {
                child: {
                    padding: 2,
                },
                always_show_image: true,
                image: Some(&gtk::Image::new_from_pixbuf(Some(&fontawesome_image("calendar-alt", 16)))),
                label: self.model.date.format("%A, %Y-%m-%d").to_string().as_str(),
                clicked => DatePickerMsg::ButtonClicked
            },
        }
    }
}

trait EventTypeTrait {
    fn get_desc(&self) -> &str;
    fn get_icon(&self) -> &str;
}

#[derive(Clone, Copy)]
enum EventType {
    Git,
    Email,
}

impl EventTypeTrait for EventType {
    fn get_desc(&self) -> &str {
        match self {
            EventType::Git => "Git",
            EventType::Email => "Email",
        }
    }

    fn get_icon(&self) -> &str {
        match self {
            EventType::Git => "code-branch",
            EventType::Email => "envelope",
        }
    }
}

#[derive(Clone)]
pub struct Event {
    event_type: EventType,
    event_time: String,
    event_info: String,
    event_contents: String,
    event_extra_details: Option<String>,
}

impl Event {
    fn new(
        event_type: EventType,
        event_time: String,
        event_info: String,
        event_contents: String,
        event_extra_details: Option<String>,
    ) -> Event {
        Event {
            event_type,
            event_time,
            event_info,
            event_contents,
            event_extra_details,
        }
    }
}

// TODO load the icons i'm interested in only once, put them
// in the binary
fn fontawesome_image(image_name: &str, size: i32) -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_file_at_size(
        format!(
            "/home/emmanuel/home/cigale/{}/{}.svg",
            FONT_AWESOME_SVGS_ROOT, image_name
        ),
        size,
        size,
    )
    .unwrap()
}

fn main() {
    Win::run(()).unwrap();
}
