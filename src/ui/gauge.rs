use super::theme::Theme;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Mesh, Pos2, Sense, Shape, Stroke
};

pub struct Gauge;

impl Gauge {
    const TICKS: [f32; 9] = [0.0, 5.0, 10.0, 50.0, 100.0, 250.0, 500.0, 750.0, 1000.0];

    fn fraction(mbps: f32) -> f32 {
        let m = mbps.clamp(0.0, *Self::TICKS.last().unwrap());
        let n = (Self::TICKS.len() - 1) as f32;
        for (i, w) in Self::TICKS.windows(2).enumerate() {
            if m <= w[1] {
                return (i as f32 + (m - w[0]) / (w[1] - w[0])) / n;
            }
        }
        1.0
    }

    fn gradient(upload: bool, t: f32) -> Color32 {
        let stops: [Color32; 3] = if upload {
            [
                Color32::from_rgb(123, 77, 255),
                Color32::from_rgb(191, 113, 255),
                Color32::from_rgb(255, 150, 230),
            ]
        } else {
            [
                Color32::from_rgb(28, 191, 255),
                Color32::from_rgb(70, 235, 235),
                Color32::from_rgb(120, 255, 175),
            ]
        };
        if t < 0.5 {
            Theme::mix(stops[0], stops[1], t * 2.0)
        } else {
            Theme::mix(stops[1], stops[2], (t - 0.5) * 2.0)
        }
    }

    pub fn show(ui: &mut egui::Ui, theme: &Theme, mbps: f32, caption: &str, color: Color32, size: f32) {
        let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
        let painter = ui.painter();
        let upload = color.r() > 100;

        let center = pos2(rect.center().x, rect.top() + size * 0.5);
        let thickness = size * 0.09;
        let radius = size * 0.5 - thickness;
        let start = 135f32.to_radians();
        let sweep = 270f32.to_radians();
        let angle = |frac: f32| start + sweep * frac;
        let point = |frac: f32, r: f32| -> Pos2 {
            let a = angle(frac);
            pos2(center.x + r * a.cos(), center.y + r * a.sin())
        };

        let inactive = Color32::from_rgb(38, 39, 59);
        let track: Vec<Pos2> = (0..=160).map(|i| point(i as f32 / 160.0, radius)).collect();
        painter.add(Shape::line(track.clone(), Stroke::new(thickness, inactive)));
        painter.circle_filled(track[0], thickness / 2.0, inactive);
        painter.circle_filled(*track.last().unwrap(), thickness / 2.0, inactive);

        let frac = Self::fraction(mbps);
        if frac > 0.002 {
            let steps = ((frac * 300.0).ceil() as usize).max(2);
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let col = Self::gradient(upload, frac * t);
                painter.circle_filled(point(frac * t, radius), thickness / 2.0 + 0.3, col);
            }
        }

        let label_r = radius - thickness * 0.5 - size * 0.065;
        let dim = Color32::from_rgb(105, 108, 128);
        for (i, v) in Self::TICKS.iter().enumerate() {
            let reached = mbps + 0.001 >= *v;
            painter.text(
                point(i as f32 / (Self::TICKS.len() - 1) as f32, label_r),
                Align2::CENTER_CENTER,
                format!("{}", *v as u32),
                FontId::proportional(size * 0.052),
                if reached {
                    theme.text
                } else {
                    dim
                },
            );
        }

        {
            let a = angle(frac);
            let dir = vec2(a.cos(), a.sin());
            let perp = vec2(-a.sin(), a.cos());
            let r0 = size * 0.01;
            let r1 = size * 0.245;
            let w0 = size * 0.032;
            let w1 = size * 0.016;
            let base = center + dir * r0;
            let tip = center + dir * r1;
            let dim = Color32::from_white_alpha(18);
            let bright = Color32::from_white_alpha(170);
            let mut m = Mesh::default();
            m.colored_vertex(base + perp * w0, dim);
            m.colored_vertex(base - perp * w0, dim);
            m.colored_vertex(tip + perp * w1, bright);
            m.colored_vertex(tip - perp * w1, bright);
            m.add_triangle(0, 1, 2);
            m.add_triangle(1, 2, 3);
            painter.add(Shape::mesh(m));
        }

        painter.text(
            center + vec2(0.0, size * 0.285),
            Align2::CENTER_CENTER,
            format!("{:.2}", mbps),
            FontId::proportional(size * 0.11),
            theme.text,
        );
        painter.text(
            center + vec2(0.0, size * 0.375),
            Align2::CENTER_CENTER,
            "Mbps",
            FontId::proportional(size * 0.045),
            theme.muted,
        );
        painter.text(
            center + vec2(0.0, size * 0.45),
            Align2::CENTER_CENTER,
            caption,
            FontId::proportional(size * 0.042),
            color,
        );
    }
}
