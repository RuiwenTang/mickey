use std::{rc::Rc, time};

use mickey::*;
use nalgebra::Vector2;

mod common;

fn radian_to_degree(radian: f32) -> f32 {
    radian * 180.0 / std::f32::consts::PI
}

struct NanovgRender {
    begin_time: time::Instant,
    width: f32,
    height: f32,
    mouse_pos: (f32, f32),
    font: Rc<Font>,
    context: Option<GPUContext>,
}

impl NanovgRender {
    fn new(width: f32, height: f32) -> Self {
        let font = Font::new(
            FontDescription {
                name: "0xProtoNerdFont-Regular".to_string(),
                family: "0xProtoNerdFont".to_string(),
                style: FontStyle::normal(),
            },
            ab_glyph::FontArc::try_from_slice(include_bytes!(
                "./assets/0xProto/0xProtoNerdFont-Regular.ttf"
            ))
            .expect("Failed to load font"),
        );

        Self {
            begin_time: time::Instant::now(),
            width,
            height,
            mouse_pos: (0.0, 0.0),
            font: Rc::new(font),
            context: None,
        }
    }

    fn draw_graph(&self, recorder: &mut PictureRecorder, x: f32, y: f32, w: f32, h: f32, t: f64) {
        let t = t as f32;

        let samples = [
            (1.0 + (t * 1.2345).sin() + ((t * 0.3345).cos() * 0.44).sin()) * 0.5,
            (1.0 + (t * 0.68363).sin() + ((t * 1.3).cos() * 1.55).sin()) * 0.5,
            (1.0 + (t * 1.1642).sin() + ((t * 0.33457).cos() * 1.24).sin()) * 0.5,
            (1.0 + (t * 0.56345).sin() + ((t * 1.63).cos() * 0.14).sin()) * 0.5,
            (1.0 + (t * 1.6245).sin() + ((t * 0.254).cos() * 0.3).sin()) * 0.5,
            (1.0 + (t * 0.345).sin() + ((t * 0.03).cos() * 0.6).sin()) * 0.5,
        ];

        let dx = w / 5.0;

        let sx: Vec<f32> = samples
            .into_iter()
            .enumerate()
            .map(|(i, _v)| -> f32 { x + i as f32 * dx })
            .collect();

        let sy: Vec<f32> = samples
            .into_iter()
            .enumerate()
            .map(|(_i, v)| -> f32 { y + h * v * 0.8 })
            .collect();

        let mut paint = Paint::new();

        paint.color = LinearGradient::new(Point::from(x, y), Point::from(x, y + h))
            .add_color(Color::from_rgba_u8(0, 160, 192, 0))
            .add_color(Color::from_rgba_u8(0, 160, 192, 64))
            .into();

        let mut path = Path::new().move_to(sx[0], sy[0]);

        for i in 1..6 {
            path = path.cubic_to(
                sx[i - 1] + dx * 0.5,
                sy[i - 1],
                sx[i] - dx * 0.5,
                sy[i],
                sx[i],
                sy[i],
            );
        }

        path = path.line_to(x + w, y + h).line_to(x, y + h).close();

        recorder.draw_path(path, &paint);

        let mut graph_line = Path::new().move_to(sx[0], sy[0] + 2.0);

        for i in 1..6 {
            graph_line = graph_line.cubic_to(
                sx[i - 1] + dx * 0.5,
                sy[i - 1] + 2.0,
                sx[i] - dx * 0.5,
                sy[i] + 2.0,
                sx[i],
                sy[i] + 2.0,
            );
        }

        paint.color = Color::from_rgba_u8(0, 0, 0, 32).into();
        paint.style = Stroke::new().with_width(3.0).into();

        recorder.draw_path(graph_line, &paint);

        let mut graph_line = Path::new().move_to(sx[0], sy[0]);

        for i in 1..6 {
            graph_line = graph_line.cubic_to(
                sx[i - 1] + dx * 0.5,
                sy[i - 1],
                sx[i] - dx * 0.5,
                sy[i],
                sx[i],
                sy[i],
            );
        }

        paint.color = Color::from_rgba_u8(0, 160, 192, 255).into();
        recorder.draw_path(graph_line, &paint);

        paint.style = Style::Fill;

        for i in 0..6 {
            paint.color = RadialGradient::new(Point::from(sx[i], sy[i] + 2.0), 8.0)
                .with_colors_stops(
                    vec![Color::from_rgba_u8(0, 0, 0, 32), Color::transparent()],
                    vec![3.0 / 8.0, 1.0],
                )
                .into();

            recorder.draw_rect(
                &Rect::from_xywh(sx[i] - 10.0, sy[i] - 8.0, 20.0, 20.0),
                &paint,
            );

            paint.color = Color::from_rgba_u8(0, 160, 192, 255).into();
            recorder.draw_circle(sx[i], sy[i], 4.0, &paint);

            paint.color = Color::from_rgba_u8(220, 220, 220, 255).into();
            recorder.draw_circle(sx[i], sy[i], 2.0, &paint);
        }
    }

