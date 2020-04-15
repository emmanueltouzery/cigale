#[derive(PartialEq, Debug, Clone)]
pub struct Icon(&'static str);

impl Icon {
    pub fn name(&self) -> &'static str {
        self.0
    }

    pub const ANGLE_LEFT: Icon = Icon("angle-left-symbolic");
    pub const ANGLE_RIGHT: Icon = Icon("angle-right-symbolic");
    pub const CALENDAR_ALT: Icon = Icon("calendar-alt-symbolic");
    pub const TASKS: Icon = Icon("tasks-symbolic");
    pub const COMMENT_DOTS: Icon = Icon("comment-dots-symbolic");
    pub const CODE_BRANCH: Icon = Icon("code-branch-symbolic");
    pub const ENVELOPE: Icon = Icon("envelope-symbolic");
    pub const THUMBS_UP: Icon = Icon("thumbs-up-symbolic");
    pub const CHECK_SQUARE: Icon = Icon("check-square-symbolic");
    pub const COPY: Icon = Icon("copy-symbolic");
    pub const COG: Icon = Icon("cog-symbolic");
    pub const EXCLAMATION_TRIANGLE: Icon = Icon("exclamation-triangle-symbolic");
    pub const APP_ICON: Icon = Icon("com.github.emmanueltouzery.cigale");
}
