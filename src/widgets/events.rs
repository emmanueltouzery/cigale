use super::datepicker::DatePickerMsg::DayPicked as DatePickerDayPickedMsg;
use super::datepicker::*;
use super::event::EventListItem;
use crate::config::Config;
use crate::events::events::Event;
use chrono::prelude::*;
use gtk::prelude::*;
use relm::{Channel, ContainerWidget, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    EventSelected,
    DayChange(Date<Local>),
    GotEvents(Result<Vec<Event>, String>),
    ConfigUpdate(Box<Config>), // box to prevent large size difference between variants
}

pub struct Model {
    config: Config,
    relm: relm::Relm<EventView>,
    // events will be None while we're loading
    events: Option<Result<Vec<Event>, String>>,
    current_event: Option<Event>,
    day: Date<Local>,
}

#[widget]
impl Widget for EventView {
    fn init_view(&mut self) {
        self.header_label
            .get_style_context()
            .add_class("event_header_label");
        self.update_events();

        relm::connect!(
            self.model.relm,
            self.event_list,
            connect_row_selected(_, _),
            Msg::EventSelected
        );
    }

    fn model(relm: &relm::Relm<Self>, config: Config) -> Model {
        let day = Local::today().pred();
        EventView::fetch_events(&config, relm, day);
        Model {
            config,
            relm: relm.clone(),
            events: None,
            current_event: None,
            day,
        }
    }

    fn update_events(&mut self) {
        self.model.current_event = None;
        for child in self.event_list.get_children() {
            self.event_list.remove(&child);
        }
        match &self.model.events {
            Some(Ok(events)) => {
                for event in events {
                    let _child = self.event_list.add_widget::<EventListItem>(event.clone());
                }
            }
            Some(Err(err)) => {
                let info_contents = self
                    .info_bar
                    .get_content_area()
                    .unwrap()
                    .dynamic_cast::<gtk::Box>() // https://github.com/gtk-rs/gtk/issues/947
                    .unwrap();
                for child in info_contents.get_children() {
                    info_contents.remove(&child);
                }
                info_contents.add(
                    &gtk::LabelBuilder::new()
                        .label(err.to_string().as_str())
                        .ellipsize(pango::EllipsizeMode::End)
                        .build(),
                );
                info_contents.show_all();
            }
            None => {}
        }

        let has_event_sources =
            !super::win::Win::config_source_names(&self.model.config).is_empty();
        self.events_stack
            .set_visible_child_name(if has_event_sources {
                "events"
            } else {
                "no-event-sources"
            });
    }