    fn draw_window(
        &self,
        recorder: &mut PictureRecorder,
        title: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let corner_radius = 3.0;

        let rrect = RRect::from_rect_xy(
            Rect::from_xywh(x - 5.0, y - 5.0, w + 10.0, h + 10.0),
            corner_radius,
            corner_radius,
        );

        let mut paint = Paint::new();
        paint.color = Color::from_rgba_u8(0, 0, 0, 64).into();

        recorder.draw_rrect(&rrect, &paint);

        paint.color = Color::from_rgba_u8(28, 30, 34, 192).into();

        let rrect = RRect::from_rect_xy(Rect::from_xywh(x, y, w, h), corner_radius, corner_radius);

        recorder.draw_rrect(&rrect, &paint);

        paint.color = LinearGradient::new(Point::from(x, y), Point::from(x, y + 15.0))
            .add_color(Color::from_rgba_u8(255, 255, 255, 8))
            .add_color(Color::from_rgba_u8(0, 0, 0, 16))
            .into();

        let rrect = RRect::from_rect_xy(
            Rect::from_xywh(x + 1.0, y + 1.0, w - 2.0, 30.0),
            corner_radius - 1.0,
            corner_radius - 1.0,
        );

        recorder.draw_rrect(&rrect, &paint);

        paint.style = Stroke::new().with_width(3.0).into();
        paint.color = Color::from_rgba_u8(0, 0, 0, 32).into();

        let header = Path::new()
            .move_to(x + 0.5, y + 0.5 + 30.0)
            .move_to(x + 0.5 + w - 1.0, y + 0.5 + 30.0);

        recorder.draw_path(header, &paint);

        let text_blob = TextBlobBuilder::new(self.font.clone(), 15.0).build(title);

        let tw = text_blob.width;
        let th = text_blob.height;
        recorder.draw_text(
            text_blob,
            Point::from(x + (w - tw) * 0.5, y + th),
            Color::from_rgba_u8(220, 220, 220, 160),
        );
    }

    fn draw_search_box(
        &self,
        recorder: &mut PictureRecorder,
        title: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let corner_radius = h / 2.0 - 1.0;
        let mut paint = Paint::new();

        let mut rrect =
            RRect::from_rect_xy(Rect::from_xywh(x, y, w, h), corner_radius, corner_radius);

        paint.color = LinearGradient::new(Point::from(x, y), Point::from(x, y + h))
            .with_colors(vec![
                Color::from_rgba_u8(0, 0, 0, 16),
                Color::from_rgba_u8(0, 0, 0, 92),
            ])
            .into();

        recorder.draw_rrect(&rrect, &paint);

        paint.color = Color::from_rgba_u8(0, 0, 0, 48).into();
        rrect = RRect::from_rect_xy(
            Rect::from_xywh(x + 0.5, y + 0.5, w - 1.0, h - 1.0),
            corner_radius - 0.5,
            corner_radius - 0.5,
        );

        recorder.draw_rrect(&rrect, &paint);

        let search_icon = TextBlobBuilder::new(self.font.clone(), h * 0.6).build("\u{f002}");

        recorder.draw_text(
            search_icon,
            Point {
                x: x + h * 0.3,
                y: y + h * 0.8,
            },
            Color::from_rgba_u8(255, 255, 255, 32),
        );

        let text_blob = TextBlobBuilder::new(self.font.clone(), 17.0).build(title);

        recorder.draw_text(
            text_blob,
            Point {
                x: x + h * 1.05,
                y: y + h * 0.5 + 6.0,
            },
            Color::from_rgba_u8(255, 255, 255, 32),
        );

        let cancle_icon = TextBlobBuilder::new(self.font.clone(), h * 0.6).build("\u{f2d3}");

        recorder.draw_text(
            cancle_icon,
            Point {
                x: x + w - h * 1.0,
                y: y + h * 0.7,
            },
            Color::from_rgba_u8(255, 255, 255, 32),
        );
    }

    fn draw_drop_down(
        &self,
        recorder: &mut PictureRecorder,
        text: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let corner_radius = 4.0;
        let mut paint = Paint::new();
        paint.color = LinearGradient::new(Point::from(x, y), Point::from(x, y + h))
            .with_colors(vec![
                Color::from_rgba_u8(255, 255, 255, 16),
                Color::from_rgba_u8(0, 0, 0, 16),
            ])
            .into();

        let mut rrect = RRect::from_rect_xy(
            Rect::from_xywh(x + 1.0, y + 1.0, w - 2.0, h - 2.0),
            corner_radius,
            corner_radius,
        );

        recorder.draw_rrect(&rrect, &paint);

        paint.style = Stroke::new().with_width(2.0).into();
        paint.color = Color::from_rgba_u8(0, 0, 0, 48).into();

        rrect = RRect::from_rect_xy(
            Rect::from_xywh(x + 0.5, y + 0.5, w - 1.0, h - 1.0),
            corner_radius - 0.5,
            corner_radius - 0.5,
        );

        recorder.draw_rrect(&rrect, &paint);

        let text_blob = TextBlobBuilder::new(self.font.clone(), 17.0).build(text);

        recorder.draw_text(
            text_blob,
            Point {
                x: x + h * 0.3,
                y: y + h * 0.7,
            },
            Color::from_rgba_u8(255, 255, 255, 160),
        );

        let angle_right = TextBlobBuilder::new(self.font.clone(), h * 1.1).build("\u{f105}");

        recorder.draw_text(
            angle_right,
            Point {
                x: x + w - h * 0.8,
                y: y + h * 0.9,
            },
            Color::from_rgba_u8(255, 255, 255, 160),
        );
    }

