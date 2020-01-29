const FONT_AWESOME_SVGS_ROOT: &str = "fontawesome-free-5.12.0-desktop/svgs/solid";

// TODO load the icons i'm interested in only once, put them
// in the binary
pub fn fontawesome_image(image_name: &str, size: i32) -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_file_at_size(
        format!(
            "/home/emmanuel/home/cigale/{}/{}.svg",
            FONT_AWESOME_SVGS_ROOT, image_name
        ),
        size,
        size,
    )
    .unwrap()
}
