use crate::icons::*;
use chrono::prelude::*;
use gtk::prelude::*;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum DatePickerMsg {
    ButtonClicked,
    DayClicked,
    MonthChanged,
    NextDay,
    PreviousDay,
    DayPicked(Date<Local>),
}

pub struct DatePickerModel {
    relm: relm::Relm<DatePicker>,
    accel_group: gtk::AccelGroup,
    calendar_popover: gtk::Popover,
    calendar: gtk::Calendar,
    date: Date<Local>,
    // when the user changes month, the calendar
    // will first emit a month-changed event, then
    // a day-selected event.
    // By default the latter would close the popover.
    // So when we get a month change event, set up
    // a marker to ignore the upcoming day-selected.
    //
    // we want to close the popover only when the
    // user clicks on a specific day.
    month_change_ongoing: bool,
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
        relm::connect!(
            self.model.relm,
            self.model.calendar,
            connect_month_changed(_),
            DatePickerMsg::MonthChanged
        );
        // https://askubuntu.com/a/138520/188440
        self.prev_button.add_accelerator(
            "activate",
            &self.model.accel_group,
            65361, //arrow left
            gdk::ModifierType::MOD1_MASK,
            gtk::AccelFlags::VISIBLE,
        );
        self.next_button.add_accelerator(
            "activate",
            &self.model.accel_group,
            65363, //arrow right
            gdk::ModifierType::MOD1_MASK,
            gtk::AccelFlags::VISIBLE,
        )
    }
    fn model(relm: &relm::Relm<Self>, accel_group: gtk::AccelGroup) -> DatePickerModel {
        let date = Local::today().pred();
        let cal = gtk::Calendar::new();
        Self::calendar_set_date(&cal, date);
        DatePickerModel {
            relm: relm.clone(),
            accel_group,
            calendar_popover: gtk::Popover::new(None::<&gtk::Button>),
            calendar: cal,
            date,
            month_change_ongoing: false,
        }
    }

    fn calendar_set_date(cal: &gtk::Calendar, date: Date<Local>) {
        cal.set_property_year(date.year());
        cal.set_property_month(date.month() as i32 - 1);
        cal.set_property_day(date.day() as i32);
    }

    fn update(&mut self, event: DatePickerMsg) {
        match event {
            DatePickerMsg::ButtonClicked => {
                if self.model.calendar_popover.is_visible() {
                    self.model.calendar_popover.popdown()
                } else {
                    // the date held by the calendar will be outdated
                    // if the user's been navigating with previous/next
                    Self::calendar_set_date(&self.model.calendar, self.model.date);
                    self.model.calendar_popover.popup()
                }
            }
            DatePickerMsg::DayClicked => {
                let (y, m, d) = self.model.calendar.get_date();
                // the if is useful for instance because we update the calendar when
                // opening it (it could be outdated due to previous/next navigation)
                // without the if that would trigger a reload of the current day
                let clicked_date = Local.ymd(y as i32, m + 1, d);
                if self.model.date != clicked_date {
                    self.model
                        .relm
                        .stream()
                        .emit(DatePickerMsg::DayPicked(clicked_date))
                }
            }
            DatePickerMsg::DayPicked(d) => {
                self.model.date = d;
                if self.model.month_change_ongoing {
                    self.model.month_change_ongoing = false;
                } else {
                    self.model.calendar_popover.popdown();
                }
            }
            DatePickerMsg::MonthChanged => {
                // getting false positives, because this is called even if the month
                // was changed by API call to the same value as before...
                let (_y, m, _d) = self.model.calendar.get_date();
                self.model.month_change_ongoing = m + 1 != self.model.date.month();
            }
            DatePickerMsg::NextDay => self
                .model
                .relm
                .stream()
                .emit(DatePickerMsg::DayPicked(self.model.date.succ())),
            DatePickerMsg::PreviousDay => self
                .model
                .relm
                .stream()
                .emit(DatePickerMsg::DayPicked(self.model.date.pred())),
        }
    }

    view! {
        #[name="picker_box"]
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
            #[name="prev_button"]
            gtk::Button {
                always_show_image: true,
                image: Some(&gtk::Image::new_from_icon_name(
                    Some(Icon::ANGLE_LEFT.name()), gtk::IconSize::Menu)),
                valign: gtk::Align::Center,
                relief: gtk::ReliefStyle::None,
                clicked => DatePickerMsg::PreviousDay
            },
            #[name="calendar_button"]
            gtk::Button {
                child: {
                    padding: 2,
                },
                always_show_image: true,
                image: Some(&gtk::Image::new_from_icon_name(
                    Some(Icon::CALENDAR_ALT.name()), gtk::IconSize::Menu)),
                label: self.model.date.format("%A, %Y-%m-%d").to_string().as_str(),
                clicked => DatePickerMsg::ButtonClicked
            },
            #[name="next_button"]
            gtk::Button {
                always_show_image: true,
                image: Some(&gtk::Image::new_from_icon_name(
                    Some(Icon::ANGLE_RIGHT.name()), gtk::IconSize::Menu)),
                valign: gtk::Align::Center,
                relief: gtk::ReliefStyle::None,
                clicked => DatePickerMsg::NextDay
            },
        }
    }
}
