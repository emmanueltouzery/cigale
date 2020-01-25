extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;

use gtk::{Application, ApplicationWindow, Box, Button, Calendar, Popover, TextView};

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
            println!("Clicked!");
        });

        vbox.add(&button);

        let textview = TextView::new();
        vbox.add(&textview);
        window.add(&vbox);

        window.show_all();
    });

    application.run(&[]);
}
