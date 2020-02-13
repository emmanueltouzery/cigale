use super::eventsource::{EventSourceListItem, EventSourceListItemModel};
use crate::config::Config;
use gtk::prelude::*;
use relm::ContainerWidget;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {}

pub struct Model {
    config: Config,
    relm: relm::Relm<EventSources>,
}

#[widget]
impl Widget for EventSources {
    fn init_view(&mut self) {
        self.update_eventsources();
    }

    fn model(relm: &relm::Relm<Self>, config: Config) -> Model {
        Model {
            config: config,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, _event: Msg) {}

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
                        config_name: event_config_name.to_string(),
                        event_source: event_config.clone(),
                    },
                );
            }
        }
    }

    view! {
       gtk::Box {
           orientation: gtk::Orientation::Vertical,
           #[name="eventsources_list"]
           gtk::ListBox {
               child: {
                   fill: true,
                   expand: true,
               }
           }
       }
    }
}
