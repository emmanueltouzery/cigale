use crate::config::{Config, PrevNextDaySkipWeekends};
use gtk::prelude::*;
use gtk::traits::SettingsExt;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum Msg {
    DarkThemeToggled(bool),
    PrevNextSkipWeekendsToggled(bool),
    ConfigUpdated(Box<Config>),
    KeyPress(gdk::EventKey),
}

pub struct Model {
    relm: relm::Relm<Preferences>,
    prefer_dark_theme: bool,
    prev_next_day_skip_weekends: PrevNextDaySkipWeekends,
    config: Config,
    win: gtk::Window,
}

#[widget]
impl Widget for Preferences {
    fn init_view(&mut self) {}

    fn model(relm: &relm::Relm<Self>, win: gtk::Window) -> Model {
        let config = Config::read_config();
        let prefer_dark_theme = config.prefer_dark_theme;
        let prev_next_day_skip_weekends = config.prev_next_day_skip_weekends;
        Model {
            relm: relm.clone(),
            prefer_dark_theme,
            prev_next_day_skip_weekends,
            config,
            win,
        }
    }

    fn update_config(&self) {
        self.model.config.save_config(&self.model.win);
        self.model
            .relm
            .stream()
            .emit(Msg::ConfigUpdated(Box::new(self.model.config.clone())));
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::DarkThemeToggled(t) => {
                gtk::Settings::default()
                    .unwrap()
                    .set_gtk_application_prefer_dark_theme(t);
                self.model.config.prefer_dark_theme = t;
                self.update_config();
            }
            Msg::PrevNextSkipWeekendsToggled(t) => {
                self.model.config.prev_next_day_skip_weekends = if t {
                    PrevNextDaySkipWeekends::Skip
                } else {
                    PrevNextDaySkipWeekends::DontSkip
                };
                self.update_config();
            }
            Msg::ConfigUpdated(_) => {
                // meant for my parent, not for me
            }
            Msg::KeyPress(key) => {
                if key.keyval() == gdk::keys::constants::Escape {
                    self.widgets.prefs_win.close();
                }
            }
        }
    }

    view! {
        #[name="prefs_win"]
        gtk::Window {
            titlebar: view! {
                gtk::HeaderBar {
                    title: Some("Preferences"),
                    show_close_button: true,
                },
            },
            default_width: 600,
            default_height: 200,
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                margin_top: 10,
                margin_start: 6,
                margin_end: 6,
                margin_bottom: 6,
                spacing: 6,
                gtk::CheckButton {
                    label: "Prefer dark theme",
                    active: self.model.prefer_dark_theme,
                    toggled(t) => Msg::DarkThemeToggled(t.is_active())
                },
                gtk::CheckButton {
                    label: "Previous & Next day skip week-ends",
                    active: self.model.prev_next_day_skip_weekends == PrevNextDaySkipWeekends::Skip,
                    toggled(t) => Msg::PrevNextSkipWeekendsToggled(t.is_active())
                },
            },
            key_press_event(_, key) => (Msg::KeyPress(key.clone()), Inhibit(false)), // just for the ESC key.. surely there's a better way..
        }
    }
}
