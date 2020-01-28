use gdk;
use gtk::prelude::*;
use gtk::{ImageExt, Inhibit};
use relm::{ContainerWidget, Widget};
use relm_derive::{widget, Msg};
use EventListItemMsg::EventSelected as EventListItemEventSelected;

const FONT_AWESOME_SVGS_ROOT: &str = "fontawesome-free-5.12.0-desktop/svgs/solid";

#[derive(Msg)]
pub enum Msg {
    Decrement,
    Increment,
    Quit,
    EventSelected(Event),
    EventSelected2,
}

pub struct Model {
    relm: relm::Relm<Win>,
    counter: u32,
    events: Vec<Event>,
    current_event: Option<Event>,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        self.event_pane.set_size_request(350, -1);
        for event in &self.model.events {
            let child = self.event_list.add_widget::<EventListItem>(event.clone());

            relm::connect!(child@EventListItemEventSelected(ref event),
                           self.model.relm, Msg::EventSelected(event.clone()));
        }

        relm::connect!(
            self.model.relm,
            self.event_list,
            connect_row_selected(_, _),
            Msg::EventSelected2
        );
    }

    fn model(relm: &relm::Relm<Self>, _: ()) -> Model {
        Model {
            relm: relm.clone(),
            counter: 0,
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
            // A call to self.label1.set_text() is automatically inserted by the
            // attribute every time the model.counter attribute is updated.
            Msg::Decrement => self.model.counter -= 1,
            Msg::Increment => self.model.counter += 1,
            Msg::Quit => gtk::main_quit(),
            Msg::EventSelected(ref event) => self.model.current_event = Some(event.clone()),
            Msg::EventSelected2 => self.model.relm.stream().emit(Msg::EventSelected(
                self.model
                    .events
                    .get(self.event_list.get_selected_row().unwrap().get_index() as usize)
                    .unwrap()
                    .clone(),
            )),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                gtk::Button {
                    // By default, an event with one paramater is assumed.
                    clicked => Msg::Increment,
                    // Hence, the previous line is equivalent to:
                    // clicked(_) => Increment,
                    label: "+",
                },
                gtk::Label {
                    // Bind the text property of this Label to the counter attribute
                    // of the model.
                    // Every time the counter attribute is updated, the text property
                    // will be updated too.
                    text: &self.model.counter.to_string(),
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
                gtk::Button {
                    clicked => Msg::Decrement,
                    label: "-",
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
pub enum EventListItemMsg {
    Click,
    EventSelected(Event),
}

pub struct EventListItemModel {
    relm: relm::Relm<EventListItem>,
    event: Event,
}

#[widget]
impl Widget for EventListItem {
    fn model(relm: &relm::Relm<Self>, event: Event) -> EventListItemModel {
        EventListItemModel {
            relm: relm.clone(),
            event,
        }
    }

    fn update(&mut self, event: EventListItemMsg) {
        match event {
            EventListItemMsg::Click => self
                .model
                .relm
                .stream()
                .emit(EventListItemMsg::EventSelected(self.model.event.clone())),
            _ => (),
        }
    }

    view! {
        gtk::EventBox {
            button_press_event(_, event) => ({
                if event.get_event_type() == gdk::EventType::DoubleButtonPress {
                    EventListItemMsg::Click
                }
                else {
                    EventListItemMsg::Click
                }
            }, Inhibit(false)),
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
                        from_pixbuf: Some(&fontawesome_image("code-branch"))
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

// fn main() {
//     let application =
//         Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
//             .expect("failed to initialize GTK application");

//     application.connect_activate(|app| {
//         let window = ApplicationWindow::new(app);
//         window.set_title("First GTK+ Program");
//         window.set_default_size(800, 600);

//         let vbox = Box::new(gtk::Orientation::Vertical, 0);

//         let button = Button::new_with_label("Click me!");
//         button.show();

//         let cal = Calendar::new();
//         cal.show();

//         let cal_popover = Popover::new(Some(&button));
//         cal_popover.add(&cal);

//         button.connect_clicked(move |_| {
//             cal_popover.popup();
//         });

//         vbox.add(&button);

//         let event_box = Box::new(gtk::Orientation::Horizontal, 0);
//         let events = vec![
//             Rc::new(Event::new(
//                 EventType::Git,
//                 "12:56".to_string(),
//                 "Emmanuel Touzery, Jane Doe".to_string(),
//                 "Commit message details".to_string(),
//                 Some("42 messages, lasted 2:30".to_string()),
//             )),
//             Rc::new(Event::new(
//                 EventType::Email,
//                 "13:42".to_string(),
//                 "important email".to_string(),
//                 "Hello John, Goodbye John".to_string(),
//                 Some("to: John Doe (john@example.com)".to_string()),
//             )),
//         ];
//         let text_view = gtk::TextView::new();
//         let text_buf = text_view.get_buffer().unwrap();
//         let cb = Rc::new(move |evt: &Rc<Event>| text_buf.set_text(evt.event_contents.as_str()));
//         let event_list = event_list(&events, cb);
//         event_box.pack_start(&event_list, false, true, 0);
//         event_box.pack_start(&text_view, true, true, 0);
//         vbox.pack_start(&event_box, true, true, 0);

//         window.add(&vbox);
//         window.show_all();
//     });

//     application.run(&[]);
// }

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
fn fontawesome_image(image_name: &str) -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_file_at_size(
        format!(
            "/home/emmanuel/home/cigale/{}/{}.svg",
            FONT_AWESOME_SVGS_ROOT, image_name
        ),
        40,
        40,
    )
    .unwrap()
}

fn main() {
    Win::run(()).unwrap();
}

// fn single_event<F: 'static + Fn(&Rc<Event>)>(event: &Rc<Event>, clicked: Rc<F>) -> gtk::EventBox {
//     let hbox = Box::new(gtk::Orientation::Horizontal, 10);

//     let vbox_eventtype = Box::new(gtk::Orientation::Vertical, 2);
//     vbox_eventtype.add(&fontawesome_image(event.event_type.get_icon()));
//     vbox_eventtype.add(&Label::new(Some(event.event_type.get_desc())));
//     hbox.add(&vbox_eventtype);

//     let vbox_eventdetails = Box::new(gtk::Orientation::Vertical, 2);
//     let vbox_eventtime_and_extra_info = Box::new(gtk::Orientation::Horizontal, 2);
//     let time_label = Label::new(Some(&format!("<b>{}</b>", event.event_time)));
//     time_label.set_use_markup(true);
//     time_label.set_halign(gtk::Align::Start);
//     vbox_eventtime_and_extra_info.pack_start(&time_label, true, true, 0);
//     let extra_details_label = Label::new(event.event_extra_details.as_ref().map(|t| t.as_str()));
//     extra_details_label.set_halign(gtk::Align::Start);
//     vbox_eventtime_and_extra_info.pack_end(&extra_details_label, false, false, 0);
//     vbox_eventdetails.pack_start(&vbox_eventtime_and_extra_info, true, true, 5);
//     let details_label = Label::new(Some(&event.event_info));
//     details_label.set_halign(gtk::Align::Start);
//     vbox_eventdetails.pack_start(&details_label, true, false, 5);
//     hbox.pack_start(&vbox_eventdetails, true, true, 5);

//     hbox.set_margin_start(10);
//     hbox.set_margin_end(10);
//     hbox.set_margin_top(10);
//     hbox.set_margin_bottom(10);

//     let event_box = gtk::EventBox::new();
//     event_box.add(&hbox);

//     event_box
//         .connect_local(
//             // i don't understand the diff between connect and connect_local!!
//             // but connect wants some multithread things that connect_local doesn't require
//             "button-press-event",
//             false,
//             glib::clone!(@strong event => move |_| {
//                 clicked(&event);
//                 Some(true.to_value())
//             }),
//         )
//         .unwrap();

//     event_box
// }

// fn event_list<F: 'static + Fn(&Rc<Event>)>(
//     events: &Vec<Rc<Event>>,
//     event_selected: Rc<F>,
// ) -> gtk::ScrolledWindow {
//     let vbox = Box::new(gtk::Orientation::Vertical, 0);
//     for event in events {
//         vbox.add(&single_event(event, event_selected.clone()));
//     }
//     let scrolled_win = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
//     scrolled_win.add(&vbox);
//     scrolled_win.set_size_request(350, -1);
//     scrolled_win
// }
