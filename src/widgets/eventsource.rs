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
        let mut i = 1;
        for kv in &self.model.event_source {
            let desc = gtk::LabelBuilder::new().label(kv.0).xalign(0.0).build();
            desc.get_style_context()
                .add_class("event_source_config_label");
            self.items_box.attach(&desc, 0, i, 1, 1);
            self.items_box.attach(
                &gtk::LabelBuilder::new()
                    .label(kv.1)
                    .ellipsize(pango::EllipsizeMode::End)
                    .xalign(0.0)
                    .build(),
                1,
                i,
                1,
                1,
            );
            i += 1;
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
        gtk::Grid {
            orientation: gtk::Orientation::Vertical,
            margin_start: 10,
            margin_end: 10,
            margin_top: 10,
            margin_bottom: 5,
            row_spacing: 5,
            column_spacing: 10,
            #[name="event_source_name"]
            gtk::Label {
                cell: {
                    width: 2
                },
                text: (self.model.event_provider_name.to_string() + " - " + &self.model.config_name).as_str(),
                xalign: 0.0,
            },
        }
    }
}
