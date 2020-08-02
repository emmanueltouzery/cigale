use relm::Widget;
mod config;
mod events;
mod icons;
mod widgets;

fn main() {
    env_logger::init();

    let res_bytes = include_bytes!("icons.bin");
    let data = glib::Bytes::from(&res_bytes[..]);
    let resource = gio::Resource::from_data(&data).unwrap();
    gio::resources_register(&resource);

    widgets::win::Win::run(()).unwrap();
}
