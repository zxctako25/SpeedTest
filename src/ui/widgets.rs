use super::theme::Theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, Vec2};

pub struct StatCard;

impl StatCard {
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        size: Vec2,
        label: &str,
        value: &str,
        unit: &str,
        accent: Color32,
    ) {
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let value_font = FontId::proportional(24.0);
        let value_width = ui.fonts(|f| f.layout_no_wrap(value.to_string(), value_font.clone(), theme.text).size().x);

        let p = ui.painter();
        p.rect_filled(rect, theme.rounding, theme.card);

        let bar = Rect::from_min_size(rect.left_top() + vec2(0.0, 14.0), vec2(3.0, rect.height() - 28.0));
        p.rect_filled(bar, 2.0, accent);
        p.text(
            rect.left_top() + vec2(16.0, 12.0),
            Align2::LEFT_TOP,
            label,
            FontId::proportional(11.0),
            theme.muted,
        );
        
        p.text(rect.left_top() + vec2(16.0, 28.0), Align2::LEFT_TOP, value, value_font, theme.text);
        if !unit.is_empty() {
            p.text(
                rect.left_top() + vec2(16.0 + value_width + 5.0, 38.0),
                Align2::LEFT_TOP,
                unit,
                FontId::proportional(12.0),
                theme.muted,
            );
        }
    }
}

pub struct InfoCard;

impl InfoCard {
    pub fn show(ui: &mut egui::Ui, theme: &Theme, rows: &[(&str, String)]) {
        let row_h = 26.0;
        let pad = 12.0;
        let size = vec2(ui.available_width(), row_h * rows.len() as f32 + pad * 2.0);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

        let p = ui.painter();
        p.rect_filled(rect, theme.rounding, theme.card);

        let value_x = rect.left() + 108.0;
        let max_chars = (((rect.right() - 14.0 - value_x) / 7.6).floor() as usize).max(8);

        for (i, (label, value)) in rows.iter().enumerate() {
            let y = rect.top() + pad + row_h * i as f32 + row_h / 2.0;
            p.text(
                pos2(rect.left() + 16.0, y),
                Align2::LEFT_CENTER,
                *label,
                FontId::proportional(11.0),
                theme.muted,
            );
            p.text(
                pos2(value_x, y),
                Align2::LEFT_CENTER,
                fit(value, max_chars),
                FontId::proportional(14.0),
                theme.text,
            );
        }
    }
}

fn fit(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{t}...")
    }
}

pub struct ActionButton;

impl ActionButton {
    pub fn show(ui: &mut egui::Ui, theme: &Theme, height: f32, text: &str, color: Color32) -> bool {
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
        let fill = if resp.is_pointer_button_down_on() {
            Theme::mix(color, Color32::BLACK, 0.2)
        } else if resp.hovered() {
            Theme::mix(color, Color32::WHITE, 0.1)
        } else {
            color
        };

        let p = ui.painter();
        p.rect_filled(rect, theme.rounding, fill);
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(17.0),
            Color32::WHITE,
        );

        resp.clicked()
    }
}