    fn fetch_events(config: &Config, relm: &relm::Relm<Self>, day: Date<Local>) {
        let stream = relm.stream().clone();
        let (_channel, sender) = Channel::new(move |events| {
            stream.emit(Msg::GotEvents(events));
        });
        let c = config.clone();
        std::thread::spawn(move || {
            sender
                .send(crate::events::events::get_all_events(c, day).map_err(|e| e.to_string()))
                .unwrap_or_else(|err| println!("Thread communication error: {}", err));
        });
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::EventSelected => {
                if let Some(Ok(events)) = &self.model.events {
                    let selected_index_maybe = self
                        .event_list
                        .get_selected_row()
                        .map(|r| r.get_index() as usize);
                    self.model.current_event = selected_index_maybe
                        .and_then(|idx| events.get(idx))
                        .cloned();
                }
            }
            Msg::DayChange(day) => {
                self.model.events = None;
                self.model.day = day;
                self.update_events();
                EventView::fetch_events(&self.model.config, &self.model.relm, day);
            }
            Msg::GotEvents(events) => {
                self.model.events = Some(events);
                self.update_events();
            }
            Msg::ConfigUpdate(config) => {
                self.model.config = *config;
                EventView::fetch_events(&self.model.config, &self.model.relm, self.model.day);
            }
        }
    }

    view! {
        #[name="events_stack"]
        gtk::Stack {
            gtk::Box {
                child: {
                    name: Some("events")
                },
                orientation: gtk::Orientation::Vertical,
                gtk::Box {
                    orientation: gtk::Orientation::Horizontal,
                    DatePicker {
                        DatePickerDayPickedMsg(d) => Msg::DayChange(d)
                    },
                    gtk::Spinner {
                        property_active: self.model.events.is_none()
                    }
                },
                #[name="info_bar"]
                gtk::InfoBar {
                    revealed: self.model.events.as_ref()
                                               .filter(|r| r.is_err())
                                               .is_some(),
                    message_type: gtk::MessageType::Error,
                },
                gtk::Box {
                    orientation: gtk::Orientation::Horizontal,
                    child: {
                        fill: true,
                        expand: true,
                    },
                    gtk::ScrolledWindow {
                        halign: gtk::Align::Start,
                        property_width_request: 350,
                        gtk::Box {
                            #[name="event_list"]
                            gtk::ListBox {
                                child: {
                                    fill: true,
                                    expand: true,
                                }
                            }
                        }
                    },
                    gtk::Box {
                        orientation: gtk::Orientation::Vertical,
                        valign: gtk::Align::Fill,
                        child: {
                            fill: true,
                            expand: true,
                            padding: 10, // horizontal padding for the label
                        },
                        #[name="header_label"]
                        gtk::Label {
                            child: {
                                padding: 10, // vertical padding for the label
                                fill: true,
                                expand: false,
                                pack_type: gtk::PackType::Start,
                            },
                            xalign: 0.0,
                            halign: gtk::Align::Start,
                            valign: gtk::Align::Start,
                            line_wrap: true,
                            selectable: true,
                            text: self.model
                                      .current_event
                                      .as_ref()
                                      .map(|e| e.event_contents_header.as_str())
                                      .unwrap_or("No current event")
                        },
                        gtk::ScrolledWindow {
                            child: {
                                expand: true,
                                fill: true,
                                pack_type: gtk::PackType::Start,
                            },
                            propagate_natural_height: true,
                            gtk::Box {
                                // two labels: one in case we have markup, one in case we have plain text.
                                // I used to have a single label for both,  using use_markup and text, and it worked,
                                // but there was no guarantee on the other in which both fields were updated. If the text
                                // was updated before 'use_markup', i could get text interpreted as markup which was not markup,
                                // then GtkLabel would fail and never recover displaying markup.
                                gtk::Label {
                                    // text label, not used when we display markup
                                    child: {
                                        pack_type: gtk::PackType::Start,
                                        fill: true,
                                        expand: true,
                                        padding: 10,
                                    },
                                    halign: gtk::Align::Start,
                                    valign: gtk::Align::Start,
                                    selectable: true,
                                    xalign: 0.0,
                                    yalign: 0.0,
                                    line_wrap: true,
                                    visible: self.model.current_event.as_ref()
                                                                     .filter(|e| e.event_contents_body.is_markup())
                                                                     .is_none(),
                                    text: self.model
                                              .current_event
                                              .as_ref()
                                              .filter(|e| !e.event_contents_body.is_markup())
                                              .map(|e| e.event_contents_body.as_str())
                                              .unwrap_or(""),
                                },
                                gtk::Label {
                                    // markup label, not used when we display text
                                    child: {
                                        pack_type: gtk::PackType::Start,
                                        fill: true,
                                        expand: true,
                                        padding: 10,
                                    },
                                    halign: gtk::Align::Start,
                                    valign: gtk::Align::Start,
                                    selectable: true,
                                    xalign: 0.0,
                                    yalign: 0.0,
                                    line_wrap: self.model.current_event.as_ref()
                                                                       .filter(|e| e.event_contents_body.is_markup())
                                                                       .map(|e| e.event_contents_body.is_word_wrap())
                                                                       .unwrap_or(false),
                                    visible: self.model.current_event.as_ref()
                                                                     .filter(|e| e.event_contents_body.is_markup())
                                                                     .is_some(),
                                    markup: self.model.current_event.as_ref()
                                                                    .filter(|e| e.event_contents_body.is_markup())
                                                                    .map(|e| e.event_contents_body.as_str())
                                                                    .unwrap_or(""),
                                }
                            }
                        }
                    }
                },
            },
            gtk::Label {
                child: {
                    name: Some("no-event-sources")
                },
                text: "No event sources have been set up yet.\n\nUse the second tab to configure event sources.",
                justify: gtk::Justification::Center,
                use_markup: true
            }
        }
    }
}
