macro_rules! include_fontawesome_svg {
    ($ file : expr) => {
        include_bytes!(concat!(
            "../fontawesome-free-5.12.0-desktop/svgs/solid/",
            $file,
            ".svg"
        ))
    };
}

const FONTAWESOME_COG_SVG: &[u8] = include_fontawesome_svg!("cog");
pub const FONTAWESOME_CODE_BRANCH_SVG: &[u8] = include_fontawesome_svg!("code-branch");
pub const FONTAWESOME_ENVELOPE_SVG: &[u8] = include_fontawesome_svg!("envelope");
pub const FONTAWESOME_TASKS_SVG: &[u8] = include_fontawesome_svg!("tasks");
const FONTAWESOME_ANGLE_LEFT_SVG: &[u8] = include_fontawesome_svg!("angle-left");
const FONTAWESOME_ANGLE_RIGHT_SVG: &[u8] = include_fontawesome_svg!("angle-right");
pub const FONTAWESOME_CALENDAR_ALT_SVG: &[u8] = include_fontawesome_svg!("calendar-alt");
const FONTAWESOME_EXCLAMATION_TRIANGLE_SVG: &[u8] =
    include_fontawesome_svg!("exclamation-triangle");
pub const FONTAWESOME_COMMENT_DOTS_SVG: &[u8] = include_fontawesome_svg!("comment-dots");
pub const FONTAWESOME_CHECK_SQUARE_SVG: &[u8] = include_fontawesome_svg!("check-square");
const APPICON_SVG: &[u8] = include_bytes!("../com.github.emmanueltouzery.cigale.svg");

pub fn load_pixbuf(icon_bytes: &'static [u8], size: i32) -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_stream_at_scale(
        &gio::MemoryInputStream::new_from_bytes(&glib::Bytes::from_static(icon_bytes)),
        size,
        size,
        true,
        gio::NONE_CANCELLABLE,
    )
    .expect("loading icon")
}

pub fn app_icon(size: i32) -> gdk_pixbuf::Pixbuf {
    load_pixbuf(APPICON_SVG, size)
}

pub fn fontawesome_cog(size: i32) -> gdk_pixbuf::Pixbuf {
    load_pixbuf(FONTAWESOME_COG_SVG, size)
}

pub fn fontawesome_angle_left(size: i32) -> gdk_pixbuf::Pixbuf {
    load_pixbuf(FONTAWESOME_ANGLE_LEFT_SVG, size)
}

pub fn fontawesome_angle_right(size: i32) -> gdk_pixbuf::Pixbuf {
    load_pixbuf(FONTAWESOME_ANGLE_RIGHT_SVG, size)
}

pub fn fontawesome_calendar_alt(size: i32) -> gdk_pixbuf::Pixbuf {
    load_pixbuf(FONTAWESOME_CALENDAR_ALT_SVG, size)
}

pub fn fontawesome_exclamation_triangle(size: i32) -> gdk_pixbuf::Pixbuf {
    load_pixbuf(FONTAWESOME_EXCLAMATION_TRIANGLE_SVG, size)
}