    fn draw_label(&self, recorder: &mut PictureRecorder, text: &str, x: f32, y: f32, h: f32) {
        let text_blob = TextBlobBuilder::new(self.font.clone(), 20.0).build(text);

        recorder.draw_text(
            text_blob,
            Point { x, y: y + h * 0.9 },
            Color::from_rgba_u8(255, 255, 255, 128),
        );
    }

    fn draw_eyes(
        &self,
        recorder: &mut PictureRecorder,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        mx: f32,
        my: f32,
        t: f64,
    ) {
        let ex = w * 0.23;
        let ey = h * 0.5;
        let lx = x + ex;
        let ly = y + ey;
        let rx = x + w - ex;
        let ry = y + ey;
        let br = if ex < ey { ex * 0.5 } else { ey * 0.5 };
        let blink = 1.0 - (t * 0.5).sin().powf(100.0) * 0.8;

        let mut bg = Paint::new();
        bg.style = Style::Fill;
        bg.color =
            LinearGradient::new(Point::from(x, y + h * 0.5), Point::from(x + w * 0.1, y + h))
                .with_colors(vec![
                    Color::from_rgba_u8(0, 0, 0, 32),
                    Color::from_rgba_u8(0, 0, 0, 16),
                ])
                .into();

        recorder.draw_oval(
            &Rect::from_ltrb(lx + 0.3 - ex, ly + 16.0 - ey, lx + 3.0 + ex, ly + 16.0 + ey),
            &bg,
        );

        recorder.draw_oval(
            &Rect::from_ltrb(rx + 3.0 - ex, ry + 16.0 - ey, rx + 3.0 + ex, ry + 16.0 + ey),
            &bg,
        );

        bg.color = LinearGradient::new(
            Point::from(x, y + h * 0.25),
            Point::from(x + w * 0.1, y + h),
        )
        .with_colors(vec![
            Color::from_rgba_u8(220, 220, 220, 255),
            Color::from_rgba_u8(128, 128, 128, 255),
        ])
        .into();

        recorder.draw_oval(&Rect::from_ltrb(lx - ex, ly - ey, lx + ex, ly + ey), &bg);
        recorder.draw_oval(&Rect::from_ltrb(rx - ex, ry - ey, rx + ex, ry + ey), &bg);

        let mut dx = (mx - rx) / (ex * 10.0);
        let mut dy = (my - ry) / (ey * 10.0);
        let mut d = (dx * dx + dy * dy).sqrt();

        if d > 1.0 {
            dx /= d;
            dy /= d;
        }

        dx *= ex * 0.4;
        dy *= ey * 0.5;

        bg.color = Color::from_rgba_u8(32, 32, 32, 255).into();
        recorder.draw_oval(
            &Rect::from_ltrb(
                lx + dx - br,
                ly + dy + ey * 0.25 * (1.0 - blink as f32) - br * blink as f32,
                lx + dx + br,
                ly + dy + ey * 0.25 * (1.0 - blink as f32) + br * blink as f32,
            ),
            &bg,
        );

        dx = (mx - rx) / (ex * 10.0);
        dy = (my - ry) / (ey * 10.0);
        d = (dx * dx + dy * dy).sqrt();
        if d > 1.0 {
            dx /= d;
            dy /= d;
        }

        dx *= ex * 0.4;
        dy *= ey * 0.5;

        recorder.draw_oval(
            &Rect::from_ltrb(
                rx + dx - br,
                ry + dy + ey * 0.25 * (1.0 - blink as f32) - br * blink as f32,
                rx + dx + br,
                ry + dy + ey * 0.25 * (1.0 - blink as f32) + br * blink as f32,
            ),
            &bg,
        );

        let mut gloss = Paint::new();
        gloss.style = Style::Fill;
        gloss.color = RadialGradient::new(Point::from(lx - ex * 0.25, ly - ey * 0.5), ex * 0.75)
            .add_color(Color::from_rgba_u8(225, 225, 225, 128))
            .add_color(Color::from_rgba_u8(225, 225, 225, 0))
            .into();

        recorder.draw_oval(&Rect::from_ltrb(lx - ex, ly - ey, lx + ex, ly + ey), &gloss);

        gloss.color = RadialGradient::new(Point::from(rx - ex * 0.25, ry - ey * 0.5), ex * 0.75)
            .add_color(Color::from_rgba_u8(225, 225, 225, 128))
            .add_color(Color::from_rgba_u8(225, 225, 225, 0))
            .into();

        recorder.draw_oval(&Rect::from_ltrb(rx - ex, ry - ey, rx + ex, ry + ey), &gloss);
    }

