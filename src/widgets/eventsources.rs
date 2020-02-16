use super::eventsource::{EventSourceListItem, EventSourceListItemModel};
use crate::config::Config;
use gtk::prelude::*;
use relm::ContainerWidget;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    ConfigUpdate(Config),
}

pub struct Model {
    config: Config,
    relm: relm::Relm<EventSources>,
}

#[widget]
impl Widget for EventSources {
    fn init_view(&mut self) {
        self.eventsources_list
            .get_style_context()
            .add_class("item_list");
        self.update_eventsources();
    }

    fn model(relm: &relm::Relm<Self>, config: Config) -> Model {
        Model {
            config: config,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::ConfigUpdate(cfg) => {
                self.model.config = cfg;
                self.update_eventsources();
            }
        }
    }

    fn update_eventsources(&mut self) {
        for child in self.eventsources_list.get_children() {
            self.eventsources_list.remove(&child);
        }
        let event_providers = crate::events::events::get_event_providers();
        for event_provider in event_providers {
            for event_config_name in event_provider.get_config_names(&self.model.config) {
                let event_config =
                    event_provider.get_config_values(&self.model.config, event_config_name);
                let _child = self.eventsources_list.add_widget::<EventSourceListItem>(
                    EventSourceListItemModel {
                        event_provider_name: event_provider.name(),
                        event_provider_icon: event_provider.default_icon(),
                        config_name: event_config_name.to_string(),
                        event_source: event_config.clone(),
                    },
                );
            }
        }
        for child in self.eventsources_list.get_children() {
            // don't want the row background color to change when we hover
            // it with the mouse (activatable), or the focus dotted lines
            // around the rows to be drawn, for aesthetic reasons.
            let row = child.dynamic_cast::<gtk::ListBoxRow>().unwrap();
            row.set_activatable(false);
            row.set_can_focus(false);
        }
    }

    view! {
       gtk::Box {
           orientation: gtk::Orientation::Vertical,
           #[name="eventsources_list"]
           gtk::ListBox {
               selection_mode: gtk::SelectionMode::None,
               child: {
                   fill: true,
                   expand: true,
               }
           }
       }
    }
}
