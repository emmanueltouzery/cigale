use chrono::prelude::*;
use gtk::prelude::*;
use relm::Widget;
use relm_derive::{widget, Msg};

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
                image: Some(&gtk::Image::new_from_pixbuf(Some(&crate::icons::fontawesome_image("calendar-alt", 16)))),
                label: self.model.date.format("%A, %Y-%m-%d").to_string().as_str(),
                clicked => DatePickerMsg::ButtonClicked
            },
        }
    }
}