    fn draw_paragraph(&self, recorder: &mut PictureRecorder, x: f32, y: f32) {
        let blob_builder = TextBlobBuilder::new(self.font.clone(), 15.0);

        let blob1 = blob_builder.build("This is longer chunk of text.");
        let blob2 = blob_builder.build("Would have used lorem ipsum.");
        let blob3 = blob_builder.build("but she    was busy jumping");
        let blob4 = blob_builder.build("over the lazy dog with the fox");
        let blob5 = blob_builder.build("and all the men who came to");
        let blob6 = blob_builder.build("the aid of the party.🎉");

        let mut y_offset = y;
        recorder.draw_text(
            blob1.clone(),
            Point {
                x,
                y: y_offset + blob1.ascent,
            },
            Color::white(),
        );
        y_offset += blob1.height + blob1.line_gap;
        recorder.draw_text(
            blob2.clone(),
            Point {
                x,
                y: y_offset + blob2.ascent,
            },
            Color::white(),
        );
        y_offset += blob2.height + blob2.line_gap;
        recorder.draw_text(
            blob3.clone(),
            Point {
                x,
                y: y_offset + blob3.ascent,
            },
            Color::white(),
        );
        y_offset += blob3.height + blob3.line_gap;
        recorder.draw_text(
            blob4.clone(),
            Point {
                x,
                y: y_offset + blob4.ascent,
            },
            Color::white(),
        );
        y_offset += blob4.height + blob4.line_gap;
        recorder.draw_text(
            blob5.clone(),
            Point {
                x,
                y: y_offset + blob5.ascent,
            },
            Color::white(),
        );
        y_offset += blob5.height + blob5.line_gap;
        recorder.draw_text(
            blob6.clone(),
            Point {
                x,
                y: y_offset + blob6.ascent,
            },
            Color::white(),
        );
    }

