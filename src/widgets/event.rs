use crate::events::events::Event;
use gtk::prelude::*;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum EventListItemMsg {}

pub struct EventListItemModel {
    event: Event,
}

#[widget]
impl Widget for EventListItem {
    fn init_view(&mut self) {}

    fn model(event: Event) -> EventListItemModel {
        EventListItemModel { event }
    }

    fn update(&mut self, _event: EventListItemMsg) {}

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Horizontal,
            margin_start: 10,
            margin_end: 10,
            margin_top: 10,
            margin_bottom: 10,
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                valign: gtk::Align::Center,
                child: {
                    padding: 3,
                },
                gtk::Image {
                    icon_name: Some(self.model.event.event_type_icon.name()),
                    icon_size: gtk::IconSize::Dnd
                },
                #[style_class="event_provider_name"]
                gtk::Label {
                    text: self.model.event.event_type_desc,
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
                    },
                    #[name="event_time_label"]
                    #[style_class="event_time"]
                    gtk::Label {
                        child: {
                            pack_type: gtk::PackType::Start,
                            padding: 3,
                        },
                        // text: format!("<b>{}</b>", event.event_time) // doesn't compile
                        label: ("<b>".to_string() + &self.model.event.event_time.format("%H:%M").to_string() + "</b>").as_str(),
                        use_markup: true,
                        // text: self.model.event.event_time.as_str(),
                        halign: gtk::Align::Start
                    },
                    gtk::Label {
                        child: {
                            pack_type: gtk::PackType::End,
                            padding: 3,
                        },
                        text: self.model.event.event_extra_details.as_ref().unwrap_or(&"".to_string()).as_str(),
                        halign: gtk::Align::Start,
                        ellipsize: pango::EllipsizeMode::End
                    },
                },
                gtk::Label {
                    child: {
                        expand: true,
                        fill: true,
                        padding: 5
                    },
                    text: self.model.event.event_info.as_str(),
                    halign: gtk::Align::Start,
                    ellipsize: pango::EllipsizeMode::End
                }
            }
        }
    }
}
