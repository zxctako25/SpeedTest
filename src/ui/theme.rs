use eframe::egui::{
    self, Color32, Stroke
};

pub struct Theme {
    pub bg: Color32,
    pub card: Color32,
    pub track: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub download: Color32,
    pub upload: Color32,
    pub primary: Color32,
    pub stop: Color32,
    pub error: Color32,
    pub rounding: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color32::from_rgb(24, 24, 27),
            card: Color32::from_rgb(39, 39, 44),
            track: Color32::from_rgb(52, 52, 58),
            text: Color32::from_rgb(245, 245, 247),
            muted: Color32::from_rgb(140, 140, 150),
            download: Color32::from_rgb(0, 200, 255),
            upload: Color32::from_rgb(190, 100, 255),
            primary: Color32::from_rgb(0, 120, 215),
            stop: Color32::from_rgb(200, 60, 60),
            error: Color32::from_rgb(235, 90, 90),
            rounding: 12.0,
        }
    }
}

impl Theme {
    pub fn apply(&self, ctx: &egui::Context) {
        let mut v = egui::Visuals::dark();
        v.panel_fill = self.bg;
        v.window_fill = self.bg;
        v.widgets.noninteractive.bg_stroke = Stroke::NONE;
        ctx.set_visuals(v);
    }

    pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
        let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
        Color32::from_rgb(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()))
    }
}
