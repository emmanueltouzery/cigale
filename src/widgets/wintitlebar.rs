use super::addeventsourcedlg::Msg as AddEventSourceDialogMsg;
use super::addeventsourcedlg::{
    AddEventSourceDialog, AddEventSourceDialogParams, EventSourceEditModel,
};
use super::preferences::Msg as PreferencesMsg;
use super::preferences::Preferences;
use crate::config::Config;
use crate::icons::*;
use gtk::prelude::*;
use relm::{init, Component, Widget};
use relm_derive::{widget, Msg};
use std::collections::{HashMap, HashSet};

const SHORTCUTS_UI: &str = include_str!("shortcuts.ui");

#[derive(Msg)]
pub enum Msg {
    ScreenChanged,
    MainWindowStackReady(gtk::Stack),
    NewEventSourceClick,
    AddConfig(&'static str, String, HashMap<&'static str, String>),
    EventSourceNamesChanged(HashSet<String>),
    DisplayAbout,
    DisplayShortcuts,
    DisplayPreferences,
    ConfigUpdated(Box<Config>),
}

pub struct Model {
    relm: relm::Relm<WinTitleBar>,
    displaying_event_sources: bool,
    main_window_stack: Option<gtk::Stack>,
    existing_source_names: HashSet<String>,
    menu_popover: gtk::Popover,
    prefs_win: Option<Component<Preferences>>,
}

#[widget]
impl Widget for WinTitleBar {
    fn init_view(&mut self) {
        self.new_event_source_btn
            .get_style_context()
            .add_class("suggested-action");
        let vbox = gtk::BoxBuilder::new()
            .margin(10)
            .orientation(gtk::Orientation::Vertical)
            .build();
        let preferences_btn = gtk::ModelButtonBuilder::new().label("Preferences").build();
        relm::connect!(
            self.model.relm,
            &preferences_btn,
            connect_clicked(_),
            Msg::DisplayPreferences
        );
        vbox.add(&preferences_btn);
        let shortcuts_btn = gtk::ModelButtonBuilder::new().label("Shortcuts").build();
        relm::connect!(
            self.model.relm,
            &shortcuts_btn,
            connect_clicked(_),
            Msg::DisplayShortcuts
        );
        vbox.add(&shortcuts_btn);
        let about_btn = gtk::ModelButtonBuilder::new().label("About").build();
        relm::connect!(
            self.model.relm,
            &about_btn,
            connect_clicked(_),
            Msg::DisplayAbout
        );
        vbox.add(&about_btn);
        vbox.show_all();
        self.model.menu_popover.add(&vbox);
        self.menu_button.set_popover(Some(&self.model.menu_popover));
    }

    fn model(relm: &relm::Relm<Self>, existing_source_names: HashSet<String>) -> Model {
        Model {
            relm: relm.clone(),
            displaying_event_sources: false,
            main_window_stack: None,
            existing_source_names,
            menu_popover: gtk::Popover::new(None::<&gtk::MenuButton>),
            prefs_win: None,
        }
    }

    pub fn prepare_addedit_eventsource_dlg(
        main_win: &gtk::Window,
        existing_source_names: &HashSet<String>,
        edit_model: Option<EventSourceEditModel>,
    ) -> (gtk::Dialog, Component<AddEventSourceDialog>) {
        let dialog = gtk::DialogBuilder::new()
            .use_header_bar(1)
            .default_width(400)
            .default_height(250)
            .title(if edit_model.is_some() {
                "Edit event source"
            } else {
                "Add event source"
            })
            .transient_for(main_win)
            .build();
        let header_bar = dialog
            .get_header_bar()
            .unwrap()
            .dynamic_cast::<gtk::HeaderBar>()
            .unwrap();
        // i'm not using the 'official' dialog buttons,
        // because i've had problems with relm events
        // not propagating when using those. worked
        // fine when i started using my own buttons.
        let btn = gtk::Button::with_label("Next");
        btn.get_style_context().add_class("suggested-action");
        header_bar.pack_end(&btn);
        btn.show();
        let dialog_contents = init::<AddEventSourceDialog>(AddEventSourceDialogParams {
            existing_source_names: existing_source_names.clone(),
            next_btn: btn,
            dialog: dialog.clone(),
            edit_model,
        })
        .expect("error initializing the add event source modal");
        dialog
            .get_content_area()
            .pack_start(dialog_contents.widget(), true, true, 0);

        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        (dialog, dialog_contents)
    }