    fn draw_color_wheel(
        &self,
        recorder: &mut PictureRecorder,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        t: f64,
    ) {
        let hue = (t * 0.12).sin() as f32;

        let cx = x + w * 0.5;
        let cy = y + h * 0.5;
        let r1 = w.min(h) * 0.5 - 5.0;
        let r0 = r1 - 20.0;
        let aeps = 0.5 / r1; // half a pixel arc length in radians (2pi cancels out).

        for i in 0..6 {
            let a0 = i as f32 / 6.0 * std::f32::consts::PI * 2.0 - aeps;
            let a1 = (i as f32 + 1.0) / 6.0 * std::f32::consts::PI * 2.0 + aeps;

            let p1_x = cx + r0 * a0.cos();
            let p1_y = cy + r0 * a0.sin();

            let p3_x = cx + r0 * a1.cos();
            let p3_y = cy + r0 * a1.sin();

            let p1r = Vector2::new(p1_x - cx, p1_y - cy).normalize();
            let p3r = Vector2::new(p3_x - cx, p3_y - cy).normalize();
            let p2rt = ((p1r + p3r) * 0.5).normalize();
            let p2r = Vector2::new(cx, cy)
                + p2rt
                    * (r0
                        + r0 * std::f32::consts::PI
                            * 0.1
                            * ((a1 - a0) * 2.0 / std::f32::consts::PI).powi(2));

            let p4_x = cx + a0.cos() * r1;
            let p4_y = cy + a0.sin() * r1;

            let p6_x = cx + a1.cos() * r1;
            let p6_y = cy + a1.sin() * r1;

            let p4r = Vector2::new(p4_x - cx, p4_y - cy).normalize();
            let p6r = Vector2::new(p6_x - cx, p6_y - cy).normalize();
            let p5rt = ((p4r + p6r) * 0.5).normalize();
            let p5r = Vector2::new(cx, cy)
                + p5rt
                    * (r1
                        + r1 * std::f32::consts::PI
                            * 0.1
                            * ((a1 - a0) * 2.0 / std::f32::consts::PI).powi(2));

            let path = Path::new()
                .move_to(p1_x, p1_y)
                .quad_to(p2r.x, p2r.y, p3_x, p3_y)
                .line_to(p6_x, p6_y)
                .quad_to(p5r.x, p5r.y, p4_x, p4_y)
                .close();

            let ax = cx + a0.cos() * (r0 + r1) * 0.5;
            let ay = cy + a0.sin() * (r0 + r1) * 0.5;
            let bx = cx + a1.cos() * (r0 + r1) * 0.5;
            let by = cy + a1.sin() * (r0 + r1) * 0.5;

            let mut paint = Paint::new();
            paint.color = LinearGradient::new(Point::from(ax, ay), Point::from(bx, by))
                .add_color(Color::from_hsla(
                    a0 / (2.0 * std::f32::consts::PI),
                    1.0,
                    0.55,
                    255,
                ))
                .add_color(Color::from_hsla(
                    a1 / (2.0 * std::f32::consts::PI),
                    1.0,
                    0.55,
                    255,
                ))
                .into();

            recorder.draw_path(path, &paint);
        }

        let mut paint = Paint::new();

        paint.style = Stroke::new()
            .with_join(StrokeJoin::Round)
            .with_width(1.0)
            .into();
        paint.color = Color::from_rgba_u8(0, 0, 0, 64).into();

        recorder.draw_circle(cx, cy, r0 - 0.5, &paint);
        recorder.draw_circle(cx, cy, r1 - 0.5, &paint);

        recorder.save();
        recorder.translate(cx, cy);

        recorder.rotate(radian_to_degree(hue * 2.0 * std::f32::consts::PI));

        paint.style = Stroke::new().with_width(2.0).into();
        paint.color = Color::from_rgba_u8(255, 255, 255, 192).into();

        recorder.draw_rect(&Rect::from_xywh(r0 - 1.0, -3.0, r1 - r0 + 2.0, 6.0), &paint);

        paint.style = Stroke::new().with_width(1.0).into();
        paint.color = Color::from_rgba_u8(0, 0, 0, 64).into();
        recorder.draw_rect(
            &Rect::from_xywh(r0 - 2.0, -4.0, r1 - r0 + 2.0 + 2.0, 6.0 + 2.0),
            &paint,
        );

        let r = r0 - 6.0;
        let ax = (120.0 / 180.0 * std::f32::consts::PI).cos() * r;
        let ay = (120.0 / 180.0 * std::f32::consts::PI).sin() * r;
        let bx = (-120.0 / 180.0 * std::f32::consts::PI).cos() * r;
        let by = (-120.0 / 180.0 * std::f32::consts::PI).sin() * r;

        let path = Path::new()
            .move_to(r, 0.0)
            .line_to(ax, ay)
            .line_to(bx, by)
            .close();

        paint.style = Style::Fill;
        paint.color = LinearGradient::new(Point::from(r, 0.0), Point::from(ax, ay))
            .add_color(Color::from_hsla(hue, 1.0, 0.5, 255))
            .add_color(Color::from_rgba_u8(255, 255, 255, 255))
            .into();

        recorder.draw_path(path.clone(), &paint);

        paint.color =
            LinearGradient::new(Point::from((r + ax) * 0.5, ay * 0.5), Point::from(bx, by))
                .add_color(Color::transparent())
                .add_color(Color::black())
                .into();

        recorder.draw_path(path.clone(), &paint);

        let ax = (120.0 / 180.0 * std::f32::consts::PI).cos() * r * 0.3;
        let ay = (120.0 / 180.0 * std::f32::consts::PI).sin() * r * 0.4;

        paint.style = Stroke::new().with_width(2.0).into();
        paint.color = Color::white().with_alpha(192.0 / 255.0).into();

        recorder.draw_circle(ax, ay, 5.0, &paint);

        paint.color = RadialGradient::new(Point::from(ax, ay), 9.0)
            .with_colors_stops(
                vec![Color::from_rgba_u8(0, 0, 0, 64), Color::transparent()],
                vec![7.0 / 9.0, 1.0],
            )
            .into();

        recorder.draw_circle(ax, ay, 8.0, &paint);

        recorder.restore();
    }

    fn draw_lines(&self, recorder: &mut PictureRecorder, x: f32, y: f32, w: f32, t: f64) {
        let pad = 5.0;
        let s = w / 9.0 - pad * 2.0;
        let t = t as f32;
        let pts: [Point; 4] = [
            Point::from(
                -s * 0.25 + (t * 0.3).cos() * s * 0.5,
                (t * 0.3).sin() * s * 0.5,
            ),
            Point::from(-s * 0.25, 0.0),
            Point::from(s * 0.25, 0.0),
            Point::from(
                s * 0.25 + (-t * 0.3).cos() * s * 0.5,
                (-t * 0.3).sin() * s * 0.5,
            ),
        ];

        let joins: [StrokeJoin; 3] = [StrokeJoin::Miter, StrokeJoin::Round, StrokeJoin::Bevel];
        let caps: [StrokeCap; 3] = [StrokeCap::Butt, StrokeCap::Round, StrokeCap::Square];

        for i in 0..3 {
            for j in 0..3 {
                let fx = x + s * 0.5 + ((i as f32) * 3.0 + j as f32) / 9.0 * w + pad;
                let fy = y - s * 0.5 + pad;

                recorder.save();

                recorder.translate(fx, fy);

                let mut paint = Paint::new();
                paint.style = Stroke::new()
                    .with_width(s * 0.3)
                    .with_cap(caps[i])
                    .with_join(joins[j])
                    .into();
                paint.color = Color::from_rgba_u8(0, 0, 0, 160).into();

                let path = Path::new()
                    .move_to_point(pts[0])
                    .line_to_point(pts[1])
                    .line_to_point(pts[2])
                    .line_to_point(pts[3]);

                recorder.draw_path(path.clone(), &paint);

                paint.style = Stroke::new()
                    .with_width(1.0)
                    .with_cap(StrokeCap::Butt)
                    .with_join(StrokeJoin::Bevel)
                    .into();
                paint.color = Color::from_rgba_u8(0, 192, 255, 255).into();

                recorder.draw_path(path, &paint);

                recorder.restore();
            }
        }
    }

