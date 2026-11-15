use super::gauge::Gauge;
use super::theme::Theme;
use super::widgets::{
    ActionButton, InfoCard, StatCard
};

use crate::speedtest::model::Phase;
use crate::speedtest::session::Session;
use eframe::egui::{
    self, vec2, RichText
};

pub struct SpeedTestWindow {
    session: Session,
    theme: Theme,
}

impl SpeedTestWindow {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = Theme::default();
        theme.apply(&cc.egui_ctx);
        let mut session = Session::new();
        session.lookup_info(&cc.egui_ctx);
        Self { 
            session, theme 
        }
    }
}

impl eframe::App for SpeedTestWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.session.poll();
        let mut toggle = false;

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.theme.bg).inner_margin(egui::Margin::same(16.0)))
            .show(ctx, |ui| {
                let theme = &self.theme;
                let s = &self.session;
                let color = if s.phase == Phase::Upload {
                    theme.upload
                } else {
                    theme.download
                };
                
                let shown = ctx.animate_value_with_time(egui::Id::new("gauge"), s.live_mbps as f32, 0.25);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("SPEED TEST").size(16.0).strong().color(theme.text));
                    ui.add_space(6.0);
                    let size = (ui.available_height() - 385.0)
                        .clamp(170.0, 320.0)
                        .min(ui.available_width());
                    Gauge::show(ui, theme, shown, s.phase.caption(), color, size);
                });

                ui.add_space(8.0);
                let gap = 8.0;
                ui.spacing_mut().item_spacing = vec2(gap, gap);
                let card = vec2((ui.available_width() - gap) / 2.0, 64.0);
                let stat = |v: Option<f64>, decimals: usize, unit: &str| match v {
                    Some(x) => (format!("{:.*}", decimals, x), unit.to_string()),
                    None => ("--".to_string(), String::new()),
                };
                
                let r = &s.result;
                ui.horizontal(|ui| {
                    let (v, u) = stat(r.ping_ms, 0, "ms");
                    StatCard::show(ui, theme, card, "PING", &v, &u, theme.muted);
                    let (v, u) = stat(r.jitter_ms, 1, "ms");
                    StatCard::show(ui, theme, card, "JITTER", &v, &u, theme.muted);
                });
                ui.horizontal(|ui| {
                    let (v, u) = stat(r.download_mbps, 1, "Mbps");
                    StatCard::show(ui, theme, card, "DOWNLOAD", &v, &u, theme.download);
                    let (v, u) = stat(r.upload_mbps, 1, "Mbps");
                    StatCard::show(ui, theme, card, "UPLOAD", &v, &u, theme.upload);
                });

                let dash = || "--".to_string();
                let or_dash = |v: &str| if v.is_empty() {
                    dash() 
                } else {
                    v.to_string()
                };
                
                let (provider, location, ip) = match &s.info {
                    Some(i) => {
                        let location = match (i.city.is_empty(), i.country.is_empty()) {
                            (false, false) => format!("{}, {}", i.city, i.country),
                            (false, true) => i.city.clone(),
                            (true, false) => i.country.clone(),
                            (true, true) => dash(),
                        };
                        (or_dash(&i.isp), location, or_dash(&i.ip))
                    }
                    None => (dash(), dash(), dash()),
                };

                InfoCard::show(
                    ui,
                    theme,
                    &[
                        ("PROVIDER", provider),
                        ("LOCATION", location),
                        ("IP ADDRESS", ip),
                    ],
                );

                if let Some(err) = &s.error {
                    ui.label(RichText::new(err).size(13.0).color(theme.error));
                }

                ui.add_space((ui.available_height() - 48.0).max(4.0));
                let (text, btn_color) = if s.is_running() {
                    ("STOP", theme.stop)
                } else if s.phase == Phase::Done || s.phase == Phase::Failed {
                    ("TEST AGAIN", theme.primary)
                } else {
                    ("START", theme.primary)
                };
                
                if ActionButton::show(ui, theme, 46.0, text, btn_color) {
                    toggle = true;
                }
            });

        if toggle {
            if self.session.is_running() {
                self.session.stop();
            } else {
                self.session.start(ctx);
            }
        }
    }
}
