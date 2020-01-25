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

        let event_list = event_list();
        vbox.pack_start(&event_list, true, true, 0);
        window.add(&vbox);

        window.show_all();
    });

    application.run(&[]);
}

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

fn event_list() -> gtk::ScrolledWindow {
    let vbox = Box::new(gtk::Orientation::Vertical, 0);
    vbox.add(&Label::new(Some("hello")));
    vbox.add(&fontawesome_image("code-branch"));
    vbox.add(&Label::new(Some("world")));
    vbox.add(&Label::new(Some("here")));
    vbox.add(&Label::new(Some("are")));
    vbox.add(&Label::new(Some("some")));
    vbox.add(&Label::new(Some("words")));
    let scrolled_win = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scrolled_win.add(&vbox);
    scrolled_win
}