    fn draw_widths(&self, recorder: &mut PictureRecorder, x: f32, y: f32, width: f32) {
        let mut paint = Paint::new();
        paint.color = Color::from_rgba_u8(0, 0, 0, 255).into();

        let mut y = y;
        for i in 0..20 {
            let w = (i as f32 + 0.5) * 0.1;
            paint.style = Stroke::new().with_width(w).into();

            let path = Path::new()
                .move_to(x, y)
                .line_to(x + width, y + width * 0.3);

            recorder.draw_path(path, &paint);

            y += 10.0;
        }
    }

    fn draw_caps(&self, recorder: &mut PictureRecorder, x: f32, y: f32, width: f32) {
        let caps = [StrokeCap::Butt, StrokeCap::Round, StrokeCap::Square];
        let line_width = 8.0;

        let mut paint = Paint::new();
        paint.style = Style::Fill;
        paint.color = Color::from_rgba_u8(255, 255, 255, 32).into();

        recorder.draw_rect(
            &Rect::from_xywh(x - line_width / 2.0, y, width + line_width, 40.0),
            &paint,
        );
        recorder.draw_rect(&Rect::from_xywh(x, y, width, 40.0), &paint);

        paint.color = Color::black().into();

        for (i, cap) in caps.iter().enumerate() {
            paint.style = Stroke::new().with_width(line_width).with_cap(*cap).into();

            let path = Path::new()
                .move_to(x, y + (i as f32 * 10.0) + 5.0)
                .line_to(x + width, y + (i as f32 * 10.0) + 5.0);

            recorder.draw_path(path, &paint);
        }
    }

    fn draw_scissor(&self, recorder: &mut PictureRecorder, x: f32, y: f32, t: f64) {
        recorder.save();
        recorder.translate(x, y);
        recorder.rotate(5.0);

        let mut paint = Paint::new();
        paint.style = Style::Fill;
        paint.color = Color::from_rgba_u8(255, 0, 0, 255).into();
        recorder.draw_rect(&Rect::from_xywh(-20.0, -20.0, 60.0, 40.0), &paint);

        recorder.translate(40.0, 0.0);
        recorder.rotate(radian_to_degree(t as f32));

        paint.color = Color::from_rgba_u8(255, 128, 0, 64).into();

        let rect = Rect::from_xywh(-20.0, -10.0, 60.0, 30.0);

        recorder.draw_rect(&rect, &paint);

        recorder.clip_rect(&rect, ClipOp::Intersect);

        recorder.rotate(radian_to_degree(t as f32));

        paint.color = Color::from_rgba_u8(255, 128, 0, 255).into();

        recorder.draw_rect(&rect, &paint);

        recorder.restore();
    }

    fn draw_edit_box_base(&self, recorder: &mut PictureRecorder, x: f32, y: f32, w: f32, h: f32) {
        let mut paint = Paint::new();
        paint.color = Color::from_rgba_u8(255, 255, 255, 32).into();

        let rrect = RRect::from_rect_xy(
            Rect::from_xywh(x + 1.0, y + 1.0, w - 2.0, h - 2.0),
            3.0,
            3.0,
        );

        recorder.draw_rrect(&rrect, &paint);

        paint.color = Color::from_rgba_u8(0, 0, 0, 48).into();
        paint.style = Stroke::new().with_width(1.0).into();

        let rrect = RRect::from_rect_xy(
            Rect::from_xywh(x + 0.5, y + 0.5, w - 1.0, h - 1.0),
            3.5,
            3.5,
        );
        recorder.draw_rrect(&rrect, &paint);
    }

    fn draw_edit_box(
        &self,
        recorder: &mut PictureRecorder,
        text: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        self.draw_edit_box_base(recorder, x, y, w, h);

        let text_blob = TextBlobBuilder::new(self.font.clone(), 17.0).build(text);

        recorder.draw_text(
            text_blob,
            Point {
                x: x + h * 0.3,
                y: y + h * 0.7,
            },
            Color::from_rgba_u8(255, 255, 255, 64),
        );
    }

    fn draw_edit_box_num(
        &self,
        recorder: &mut PictureRecorder,
        text: &str,
        units: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        self.draw_edit_box_base(recorder, x, y, w, h);

        let units_blob = TextBlobBuilder::new(self.font.clone(), 17.0).build(units);
        let text_blob = TextBlobBuilder::new(self.font.clone(), 20.0).build(text);

        let uw = units_blob.width;
        let tw = text_blob.width;

        recorder.draw_text(
            units_blob,
            Point {
                x: x + w - h * 0.3 - uw,
                y: y + h * 0.6,
            },
            Color::from_rgba_u8(255, 255, 255, 64),
        );

        recorder.draw_text(
            text_blob,
            Point {
                x: x + w - h * 0.5 - uw - tw,
                y: y + h * 0.65,
            },
            Color::from_rgba_u8(255, 255, 255, 128),
        );
    }

