use super::eventsource::{EventSourceListItem, EventSourceListItemInfo, EventSourceListItemMsg};
use crate::config::Config;
use gtk::prelude::*;
use relm::ContainerWidget;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    ConfigUpdate(Config),
    ActionsClicked(gtk::Button, &'static str, String),
    EditEventSource(&'static str, String),
    RemoveEventSource(&'static str, String),
}

pub struct Model {
    config: Config,
    relm: relm::Relm<EventSources>,
    eventsource_action_popover: gtk::Popover,
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
            config,
            relm: relm.clone(),
            eventsource_action_popover: gtk::PopoverBuilder::new()
                .position(gtk::PositionType::Bottom)
                .build(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::ConfigUpdate(cfg) => {
                self.model.config = cfg;
                self.update_eventsources();
            }
            Msg::ActionsClicked(btn, ep_name, config_name) => {
                // the actions button for an event source was clicked
                // display the popover with actions (remove, edit...)
                let popover = &self.model.eventsource_action_popover;
                popover.popdown();

                for child in popover.get_children() {
                    popover.remove(&child);
                }
                popover.set_relative_to(Some(&btn));
                let vbox = gtk::BoxBuilder::new()
                    .margin(10)
                    .orientation(gtk::Orientation::Vertical)
                    .build();
                let edit_btn = gtk::ModelButtonBuilder::new().label("Edit").build();
                let remove_btn = gtk::ModelButtonBuilder::new().label("Remove").build();
                // my parent is listening to these editeventsource / removeeventsource event.
                let config_name1 = config_name.clone();
                relm::connect!(
                    self.model.relm,
                    &edit_btn,
                    connect_clicked(_),
                    // TODO i'd need the connect! macro to do a "move ||" to avoid the clone
                    Msg::EditEventSource(ep_name, config_name1.clone())
                );
                relm::connect!(
                    self.model.relm,
                    &remove_btn,
                    connect_clicked(_),
                    Msg::RemoveEventSource(ep_name, config_name.clone())
                );
                vbox.add(&edit_btn);
                vbox.add(&remove_btn);
                popover.add(&vbox);
                vbox.show_all();
                popover.popup();
            }
            Msg::EditEventSource(_, _) => {
                // that's meant only for my parent, not for me.
            }
            Msg::RemoveEventSource(_, _) => {
                // that's meant only for my parent, not for me.
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
                let child = self.eventsources_list.add_widget::<EventSourceListItem>(
                    EventSourceListItemInfo {
                        event_provider_name: event_provider.name(),
                        event_provider_icon: event_provider.default_icon(),
                        config_name: event_config_name.to_string(),
                        event_source: event_config.clone(),
                    },
                );
                let ep_name = event_provider.name();
                let cfg_name = event_config_name.to_string();
                // this is a little confusing for me here, but somehow
                // the child doesn't get notified of an event triggered
                // there, but I as the parent get notified. So handle it here.
                relm::connect!(
                    child@EventSourceListItemMsg::ActionsClicked(ref btn),
                    self.model.relm,
                    Msg::ActionsClicked(btn.clone(), ep_name, cfg_name.clone())
                );
            }
        }
        let children = self.eventsources_list.get_children();
        self.eventsources_stack
            .set_visible_child_name(if children.is_empty() {
                "no-events"
            } else {
                "events"
            });
        for child in children {
            // don't want the row background color to change when we hover
            // it with the mouse (activatable), or the focus dotted lines
            // around the rows to be drawn, for aesthetic reasons.
            let row = child.dynamic_cast::<gtk::ListBoxRow>().unwrap();
            row.set_activatable(false);
            row.set_can_focus(false);
        }
    }

    view! {
        #[name="eventsources_stack"]
        gtk::Stack {
            gtk::ScrolledWindow {
                child: {
                    name: Some("events")
                },
                #[name="eventsources_list"]
                gtk::ListBox {
                    selection_mode: gtk::SelectionMode::None,
                }
            },
            gtk::Label {
                child: {
                    name: Some("no-events")
                },
                markup: "No event sources have been set up yet.\n\nUse the <b>'New'</b> button on the top-left of this window to add one.",
                justify: gtk::Justification::Center,
            }
        }
    }
}
