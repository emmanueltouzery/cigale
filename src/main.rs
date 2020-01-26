extern crate gdk;
extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;

const FONT_AWESOME_SVGS_ROOT: &str = "fontawesome-free-5.12.0-desktop/svgs/solid";

use gtk::{
    Application, ApplicationWindow, Box, Button, Calendar, Image, Label, Popover, ScrolledWindow,
};

fn main() {
    let application =
        Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
            .expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("First GTK+ Program");
        window.set_default_size(800, 600);

        let vbox = Box::new(gtk::Orientation::Vertical, 0);

        let button = Button::new_with_label("Click me!");
        button.show();

        let cal = Calendar::new();
        cal.show();

        let cal_popover = Popover::new(Some(&button));
        cal_popover.add(&cal);

        button.connect_clicked(move |_| {
            cal_popover.popup();
        });

        vbox.add(&button);

        let event_box = Box::new(gtk::Orientation::Horizontal, 0);
        let event_list = event_list(&vec![Event::new(
            EventType::Git,
            "12:56".to_string(),
            "Emmanuel Touzery, Jane Doe".to_string(),
            Some("42 messages, lasted 2:30".to_string()),
        )]);
        event_box.pack_start(&event_list, false, true, 0);
        event_box.pack_start(&gtk::TextView::new(), true, true, 0);
        vbox.pack_start(&event_box, true, true, 0);

        window.add(&vbox);
        window.show_all();
    });

    application.run(&[]);
}

trait EventTypeTrait {
    fn get_desc(&self) -> &str;
    fn get_icon(&self) -> &str;
}

enum EventType {
    Git,
}

impl EventTypeTrait for EventType {
    fn get_desc(&self) -> &str {
        match self {
            EventType::Git => "Git",
        }
    }

    fn get_icon(&self) -> &str {
        match self {
            EventType::Git => "code-branch",
        }
    }
}

struct Event {
    event_type: EventType,
    event_time: String,
    event_info: String,
    event_extra_details: Option<String>,
}

impl Event {
    fn new(
        event_type: EventType,
        event_time: String,
        event_info: String,
        event_extra_details: Option<String>,
    ) -> Event {
        Event {
            event_type,
            event_time,
            event_info,
            event_extra_details,
        }
    }
}

// TODO load the icons i'm interested in only once, put them
// in the binary
fn fontawesome_image(image_name: &str) -> Image {
    Image::new_from_pixbuf(Some(
        &gdk_pixbuf::Pixbuf::new_from_file_at_size(
            format!(
                "/home/emmanuel/home/cigale/{}/{}.svg",
                FONT_AWESOME_SVGS_ROOT, image_name
            ),
            40,
            40,
        )
        .unwrap(),
    ))
}

fn single_event(event: &Event) -> gtk::Box {
    let hbox = Box::new(gtk::Orientation::Horizontal, 10);

    let vbox_eventtype = Box::new(gtk::Orientation::Vertical, 2);
    vbox_eventtype.add(&fontawesome_image(event.event_type.get_icon()));
    vbox_eventtype.add(&Label::new(Some(event.event_type.get_desc())));
    hbox.add(&vbox_eventtype);

    let vbox_eventdetails = Box::new(gtk::Orientation::Vertical, 2);
    let vbox_eventtime_and_extra_info = Box::new(gtk::Orientation::Horizontal, 2);
    let time_label = Label::new(Some(&format!("<b>{}</b>", event.event_time)));
    time_label.set_use_markup(true);
    time_label.set_halign(gtk::Align::Start);
    vbox_eventtime_and_extra_info.pack_start(&time_label, true, true, 0);
    let extra_details_label = Label::new(event.event_extra_details.as_ref().map(|t| t.as_str()));
    extra_details_label.set_halign(gtk::Align::Start);
    vbox_eventtime_and_extra_info.pack_end(&extra_details_label, false, false, 0);
    vbox_eventdetails.pack_start(&vbox_eventtime_and_extra_info, true, true, 5);
    let details_label = Label::new(Some(&event.event_info));
    details_label.set_halign(gtk::Align::Start);
    vbox_eventdetails.pack_start(&details_label, true, false, 5);
    hbox.pack_start(&vbox_eventdetails, true, true, 5);

    hbox.set_margin_start(10);
    hbox.set_margin_end(10);
    hbox.set_margin_top(10);
    hbox.set_margin_bottom(10);

    hbox
}

fn event_list(events: &Vec<Event>) -> gtk::ScrolledWindow {
    let vbox = Box::new(gtk::Orientation::Vertical, 0);
    for event in events {
        vbox.add(&single_event(event));
    }
    let scrolled_win = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scrolled_win.add(&vbox);
    scrolled_win.set_size_request(320, -1);
    scrolled_win
}