    fn draw_check_box(&self, recorder: &mut PictureRecorder, text: &str, x: f32, y: f32, h: f32) {
        let text_blob = TextBlobBuilder::new(self.font.clone(), 17.0).build(text);

        recorder.draw_text(
            text_blob,
            Point {
                x: x + 28.0,
                y: y + h * 0.7,
            },
            Color::from_rgba_u8(255, 255, 255, 160),
        );

        let rrect = RRect::from_rect_xy(
            Rect::from_xywh(x + 1.0, y + h * 0.5 - 9.0 + 1.0, 18.0, 18.0),
            3.0,
            3.0,
        );

        let mut paint = Paint::new();
        paint.style = Style::Fill;
        paint.color = Color::from_rgba_u8(0, 0, 0, 128).into();
        recorder.draw_rrect(&rrect, &paint);

        let icon_check = TextBlobBuilder::new(self.font.clone(), 20.0).build("\u{f00c}");

        recorder.draw_text(
            icon_check,
            Point {
                x: x + 2.0,
                y: y + h * 0.8,
            },
            Color::from_rgba_u8(255, 255, 255, 128),
        );
    }

    fn draw_button(
        &self,
        recorder: &mut PictureRecorder,
        text: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
        pre_icon: Option<&str>,
    ) {
        let corner_radius = 4.0;

        let mut paint = Paint::new();
        paint.style = Style::Fill;
        paint.color = color.clone().into();

        let rrect = RRect::from_rect_xy(
            Rect::from_xywh(x + 1.0, y + 1.0, w - 2.0, h - 2.0),
            corner_radius,
            corner_radius,
        );

        recorder.draw_rrect(&rrect, &paint);

        paint.color = LinearGradient::new(Point { x, y }, Point { x, y: y + h })
            .with_colors(vec![
                Color::from_rgba_u8(255, 255, 255, 32),
                Color::from_rgba_u8(0, 0, 0, 132),
            ])
            .into();

        recorder.draw_rrect(&rrect, &paint);

        paint.color = Color::from_rgba_u8(0, 0, 0, 48).into();
        paint.style = Stroke::new().with_width(1.0).into();

        let rrect = RRect::from_rect_xy(
            Rect::from_xywh(x + 0.5, y + 0.5, w - 1.0, h - 1.0),
            corner_radius - 0.5,
            corner_radius - 0.5,
        );

        recorder.draw_rrect(&rrect, &paint);

        let text_blob = TextBlobBuilder::new(self.font.clone(), 17.0).build(text);

        let tw = text_blob.width;
        let mut iw = 0.0;

        if let Some(icon) = pre_icon {
            let icon_blob = TextBlobBuilder::new(self.font.clone(), h * 0.8).build(icon);
            iw = icon_blob.width;
            recorder.draw_text(
                icon_blob,
                Point {
                    x: x + w * 0.5 - tw * 0.5 - iw - 4.0,
                    y: y + h * 0.75,
                },
                Color::from_rgba_u8(255, 255, 255, 128),
            );
        }

        recorder.draw_text(
            text_blob.clone(),
            Point {
                x: x + w * 0.5 - tw * 0.5 + iw * 0.25,
                y: y + h * 0.7 - 1.0,
            },
            Color::from_rgba_u8(0, 0, 0, 160),
        );

        recorder.draw_text(
            text_blob,
            Point {
                x: x + w * 0.5 - tw * 0.5 + iw * 0.25,
                y: y + h * 0.7,
            },
            Color::white().with_alpha(160.0 / 255.0),
        );
    }

    fn draw_slider(
        &self,
        recorder: &mut PictureRecorder,
        pos: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let cy = y + (h * 0.5).floor();
        let kr = (h * 0.25).floor();

        let mut paint = Paint::new();
        paint.color = Color::from_rgba_u8(0, 0, 0, 64).into();

        let rrect = RRect::from_rect_xy(Rect::from_xywh(x, cy - 2.0, w, 4.0), 2.0, 2.0);
        recorder.draw_rrect(&rrect, &paint);

        // Knob Shadow
        paint.color = RadialGradient::new(
            Point {
                x: x + pos * w,
                y: cy + 1.0,
            },
            kr + 3.0,
        )
        .with_colors(vec![
            Color::from_rgba_u8(0, 0, 0, 64),
            Color::from_rgba_u8(0, 0, 0, 0),
        ])
        .into();
        {
            let path = Path::new()
                .add_rect(&Rect::from_xywh(
                    x + pos * w - kr - 5.0,
                    cy - kr - 5.0,
                    kr * 2.0 + 5.0 + 5.0,
                    kr * 2.0 + 5.0 + 5.0 + 3.0,
                ))
                .add_circle_dir(x + pos * w, cy, kr, PathDirection::CounterClockwise);

            recorder.draw_path(path, &paint);
        }

        // knob
        {
            let knob = Path::new().add_circle(x + pos * w, cy, kr - 1.0);
            paint.color = Color::from_rgba_u8(40, 43, 48, 255).into();

            recorder.draw_path(knob.clone(), &paint);

            paint.color = LinearGradient::new(Point { x, y: cy - kr }, Point { x, y: cy + kr })
                .with_colors(vec![
                    Color::from_rgba_u8(255, 255, 255, 16),
                    Color::from_rgba_u8(0, 0, 0, 16),
                ])
                .into();

            recorder.draw_path(knob.clone(), &paint);
        }

        paint.color = Color::from_rgba_u8(0, 0, 0, 92).into();
        paint.style = Stroke::new().with_width(1.0).into();

        let circle = Path::new().add_circle(x + pos * w, cy, kr - 0.5);
        recorder.draw_path(circle, &paint);
    }