    fn get_main_window(&self) -> gtk::Window {
        self.model
            .main_window_stack
            .as_ref()
            .unwrap()
            .get_toplevel()
            .and_then(|w| w.dynamic_cast::<gtk::Window>().ok())
            .unwrap()
    }

    fn run_event_source_addedit_dlg(&self) {
        let main_win = self.get_main_window();
        let (dialog, dialog_contents) = Self::prepare_addedit_eventsource_dlg(
            &main_win,
            &self.model.existing_source_names,
            None,
        );
        relm::connect!(dialog_contents@AddEventSourceDialogMsg::AddConfig(ref providername, ref name, ref cfg),
                               self.model.relm, Msg::AddConfig(providername, name.clone(), cfg.clone()));
        let resp = dialog.run();
        match resp {
            gtk::ResponseType::Cancel | gtk::ResponseType::DeleteEvent => dialog.close(),
            _ => {}
        }
    }

    fn display_about() {
        let dlg = gtk::AboutDialogBuilder::new()
            .name("Cigale")
            .version(env!("CARGO_PKG_VERSION"))
            .logo_icon_name(Icon::APP_ICON.name())
            .website("https://github.com/emmanueltouzery/cigale/")
            .comments("Review your past activity")
            .build();
        dlg.run();
        dlg.close();
    }

    fn display_shortcuts(&self) {
        let win = gtk::Builder::from_string(SHORTCUTS_UI)
            .get_object::<gtk::Window>("shortcuts")
            .unwrap();
        win.set_title("Shortcuts");
        win.set_transient_for(Some(&self.get_main_window()));
        win.show();
    }

    fn display_preferences(&mut self) {
        self.model.prefs_win = Some(
            init::<Preferences>(self.get_main_window())
                .expect("error initializing the preferences window"),
        );
        let prefs_win = self.model.prefs_win.as_ref().unwrap();
        relm::connect!(prefs_win@PreferencesMsg::ConfigUpdated(ref cfg),
                               self.model.relm, Msg::ConfigUpdated(cfg.clone()));
        prefs_win
            .widget()
            .set_transient_for(Some(&self.get_main_window()));
        prefs_win.widget().show();
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::MainWindowStackReady(stack) => {
                self.model.main_window_stack = Some(stack.clone());
                self.main_window_stack_switcher
                    .set_stack(self.model.main_window_stack.as_ref());
                relm::connect!(
                    self.model.relm,
                    &stack,
                    connect_property_visible_child_name_notify(_),
                    Msg::ScreenChanged
                );
            }
            Msg::ScreenChanged => {
                self.model.displaying_event_sources = self
                    .model
                    .main_window_stack
                    .as_ref()
                    .unwrap()
                    .get_visible_child_name()
                    .as_ref()
                    .map(|s| s.as_str())
                    == Some("event-sources");
                self.header_bar.set_subtitle(
                    Some("Event Sources").filter(|_| self.model.displaying_event_sources),
                );
                self.new_event_source_btn
                    .set_visible(self.model.displaying_event_sources);
            }
            Msg::NewEventSourceClick => {
                self.run_event_source_addedit_dlg();
            }
            Msg::EventSourceNamesChanged(src) => {
                self.model.existing_source_names = src;
            }
            Msg::AddConfig(_, _, _) => {
                // this is meant for win... we emit here, not interested by it ourselves
            }
            Msg::DisplayAbout => Self::display_about(),
            Msg::DisplayShortcuts => self.display_shortcuts(),
            Msg::DisplayPreferences => self.display_preferences(),
            Msg::ConfigUpdated(_) => {
                // this is meant for win... we emit here, not interested by it ourselves
            }
        }
    }

    view! {
        #[name="header_bar"]
        gtk::HeaderBar {
            #[name="new_event_source_btn"]
            gtk::Button {
                label: "New",
                visible:false,
                clicked() => Msg::NewEventSourceClick,
            },
            show_close_button: true,
            title: Some("Cigale"),
            #[name="menu_button"]
            gtk::MenuButton {
                image: Some(&gtk::Image::from_icon_name(Some("open-menu-symbolic"), gtk::IconSize::Menu)),
                child: {
                    pack_type: gtk::PackType::End
                },
            },
            #[name="main_window_stack_switcher"]
            gtk::StackSwitcher {
                child: {
                    pack_type: gtk::PackType::End
                }
            },
        }
    }
}
