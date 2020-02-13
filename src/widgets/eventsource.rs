use gtk::prelude::*;
use relm::Widget;
use relm_derive::{widget, Msg};
use std::collections::HashMap;

#[derive(Msg)]
pub enum EventSourceListItemMsg {}

pub struct EventSourceListItemModel {
    pub event_provider_name: &'static str,
    pub config_name: String,
    pub event_source: HashMap<&'static str, String>,
}

#[widget]
impl Widget for EventSourceListItem {
    fn init_view(&mut self) {
        for kv in &self.model.event_source {
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            hbox.add(&gtk::Label::new(Some(kv.0)));
            hbox.add(&gtk::Label::new(Some(kv.1)));
            self.items_box.add(&hbox);
        }
        self.items_box.show_all();
        self.event_source_name
            .get_style_context()
            .add_class("event_source_name");
    }

    fn model(model: EventSourceListItemModel) -> EventSourceListItemModel {
        model
    }

    fn update(&mut self, _event: EventSourceListItemMsg) {}

    view! {
        #[name="items_box"]
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            margin_start: 10,
            margin_end: 10,
            margin_top: 10,
            margin_bottom: 10,
            #[name="event_source_name"]
            gtk::Label {
                text: (self.model.event_provider_name.to_string() + " - " + &self.model.config_name).as_str(),
                xalign: 0.0,
            },
        }
    }
}
