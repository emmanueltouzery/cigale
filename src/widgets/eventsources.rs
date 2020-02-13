use crate::config::Config;
use gtk::prelude::*;
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

    fn update(&mut self, event: Msg) {}

    fn update_eventsources(&mut self) {
        for child in self.eventsources_list.get_children() {
            self.eventsources_list.remove(&child);
        }
        // for event in events {
        //     let _child = self.event_list.add_widget::<EventListItem>(event.clone());
        // }
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
