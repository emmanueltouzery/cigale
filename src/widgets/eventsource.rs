use crate::events::events::{get_event_providers, ConfigType};
use crate::icons::*;
use gtk::builders::*;
use gtk::prelude::*;
use relm::Widget;
use relm_derive::{widget, Msg};
use std::collections::HashMap;

#[derive(Msg)]
pub enum EventSourceListItemMsg {
    ActionsClicked(gtk::Button),
}

pub struct EventSourceListItemInfo {
    pub event_provider_icon: Icon,
    pub event_provider_name: &'static str,
    pub config_name: String,
    pub event_source: HashMap<&'static str, String>,
}

pub struct Model {
    list_item_info: EventSourceListItemInfo,
}

#[widget]
impl Widget for EventSourceListItem {
    fn init_view(&mut self) {
        // remove the gtk-added 'image-button' CSS class.
        // i want a gray background instead of white:
        // consistent with the gnome printers dialog, and more visible in dark mode
        self.widgets
            .event_source_actions_btn
            .style_context()
            .remove_class("image-button");

        let ep = get_event_providers()
            .into_iter()
            .find(|ep| ep.name() == self.model.list_item_info.event_provider_name)
            .unwrap();
        let mut i = 1;
        for kv in &self.model.list_item_info.event_source {
            let field_type = ep
                .get_config_fields()
                .iter()
                .find(|(fname, _)| fname == kv.0)
                .unwrap()
                .1;
            let desc = LabelBuilder::new().label(kv.0).xalign(0.0).build();
            desc.style_context().add_class("event_source_config_label");
            self.widgets.items_box.attach(&desc, 0, i, 1, 1);
            self.widgets.items_box.attach(
                &LabelBuilder::new()
                    .label(if field_type == ConfigType::Password {
                        "●●●●●"
                    } else {
                        kv.1
                    })
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
        self.widgets.items_box.show_all();
    }

    fn model(list_item_info: EventSourceListItemInfo) -> Model {
        Model { list_item_info }
    }

    fn update(&mut self, event: EventSourceListItemMsg) {
        match event {
            EventSourceListItemMsg::ActionsClicked(_) => {
                // meant for my parent
            }
        }
    }

    view! {
        #[style_class="items_frame"]
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
                #[style_class="event_source_name"]
                gtk::Box {
                    orientation: gtk::Orientation::Horizontal,
                    cell: {
                        width: 2
                    },
                    gtk::Image {
                        icon_name: Some(
                            self.model.list_item_info.event_provider_icon.name()),
                        // https://github.com/gtk-rs/gtk/issues/837
                        icon_size: gtk::IconSize::Menu,
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
                    image: Some(&gtk::Image::from_icon_name(
                        Some(Icon::COG.name()), gtk::IconSize::Menu)),
                    hexpand: true,
                    halign: gtk::Align::End,
                    cell: {
                        left_attach: 2,
                        top_attach: 0,
                    },
                    button_release_event(c, _) =>
                        (EventSourceListItemMsg::ActionsClicked(c.clone()), Inhibit(false))
                }
            }
        }
    }
}
