use gtk::prelude::*;
use relm::Widget;
use relm_derive::{widget, Msg};
use std::collections::HashMap;

#[derive(Msg)]
pub enum EventSourceListItemMsg {
    ActionsClicked,
}

pub struct EventSourceListItemInfo {
    pub event_provider_icon: &'static str,
    pub event_provider_name: &'static str,
    pub config_name: String,
    pub event_source: HashMap<&'static str, String>,
    pub eventsource_action_popover: gtk::Popover,
}

pub struct Model {
    relm: relm::Relm<EventSourceListItem>,
    list_item_info: EventSourceListItemInfo,
}

#[widget]
impl Widget for EventSourceListItem {
    fn init_view(&mut self) {
        let mut i = 1;
        for kv in &self.model.list_item_info.event_source {
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
        self.items_frame
            .get_style_context()
            .add_class("items_frame");
    }

    fn model(relm: &relm::Relm<Self>, list_item_info: EventSourceListItemInfo) -> Model {
        Model {
            relm: relm.clone(),
            list_item_info,
        }
    }

    fn update(&mut self, event: EventSourceListItemMsg) {
        println!("event");
        match event {
            EventSourceListItemMsg::ActionsClicked => {
                println!("show popover");
                let popover = &self.model.list_item_info.eventsource_action_popover;
                for child in popover.get_children() {
                    popover.remove(&child);
                }
                popover.set_relative_to(Some(&self.event_source_actions_btn));
                let vbox = gtk::BoxBuilder::new()
                    .orientation(gtk::Orientation::Vertical)
                    .build();
                vbox.add(&gtk::ModelButtonBuilder::new().label("Remove").build());
                popover.add(&vbox);
                popover.popup();
            }
        }
    }

    view! {
        #[name="items_frame"]
        gtk::Frame {
            margin_start: 20,
            margin_end: 20,
            margin_top: 20,
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
                gtk::Box {
                    orientation: gtk::Orientation::Horizontal,
                    cell: {
                        width: 2
                    },
                    gtk::Image {
                        from_pixbuf: Some(&crate::icons::fontawesome_image(
                            self.model.list_item_info.event_provider_icon, 16)),
                    },
                    gtk::Label {
                        margin_start: 5,
                        text: (self.model.list_item_info.event_provider_name.to_string()
                               + " - " + &self.model.list_item_info.config_name).as_str(),
                        xalign: 0.0,
                    }
                },
                #[name="event_source_actions_btn"]
                gtk::Button {
                    always_show_image: true,
                    image: Some(&gtk::Image::new_from_pixbuf(
                        Some(&crate::icons::fontawesome_image("cog", 12)))), // emblem-system-symbolic
                    hexpand: true,
                    halign: gtk::Align::End,
                    cell: {
                        left_attach: 2,
                        top_attach: 0,
                    },
                    clicked => EventSourceListItemMsg::ActionsClicked
                    // button_release_event(_, _) => (EventSourceListItemMsg::ActionsClicked, Inhibit(false)),
                }
            }
        }
    }
}