    fn render(&self) -> Picture {
        let current = time::Instant::now();

        let delta = (current - self.begin_time).as_secs_f64();

        let mut recorder = PictureRecorder::new();

        self.draw_graph(
            &mut recorder,
            0.0,
            self.height / 2.0,
            self.width,
            self.height / 2.0,
            delta,
        );
        self.draw_eyes(
            &mut recorder,
            self.width - 250.0,
            50.0,
            150.0,
            100.0,
            self.mouse_pos.0,
            self.mouse_pos.1,
            delta,
        );

        self.draw_paragraph(&mut recorder, self.width - 450.0, 180.0);

        self.draw_color_wheel(
            &mut recorder,
            self.width - 300.0,
            self.height - 300.0,
            250.0,
            250.0,
            delta,
        );

        self.draw_lines(&mut recorder, 120.0, self.height - 50.0, 600.0, delta);
        self.draw_widths(&mut recorder, 10.0, 50.0, 30.0);
        self.draw_caps(&mut recorder, 10.0, 300.0, 30.0);
        self.draw_scissor(&mut recorder, 50.0, self.height - 80.0, delta);

        self.draw_window(&mut recorder, "Widgets 'n Stuff", 50.0, 50.0, 300.0, 400.0);

        let x = 60.0;
        let mut y = 95.0;

        self.draw_search_box(&mut recorder, "Search", x, y, 280.0, 25.0);

        y += 40.0;

        self.draw_drop_down(&mut recorder, "Effects", x, y, 280.0, 28.0);

        y += 45.0;

        self.draw_label(&mut recorder, "Login", x, y, 20.0);

        y += 25.0;
        self.draw_edit_box(&mut recorder, "Email", x, y, 280.0, 28.0);

        y += 35.0;
        self.draw_edit_box(&mut recorder, "Password", x, y, 280.0, 28.0);

        y += 38.0;
        self.draw_check_box(&mut recorder, "Remember me", x, y, 28.0);
        self.draw_button(
            &mut recorder,
            "Sign in",
            x + 138.0,
            y,
            140.0,
            28.0,
            Color::from_rgba_u8(0, 96, 128, 255),
            Some("\u{f0a9}"),
        );

        y += 45.0;
        self.draw_label(&mut recorder, "Diameter", x, y, 20.0);
        y += 25.0;
        self.draw_edit_box_num(&mut recorder, "123.00", "px", x + 180.0, y, 100.0, 28.0);
        self.draw_slider(&mut recorder, 0.4, x, y, 170.0, 28.0);

        y += 55.0;
        self.draw_button(
            &mut recorder,
            "Delete",
            x,
            y,
            160.0,
            28.0,
            Color::from_rgba_u8(128, 16, 8, 255),
            Some("\u{fafb}"),
        );
        self.draw_button(
            &mut recorder,
            "Cancel",
            x + 170.0,
            y,
            110.0,
            28.0,
            Color::transparent(),
            None,
        );

        return recorder.finish_record();
    }
}

impl common::Renderer for NanovgRender {
    fn on_init(
        &mut self,
        _format: wgpu::TextureFormat,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.context = Some(GPUContext::new(device));
    }

    fn on_render(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) {
        let text = surface
            .get_current_texture()
            .expect("can not acquire texture");

        let mut surface = GPUSurface::new(&text.texture, self.width, self.height, true, device);

        let picture = self.render();

        surface.replay(&picture);

        surface.flush(
            self.context.as_mut().unwrap(),
            device,
            queue,
            Some(wgpu::Color {
                r: 0.3,
                g: 0.3,
                b: 0.32,
                a: 1.0,
            }),
        );

        text.present();
    }

    fn on_mouse_move(&mut self, x: f32, y: f32) {
        self.mouse_pos = (x, y);
    }
}

fn main() {
    let width = 1000.0;
    let height = 600.0;
    let app = common::App::new("NanoVG Frame Example", width as u32, height as u32, true);

    app.run(NanovgRender::new(width, height));
}
