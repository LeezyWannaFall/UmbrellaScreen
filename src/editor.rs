use gtk::{cairo, gdk, prelude::*};
use std::{cell::RefCell, rc::Rc};

fn physical_shortcut(keycode: u32, shift: bool) -> Option<&'static str> {
    match keycode {
        39 => Some("save"), // physical S
        54 => Some("copy"), // physical C
        52 if shift => Some("redo"),
        52 => Some("undo"), // physical Z
        _ => None,
    }
}

fn icon(name: &str) -> gtk::DrawingArea {
    let name = name.to_owned();
    let area = gtk::DrawingArea::builder()
        .content_width(20)
        .content_height(20)
        .build();
    area.set_draw_func(move |_, c, _, _| {
        c.set_source_rgb(0.88, 0.91, 0.96);
        c.set_line_width(1.6);
        c.set_line_cap(cairo::LineCap::Round);
        c.set_line_join(cairo::LineJoin::Round);
        match name.as_str() {
            "Выбор" => {
                c.move_to(4., 2.);
                c.line_to(16., 11.);
                c.line_to(10., 12.);
                c.line_to(7., 18.);
                c.close_path();
            }
            "Перо" => {
                c.move_to(4., 16.);
                c.line_to(6., 11.);
                c.line_to(14., 3.);
                c.line_to(17., 6.);
                c.line_to(9., 14.);
                c.close_path();
            }
            "Линия" => {
                c.move_to(4., 16.);
                c.line_to(16., 4.);
            }
            "Размытие" => {
                for y in [5., 10., 15.] {
                    for x in [5., 10., 15.] {
                        c.arc(x, y, 1., 0., std::f64::consts::TAU);
                        let _ = c.fill();
                    }
                }
            }
            "Стрелка" => {
                c.move_to(4., 16.);
                c.line_to(16., 4.);
                c.move_to(8., 4.);
                c.line_to(16., 4.);
                c.line_to(16., 12.);
            }
            "Прямоугольник" => c.rectangle(3., 4., 14., 12.),
            "Эллипс" => c.arc(10., 10., 7., 0., std::f64::consts::TAU),
            "Текст" => {
                c.move_to(4., 4.);
                c.line_to(16., 4.);
                c.move_to(10., 4.);
                c.line_to(10., 17.);
                c.move_to(7., 17.);
                c.line_to(13., 17.);
            }
            "undo" | "redo" => {
                if name == "redo" {
                    c.translate(20., 0.);
                    c.scale(-1., 1.);
                }
                c.move_to(7., 4.);
                c.line_to(3., 8.);
                c.line_to(7., 12.);
                c.move_to(3., 8.);
                c.line_to(12., 8.);
                c.curve_to(19., 8., 19., 17., 11., 17.);
            }
            "save" => {
                c.rectangle(4., 3., 12., 14.);
                c.rectangle(7., 3., 6., 5.);
                c.rectangle(7., 12., 6., 5.);
            }
            "copy" => {
                c.rectangle(7., 7., 10., 10.);
                c.move_to(13., 5.);
                c.line_to(13., 3.);
                c.line_to(3., 3.);
                c.line_to(3., 13.);
                c.line_to(5., 13.);
            }
            "done" => {
                c.set_source_rgb(0., 0., 0.);
                c.move_to(4., 10.);
                c.line_to(8., 14.);
                c.line_to(16., 5.);
            }
            "width" => {
                for (y, w) in [(5., 1.), (10., 2.), (16., 3.)] {
                    c.set_line_width(w);
                    c.move_to(4., y);
                    c.line_to(16., y);
                    let _ = c.stroke();
                }
            }
            "color" => {
                for (x, y, r, g, b) in [
                    (6., 6., 1., 0.3, 0.3),
                    (14., 6., 1., 0.8, 0.2),
                    (6., 14., 0.3, 0.8, 0.5),
                    (14., 14., 0.4, 0.6, 1.),
                ] {
                    c.set_source_rgb(r, g, b);
                    c.arc(x, y, 3., 0., std::f64::consts::TAU);
                    let _ = c.fill();
                }
            }
            _ => {
                for (x, y, dx, dy) in [
                    (3., 3., 1., 1.),
                    (17., 3., -1., 1.),
                    (3., 17., 1., -1.),
                    (17., 17., -1., -1.),
                ] {
                    c.move_to(x + 4. * dx, y);
                    c.line_to(x, y);
                    c.line_to(x, y + 4. * dy);
                }
            }
        }
        let _ = c.stroke();
    });
    area
}
fn icon_button(name: &str, tooltip: &str) -> gtk::Button {
    let b = gtk::Button::new();
    b.set_child(Some(&icon(name)));
    b.set_tooltip_text(Some(tooltip));
    b.set_focus_on_click(false);
    b
}

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Select,
    Pen,
    Arrow,
    Line,
    Blur,
    Rectangle,
    Ellipse,
    Text,
}

#[derive(Clone)]
struct Annotation {
    tool: Tool,
    start: (f64, f64),
    end: (f64, f64),
    color: (f64, f64, f64, f64),
    width: f64,
    text: String,
    points: Vec<(f64, f64)>,
    matrix: [f64; 6],
}

type HistorySnapshot<T> = (Vec<T>, Option<(f64, f64, f64, f64)>);

#[derive(Default)]
struct History<T> {
    items: Vec<T>,
    redo: Vec<HistorySnapshot<T>>,
    undo: Vec<HistorySnapshot<T>>,
    region: Option<(f64, f64, f64, f64)>,
}
impl<T: Clone> History<T> {
    fn checkpoint(&mut self) {
        self.undo.push((self.items.clone(), self.region));
        self.redo.clear();
    }
    fn push(&mut self, item: T) {
        self.checkpoint();
        self.items.push(item);
    }
    fn undo(&mut self) {
        if let Some((items, region)) = self.undo.pop() {
            self.redo.push((
                std::mem::replace(&mut self.items, items),
                std::mem::replace(&mut self.region, region),
            ));
        }
    }
    fn redo(&mut self) {
        if let Some((items, region)) = self.redo.pop() {
            self.undo.push((
                std::mem::replace(&mut self.items, items),
                std::mem::replace(&mut self.region, region),
            ));
        }
    }
}
struct TextGeometry {
    path: cairo::Path,
    bounds: (f64, f64, f64, f64),
}

impl Annotation {
    fn text_geometry(&self) -> Result<TextGeometry, String> {
        // Build glyph outlines in image coordinates, independent of preview/object transforms.
        let surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, 1, 1).map_err(|e| e.to_string())?;
        let cr = cairo::Context::new(&surface).map_err(|e| e.to_string())?;
        let layout = pangocairo::functions::create_layout(&cr);
        let mut font = gtk::pango::FontDescription::from_string("Sans");
        font.set_absolute_size((self.width * 6. + 12.) * gtk::pango::SCALE as f64);
        layout.set_font_description(Some(&font));
        layout.set_text(&self.text);
        cr.move_to(self.start.0, self.start.1);
        pangocairo::functions::layout_path(&cr, &layout);
        let (x1, y1, x2, y2) = cr.path_extents().map_err(|e| e.to_string())?;
        let bounds = (x1, y1, (x2 - x1).max(1.), (y2 - y1).max(1.));
        Ok(TextGeometry {
            path: cr.copy_path().map_err(|e| e.to_string())?,
            bounds,
        })
    }

    fn raw_bounds(&self) -> (f64, f64, f64, f64) {
        if self.tool == Tool::Blur {
            return (
                self.start.0.min(self.end.0),
                self.start.1.min(self.end.1),
                (self.end.0 - self.start.0).abs(),
                (self.end.1 - self.start.1).abs(),
            );
        }
        if self.tool == Tool::Text {
            return self.text_geometry().expect("Cairo text geometry").bounds;
        }
        let mut min = (self.start.0.min(self.end.0), self.start.1.min(self.end.1));
        let mut max = (self.start.0.max(self.end.0), self.start.1.max(self.end.1));
        for &(x, y) in &self.points {
            min.0 = min.0.min(x);
            min.1 = min.1.min(y);
            max.0 = max.0.max(x);
            max.1 = max.1.max(y);
        }
        (
            min.0 - 4.,
            min.1 - 4.,
            max.0 - min.0 + 8.,
            max.1 - min.1 + 8.,
        )
    }
    fn bounds(&self) -> (f64, f64, f64, f64) {
        let (x, y, w, h) = self.raw_bounds();
        let m = self.matrix;
        let points = [(x, y), (x + w, y), (x, y + h), (x + w, y + h)]
            .map(|(x, y)| (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]));
        let minx = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let miny = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let maxx = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let maxy = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        (minx, miny, maxx - minx, maxy - miny)
    }
    fn transformed(&self, t: [f64; 6]) -> Self {
        let mut a = self.clone();
        let m = self.matrix;
        a.matrix = [
            t[0] * m[0] + t[2] * m[1],
            t[1] * m[0] + t[3] * m[1],
            t[0] * m[2] + t[2] * m[3],
            t[1] * m[2] + t[3] * m[3],
            t[0] * m[4] + t[2] * m[5] + t[4],
            t[1] * m[4] + t[3] * m[5] + t[5],
        ];
        a
    }
    fn shifted(&self, dx: f64, dy: f64) -> Self {
        self.transformed([1., 0., 0., 1., dx, dy])
    }
    fn handles(&self, scale: f64) -> [(f64, f64); 9] {
        let (x, y, w, h) = self.raw_bounds();
        let m = self.matrix;
        let vertical_scale = m[2].hypot(m[3]).max(0.001);
        [
            (x, y),
            (x + w / 2., y),
            (x + w, y),
            (x + w, y + h / 2.),
            (x + w, y + h),
            (x + w / 2., y + h),
            (x, y + h),
            (x, y + h / 2.),
            (x + w / 2., y - 24. / (scale * vertical_scale)),
        ]
        .map(|(x, y)| (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]))
    }
}

#[derive(Clone, Copy)]
struct Transform {
    scale: f64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
impl Transform {
    fn fit(canvas: (f64, f64), image: (f64, f64)) -> Self {
        let scale = (canvas.0 / image.0).min(canvas.1 / image.1).max(0.001);
        Self {
            scale,
            x: (canvas.0 - image.0 * scale) / 2.,
            y: (canvas.1 - image.1 * scale) / 2.,
            width: image.0,
            height: image.1,
        }
    }
    fn point(self, x: f64, y: f64, clamp: bool) -> Option<(f64, f64)> {
        let (x, y) = ((x - self.x) / self.scale, (y - self.y) / self.scale);
        if !clamp && (x < 0. || y < 0. || x > self.width || y > self.height) {
            return None;
        }
        Some((x.clamp(0., self.width), y.clamp(0., self.height)))
    }
}

struct State {
    image: Option<cairo::ImageSurface>,
    blurred: Option<cairo::ImageSurface>,
    selection: Option<(f64, f64, f64, f64)>,
    selecting: bool,
    anchor: Option<(f64, f64)>,
    history: History<Annotation>,
    selected: Option<usize>,
    moving: Option<(usize, Annotation)>,
    handle: Option<usize>,
    region_drag: Option<(f64, f64, f64, f64)>,
    draft: Option<Annotation>,
    tool: Tool,
    color: (f64, f64, f64, f64),
    width: f64,
}
impl State {
    fn transform(&self, area: &crate::canvas::Canvas) -> Option<Transform> {
        self.image.as_ref().map(|i| {
            Transform::fit(
                (area.width() as f64, area.height() as f64),
                (i.width() as f64, i.height() as f64),
            )
        })
    }
    fn render(&self, cr: &cairo::Context, preview: bool) -> Result<(), String> {
        let image = self.image.as_ref().ok_or("Сначала сделайте скриншот")?;
        cr.save().map_err(|e| e.to_string())?;
        cr.rectangle(0., 0., image.width() as f64, image.height() as f64);
        cr.clip();
        cr.set_source_surface(image, 0., 0.)
            .map_err(|e| e.to_string())?;
        cr.paint().map_err(|e| e.to_string())?;
        if let Some((x, y, w, h)) = self.selection {
            cr.rectangle(x, y, w, h);
            cr.clip();
        }
        for item in &self.history.items {
            draw_annotation(cr, item, self.blurred.as_ref())?;
        }
        if preview && let Some(item) = &self.draft {
            draw_annotation(cr, item, self.blurred.as_ref())?;
        }
        cr.restore().map_err(|e| e.to_string())
    }
    fn png(&self) -> Result<Vec<u8>, String> {
        let image = self.image.as_ref().ok_or("Сначала сделайте скриншот")?;
        if self.blurred.is_none() && self.history.items.iter().any(|a| a.tool == Tool::Blur) {
            return Err("Размытие ещё подготавливается. Повторите сохранение через секунду".into());
        }
        let (x, y, w, h) =
            self.selection
                .unwrap_or((0., 0., image.width() as f64, image.height() as f64));
        if self.selecting {
            return Err("Сначала выделите область".into());
        }
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, w as i32, h as i32)
            .map_err(|e| e.to_string())?;
        let cr = cairo::Context::new(&surface).map_err(|e| e.to_string())?;
        cr.translate(-x, -y);
        self.render(&cr, false)?;
        let mut bytes = Vec::new();
        surface
            .write_to_png(&mut bytes)
            .map_err(|e| e.to_string())?;
        Ok(bytes)
    }
}

fn draw_annotation(
    cr: &cairo::Context,
    a: &Annotation,
    blurred: Option<&cairo::ImageSurface>,
) -> Result<(), String> {
    cr.save().map_err(|e| e.to_string())?;
    cr.set_source_rgba(a.color.0, a.color.1, a.color.2, a.color.3);
    cr.set_line_width(a.width);
    let world = cr.matrix();
    let m = a.matrix;
    cr.transform(cairo::Matrix::new(m[0], m[1], m[2], m[3], m[4], m[5]));
    cr.set_line_cap(cairo::LineCap::Round);
    let (x, y) = a.start;
    let (ex, ey) = a.end;
    match a.tool {
        Tool::Select => {}
        Tool::Line => {
            cr.move_to(x, y);
            cr.line_to(ex, ey);
            cr.stroke().map_err(|e| e.to_string())?;
        }
        Tool::Blur => {
            if let Some(image) = blurred {
                cr.rectangle(x.min(ex), y.min(ey), (ex - x).abs(), (ey - y).abs());
                cr.clip();
                cr.set_matrix(world);
                cr.set_source_surface(image, 0., 0.)
                    .map_err(|e| e.to_string())?;
                cr.paint().map_err(|e| e.to_string())?;
            }
        }
        Tool::Pen => {
            cr.set_line_join(cairo::LineJoin::Round);
            cr.move_to(x, y);
            let mut previous = (x, y);
            // Quadratic curves through segment midpoints soften turns without overshoot.
            // Cairo accepts cubic curves, so convert each quadratic control point.
            for points in a.points.windows(2) {
                let control = points[0];
                let next = points[1];
                let midpoint = ((control.0 + next.0) / 2., (control.1 + next.1) / 2.);
                cr.curve_to(
                    previous.0 + (control.0 - previous.0) * 2. / 3.,
                    previous.1 + (control.1 - previous.1) * 2. / 3.,
                    midpoint.0 + (control.0 - midpoint.0) * 2. / 3.,
                    midpoint.1 + (control.1 - midpoint.1) * 2. / 3.,
                    midpoint.0,
                    midpoint.1,
                );
                previous = midpoint;
            }
            if let Some(&(px, py)) = a.points.last() {
                cr.line_to(px, py);
            }
            cr.stroke().map_err(|e| e.to_string())?;
        }
        Tool::Arrow => {
            cr.move_to(x, y);
            cr.line_to(ex, ey);
            let angle = (ey - y).atan2(ex - x);
            let head = (a.width * 4.).max(12.).min((ex - x).hypot(ey - y) * 0.5);
            for offset in [-0.5_f64, 0.5] {
                cr.move_to(ex, ey);
                cr.line_to(
                    ex - head * (angle + offset).cos(),
                    ey - head * (angle + offset).sin(),
                );
            }
            cr.stroke().map_err(|e| e.to_string())?;
        }
        Tool::Rectangle => {
            cr.rectangle(x.min(ex), y.min(ey), (ex - x).abs(), (ey - y).abs());
            cr.stroke().map_err(|e| e.to_string())?;
        }
        Tool::Ellipse => {
            if (ex - x).abs() > 0.1 && (ey - y).abs() > 0.1 {
                cr.save().map_err(|e| e.to_string())?;
                cr.translate((x + ex) / 2., (y + ey) / 2.);
                cr.scale((ex - x).abs() / 2., (ey - y).abs() / 2.);
                cr.arc(0., 0., 1., 0., std::f64::consts::TAU);
                cr.restore().map_err(|e| e.to_string())?;
                cr.stroke().map_err(|e| e.to_string())?;
            }
        }
        Tool::Text => {
            let geometry = a.text_geometry()?;
            cr.new_path();
            cr.append_path(&geometry.path);
            cr.fill().map_err(|e| e.to_string())?;
        }
    }
    cr.restore().map_err(|e| e.to_string())
}

#[derive(Clone)]
pub struct Editor {
    pub root: gtk::Box,
    pub status: gtk::Label,
    background: gtk::Picture,
    area: crate::canvas::Canvas,
    state: Rc<RefCell<State>>,
    busy: Rc<std::cell::Cell<bool>>,
}

impl Editor {
    pub fn new(window: &gtk::ApplicationWindow) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let area = crate::canvas::Canvas::new();
        area.set_hexpand(true);
        area.set_vexpand(true);
        let status = gtk::Label::new(Some("Выберите режим захвата"));
        status.set_wrap(true);
        let editor = Self {
            root,
            status,
            area,
            background: gtk::Picture::builder()
                .can_shrink(true)
                .hexpand(true)
                .vexpand(true)
                .build(),
            busy: Rc::new(std::cell::Cell::new(false)),
            state: Rc::new(RefCell::new(State {
                image: None,
                blurred: None,
                selection: None,
                selecting: true,
                anchor: None,
                history: History {
                    items: vec![],
                    redo: vec![],
                    undo: vec![],
                    region: None,
                },
                selected: None,
                moving: None,
                handle: None,
                region_drag: None,
                draft: None,
                tool: Tool::Select,
                color: (1., 0.25, 0.2, 1.),
                width: 4.,
            })),
        };
        let mut group: Option<gtk::ToggleButton> = None;
        for (name, tool) in [
            ("Выбор", Tool::Select),
            ("Перо", Tool::Pen),
            ("Стрелка", Tool::Arrow),
            ("Линия", Tool::Line),
            ("Размытие", Tool::Blur),
            ("Прямоугольник", Tool::Rectangle),
            ("Эллипс", Tool::Ellipse),
            ("Текст", Tool::Text),
        ] {
            let button = gtk::ToggleButton::new();
            button.set_child(Some(&icon(name)));
            button.set_tooltip_text(Some(name));
            button.set_focus_on_click(false);
            if let Some(first) = &group {
                button.set_group(Some(first));
            } else {
                button.set_active(true);
                group = Some(button.clone());
            }
            let state = editor.state.clone();
            button.connect_toggled(move |b| {
                if b.is_active() {
                    let mut state = state.borrow_mut();
                    state.tool = tool;
                }
            });
            toolbar.append(&button);
        }
        let colors = gtk::MenuButton::new();
        colors.set_property("child", icon("color"));
        colors.set_tooltip_text(Some("Цвет маркера"));
        let palette = gtk::Popover::new();
        let grid = gtk::Grid::builder()
            .row_spacing(6)
            .column_spacing(6)
            .build();
        for (i, (name, color)) in [
            ("Красный", (1., 0.25, 0.2, 1.)),
            ("Жёлтый", (1., 0.85, 0.1, 1.)),
            ("Зелёный", (0.2, 0.85, 0.4, 1.)),
            ("Синий", (0.2, 0.5, 1., 1.)),
            ("Белый", (1., 1., 1., 1.)),
            ("Чёрный", (0., 0., 0., 1.)),
        ]
        .into_iter()
        .enumerate()
        {
            let swatch = gtk::DrawingArea::builder()
                .content_width(28)
                .content_height(28)
                .build();
            swatch.set_draw_func(move |_, cr, width, height| {
                cr.set_source_rgba(color.0, color.1, color.2, color.3);
                cr.arc(
                    width as f64 / 2.,
                    height as f64 / 2.,
                    10.,
                    0.,
                    std::f64::consts::TAU,
                );
                let _ = cr.fill_preserve();
                cr.set_source_rgb(0.5, 0.5, 0.5);
                cr.set_line_width(1.);
                let _ = cr.stroke();
            });
            let button = gtk::Button::new();
            button.set_child(Some(&swatch));
            button.set_tooltip_text(Some(name));
            let state = editor.state.clone();
            let pop = palette.clone();
            button.connect_clicked(move |_| {
                state.borrow_mut().color = color;
                pop.popdown();
            });
            grid.attach(&button, (i % 3) as i32, (i / 3) as i32, 1, 1);
        }
        palette.set_child(Some(&grid));
        colors.set_popover(Some(&palette));
        toolbar.append(&colors);
        let width = gtk::MenuButton::new();
        width.set_property("child", icon("width"));
        width.set_tooltip_text(Some("Толщина маркера"));
        let sizes = gtk::Popover::new();
        let list = gtk::Box::new(gtk::Orientation::Vertical, 4);
        for size in [1., 2., 4., 6., 8., 12., 16.] {
            let b = gtk::Button::with_label(&format!("{size} px"));
            let state = editor.state.clone();
            let pop = sizes.clone();
            b.connect_clicked(move |_| {
                state.borrow_mut().width = size;
                pop.popdown();
            });
            list.append(&b);
        }
        sizes.set_child(Some(&list));
        width.set_popover(Some(&sizes));
        toolbar.append(&width);

        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        for (name, action) in [
            ("Отменить", "undo"),
            ("Повторить", "redo"),
            ("Сохранить PNG", "save"),
            ("Копировать", "copy"),
        ] {
            let button = icon_button(
                action,
                &format!(
                    "{} ({})",
                    name,
                    match action {
                        "save" => "Ctrl+S",
                        "copy" => "Ctrl+C",
                        "undo" => "Ctrl+Z",
                        _ => "Ctrl+Shift+Z",
                    }
                ),
            );
            let e = editor.clone();
            button.connect_clicked(move |_| e.action(action));
            actions.append(&button);
        }
        let fullscreen = icon_button("fullscreen", "Выбрать весь экран");
        let e = editor.clone();
        fullscreen.connect_clicked(move |_| {
            let mut state = e.state.borrow_mut();
            if let Some(image) = &state.image {
                state.selection = Some((0., 0., image.width() as f64, image.height() as f64));
                state.selecting = false;
            }
            drop(state);
            e.area.queue_draw();
        });
        actions.append(&fullscreen);
        let reselect = icon_button("select", "Выделить заново");
        toolbar.append(&reselect);
        let done = icon_button("done", "Сохранить и скопировать — Enter");
        done.add_css_class("suggested-action");
        let e = editor.clone();
        done.connect_clicked(move |_| e.action("savecopy"));
        actions.append(&done);
        let overlay = gtk::Overlay::new();
        // Reserve space so the floating controls never obscure the image.

        overlay.set_child(Some(&editor.background));
        editor.area.set_hexpand(true);
        editor.area.set_vexpand(true);
        overlay.add_overlay(&editor.area);
        let top = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::SlideDown)
            .transition_duration(300)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .build();
        top.set_margin_top(16);
        top.set_child(Some(&toolbar));
        let bottom_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        bottom_box.append(&actions);
        overlay.add_overlay(&editor.status);
        editor.status.set_halign(gtk::Align::Center);
        editor.status.set_valign(gtk::Align::End);
        editor.status.set_margin_bottom(68);
        editor.status.set_visible(false);
        let bottom = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::SlideUp)
            .transition_duration(350)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::End)
            .build();
        bottom.set_margin_bottom(16);
        bottom.set_child(Some(&bottom_box));
        for widget in [&toolbar, &bottom_box] {
            widget.add_css_class("editor-panel");
        }
        overlay.add_overlay(&top);
        overlay.add_overlay(&bottom);
        editor.root.append(&overlay);
        let e = editor.clone();
        let t = top.clone();
        let b = bottom.clone();
        reselect.connect_clicked(move |_| {
            let mut state = e.state.borrow_mut();
            state.selecting = true;
            state.selection = None;
            state.anchor = None;
            state.selected = None;
            state.moving = None;
            state.history.undo.clear();
            state.history.items.clear();
            state.history.redo.clear();
            drop(state);
            t.set_reveal_child(false);
            b.set_reveal_child(false);
            e.area.queue_draw();
        });
        let css = gtk::CssProvider::new();
        css.load_from_data(".editor-panel { background: alpha(#171a22, 0.96); border: 1px solid alpha(#aab8d4,0.20); border-radius: 14px; padding: 5px; box-shadow: 0 4px 16px alpha(black,0.35); } .editor-panel button { background: transparent; background-image: none; border: none; box-shadow: none; min-width: 32px; min-height: 32px; padding: 0; margin: 0; border-radius: 9px; } .editor-panel button:hover { background: alpha(#aab8d4,0.15); } .editor-panel button:checked { background: #41516f; } .editor-panel button.suggested-action { background: #396a62; } window { background: #12141b; }");
        gtk::style_context_add_provider_for_display(
            &gtk::prelude::WidgetExt::display(window),
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        let state = editor.state.clone();
        let coordinate_layout = editor.area.create_pango_layout(None);
        editor
            .area
            .set_draw_func(move |_, snapshot, width, height| {
                let state = state.borrow();
                if let Some(image) = &state.image {
                    let t = Transform::fit(
                        (width as f64, height as f64),
                        (image.width() as f64, image.height() as f64),
                    );
                    if !state.history.items.is_empty()
                        || state.draft.is_some()
                        || state.selected.is_some()
                    {
                        let cr = snapshot.append_cairo(&gtk::graphene::Rect::new(
                            0.,
                            0.,
                            width as f32,
                            height as f32,
                        ));
                        cr.translate(t.x, t.y);
                        cr.scale(t.scale, t.scale);
                        let _ = cr.save();
                        if let Some((x, y, w, h)) = state.selection {
                            cr.rectangle(x, y, w, h);
                            cr.clip();
                        }
                        for item in &state.history.items {
                            let _ = draw_annotation(&cr, item, state.blurred.as_ref());
                        }
                        if let Some(item) = &state.draft {
                            let _ = draw_annotation(&cr, item, state.blurred.as_ref());
                        }
                        let _ = cr.restore();
                        if let Some(a) = state.selected.and_then(|i| state.history.items.get(i)) {
                            let handles = a.handles(t.scale);
                            // Cairo paths survive save/restore, including text outlines.
                            cr.new_path();
                            cr.set_line_join(cairo::LineJoin::Miter);
                            cr.set_line_cap(cairo::LineCap::Butt);
                            cr.move_to(handles[0].0, handles[0].1);
                            for index in [2, 4, 6] {
                                cr.line_to(handles[index].0, handles[index].1);
                            }
                            cr.close_path();
                            cr.set_source_rgb(0.65, 0.70, 0.88);
                            cr.set_line_width(1. / t.scale);
                            let _ = cr.stroke();
                            cr.move_to(handles[1].0, handles[1].1);
                            cr.line_to(handles[8].0, handles[8].1);
                            let _ = cr.stroke();
                            for (hx, hy) in handles {
                                cr.arc(hx, hy, 3.5 / t.scale, 0., std::f64::consts::TAU);
                                cr.set_source_rgb(0.12, 0.12, 0.14);
                                let _ = cr.fill_preserve();
                                cr.set_source_rgb(0.78, 0.79, 0.88);
                                let _ = cr.stroke();
                            }
                        }
                    }
                    let color = gdk::RGBA::new(0., 0., 0., 0.6);
                    let rect = |x: f64, y: f64, w: f64, h: f64| {
                        gtk::graphene::Rect::new(
                            x as f32,
                            y as f32,
                            w.max(0.) as f32,
                            h.max(0.) as f32,
                        )
                    };
                    if let Some((ix, iy, iw, ih)) = state.selection {
                        let x = t.x + ix * t.scale;
                        let y = t.y + iy * t.scale;
                        let w = iw * t.scale;
                        let h = ih * t.scale;
                        for r in [
                            rect(0., 0., width as f64, y),
                            rect(0., y, x, h),
                            rect(x + w, y, width as f64 - x - w, h),
                            rect(0., y + h, width as f64, height as f64 - y - h),
                        ] {
                            snapshot.append_color(&color, &r);
                        }
                        let border = gdk::RGBA::new(0.75, 0.82, 1., 1.);
                        for r in [
                            rect(x, y, w, 1.),
                            rect(x, y + h - 1., w, 1.),
                            rect(x, y, 1., h),
                            rect(x + w - 1., y, 1., h),
                        ] {
                            snapshot.append_color(&border, &r);
                        }
                        if state.selecting {
                            let layout = &coordinate_layout;
                            layout.set_text(&format!("{} x {}", iw as i32, ih as i32));
                            let (_, lh) = layout.pixel_size();
                            snapshot.save();
                            snapshot.translate(&gtk::graphene::Point::new(
                                x.max(2.) as f32,
                                (y - lh as f64 - 6.).max(2.) as f32,
                            ));
                            snapshot.append_layout(layout, &border);
                            snapshot.restore();
                        }
                    } else {
                        snapshot.append_color(&color, &rect(0., 0., width as f64, height as f64));
                    }
                }
            });
        let drag = gtk::GestureDrag::new();
        drag.set_button(1);
        let e = editor.clone();
        let parent = window.downgrade();
        drag.connect_drag_begin(move |gesture, x, y| {
            let mut state = e.state.borrow_mut();
            let Some(point) = state.transform(&e.area).and_then(|t| t.point(x, y, false)) else {
                drop(state);
                gesture.set_state(gtk::EventSequenceState::Denied);
                return;
            };
            if state.selecting {
                state.anchor = Some(point);
                state.selection = Some((point.0, point.1, 0., 0.));
                return;
            }
            let tolerance = 10. / state.transform(&e.area).unwrap().scale;
            state.handle = None;
            if state.tool == Tool::Select
                && let Some(i) = state.selected
            {
                state.handle = state.history.items[i]
                    .handles(state.transform(&e.area).unwrap().scale)
                    .iter()
                    .position(|&(x, y)| {
                        (point.0 - x).abs() < tolerance && (point.1 - y).abs() < tolerance
                    });
                if state.handle.is_some() {
                    state.moving = Some((i, state.history.items[i].clone()));
                    return;
                }
            }
            if let Some((x, y, w, h)) = state.selection {
                let on_frame = point.0 >= x - tolerance
                    && point.0 <= x + w + tolerance
                    && point.1 >= y - tolerance
                    && point.1 <= y + h + tolerance
                    && ((point.0 - x).abs() < tolerance
                        || (point.0 - x - w).abs() < tolerance
                        || (point.1 - y).abs() < tolerance
                        || (point.1 - y - h).abs() < tolerance);
                if on_frame {
                    state.history.region = state.selection;
                    state.history.checkpoint();
                    state.region_drag = state.selection;
                    return;
                }
                if point.0 < x || point.1 < y || point.0 > x + w || point.1 > y + h {
                    drop(state);
                    gesture.set_state(gtk::EventSequenceState::Denied);
                    return;
                }
            }
            if state.tool == Tool::Select {
                state.selected = state.history.items.iter().rposition(|a| {
                    let (x, y, w, h) = a.raw_bounds();
                    let m = a.matrix;
                    let determinant = m[0] * m[3] - m[1] * m[2];
                    let dx = point.0 - m[4];
                    let dy = point.1 - m[5];
                    let px = (m[3] * dx - m[2] * dy) / determinant;
                    let py = (-m[1] * dx + m[0] * dy) / determinant;
                    px >= x && py >= y && px <= x + w && py <= y + h
                });
                state.moving = state.selected.map(|i| (i, state.history.items[i].clone()));
                drop(state);
                e.area.queue_draw();
                return;
            }
            let annotation = Annotation {
                tool: state.tool,
                start: point,
                end: point,
                color: state.color,
                width: state.width,
                text: String::new(),
                matrix: [1., 0., 0., 1., 0., 0.],
                points: vec![point],
            };
            if state.tool == Tool::Text {
                drop(state);
                gesture.set_state(gtk::EventSequenceState::Denied);
                if let Some(parent) = parent.upgrade() {
                    e.text_dialog(&parent, annotation);
                }
            } else {
                state.draft = Some(annotation);
            }
        });
        let e = editor.clone();
        drag.connect_drag_update(move |gesture, dx, dy| {
            let Some((x, y)) = gesture.start_point() else {
                return;
            };
            let mut state = e.state.borrow_mut();
            if let Some((sx, sy, sw, sh)) = state.region_drag {
                let t = state.transform(&e.area).unwrap();
                let image = state.image.as_ref().unwrap();
                let previous = state.selection.unwrap();
                state.selection = Some((
                    (sx + dx / t.scale)
                        .clamp(0., image.width() as f64 - sw)
                        .round(),
                    (sy + dy / t.scale)
                        .clamp(0., image.height() as f64 - sh)
                        .round(),
                    sw,
                    sh,
                ));
                let current = state.selection.unwrap();
                for a in &mut state.history.items {
                    *a = a.shifted(current.0 - previous.0, current.1 - previous.1);
                }
                state.history.region = state.selection;
                drop(state);
                e.area.queue_draw();
                return;
            }
            if state.selecting {
                if let Some(point) = state
                    .transform(&e.area)
                    .and_then(|t| t.point(x + dx, y + dy, true))
                    && let Some(start) = state.anchor
                {
                    let left = start.0.min(point.0).floor();
                    let top = start.1.min(point.1).floor();
                    state.selection = Some((
                        left,
                        top,
                        start.0.max(point.0).ceil() - left,
                        start.1.max(point.1).ceil() - top,
                    ));
                }
                drop(state);
                e.area.queue_draw();
                return;
            }
            if let Some((i, original)) = state.moving.clone() {
                if let Some(t) = state.transform(&e.area) {
                    let (x, y, w, h) = original.bounds();
                    let (sx, sy, sw, sh) = state.selection.unwrap();
                    let mut changed = if let Some(handle) = state.handle {
                        if handle == 8 {
                            let cx = x + w / 2.;
                            let cy = y + h / 2.;
                            let start = gesture.start_point().unwrap();
                            let start = t.point(start.0, start.1, true).unwrap();
                            let angle = (start.1 + dy / t.scale - cy)
                                .atan2(start.0 + dx / t.scale - cx)
                                - (start.1 - cy).atan2(start.0 - cx);
                            let (sin, cos) = angle.sin_cos();
                            original.transformed([
                                cos,
                                sin,
                                -sin,
                                cos,
                                cx - cos * cx + sin * cy,
                                cy - sin * cx - cos * cy,
                            ])
                        } else {
                            let (x, y, w, h) = original.raw_bounds();
                            let m = original.matrix;
                            let determinant = m[0] * m[3] - m[1] * m[2];
                            let local_dx = (m[3] * dx - m[2] * dy) / (determinant * t.scale);
                            let local_dy = (-m[1] * dx + m[0] * dy) / (determinant * t.scale);
                            let left = matches!(handle, 0 | 6 | 7);
                            let right = matches!(handle, 2..=4);
                            let top = matches!(handle, 0..=2);
                            let bottom = matches!(handle, 4..=6);
                            let nw = (w + if left {
                                -local_dx
                            } else if right {
                                local_dx
                            } else {
                                0.
                            })
                            .max(4.);
                            let nh = (h + if top {
                                -local_dy
                            } else if bottom {
                                local_dy
                            } else {
                                0.
                            })
                            .max(4.);
                            let ax = if left { x + w } else { x };
                            let ay = if top { y + h } else { y };
                            let mut local = original.clone();
                            local.matrix = [1., 0., 0., 1., 0., 0.];
                            local
                                .transformed([
                                    nw / w,
                                    0.,
                                    0.,
                                    nh / h,
                                    ax * (1. - nw / w),
                                    ay * (1. - nh / h),
                                ])
                                .transformed(original.matrix)
                        }
                    } else {
                        original.shifted(
                            (dx / t.scale).clamp(sx - x, (sx + sw - x - w).max(sx - x)),
                            (dy / t.scale).clamp(sy - y, (sy + sh - y - h).max(sy - y)),
                        )
                    };
                    let (_, _, cw, ch) = changed.bounds();
                    if cw > sw || ch > sh {
                        return;
                    }
                    let (bx, by, bw, bh) = changed.bounds();
                    changed = changed.shifted(
                        (sx - bx).max(0.) - (bx + bw - sx - sw).max(0.),
                        (sy - by).max(0.) - (by + bh - sy - sh).max(0.),
                    );
                    state.history.items[i] = changed;
                }
                drop(state);
                e.area.queue_draw();
                return;
            }
            let bounds = state.selection;
            if let Some(point) = state
                .transform(&e.area)
                .and_then(|t| t.point(x + dx, y + dy, true))
                .map(|p| {
                    if let Some((x, y, w, h)) = bounds {
                        (p.0.clamp(x, x + w), p.1.clamp(y, y + h))
                    } else {
                        p
                    }
                })
                && let Some(draft) = &mut state.draft
            {
                draft.end = point;
                draft.points.push(point);
            }
            drop(state);
            e.area.queue_draw();
        });
        let e = editor.clone();
        drag.connect_drag_end(move |_, _, _| {
            let mut state = e.state.borrow_mut();
            if state.region_drag.take().is_some() {
                return;
            }
            if state.selecting {
                if let Some((_, _, w, h)) = state.selection
                    && w >= 2.
                    && h >= 2.
                {
                    state.selecting = false;
                    state.history.region = state.selection;
                    top.set_reveal_child(true);
                    bottom.set_reveal_child(true);
                    e.status.set_text(&format!(
                        "{} × {} · Enter — сохранить и скопировать",
                        w as i32, h as i32
                    ));
                }
                state.anchor = None;
                drop(state);
                e.area.queue_draw();
                return;
            }
            if let Some((i, original)) = state.moving.take() {
                let changed = state.history.items[i].clone();
                state.history.items[i] = original.clone();
                if changed.matrix != original.matrix
                    || changed.start != original.start
                    || changed.end != original.end
                    || changed.width != original.width
                {
                    state.history.checkpoint();
                    state.history.items[i] = changed;
                }
                drop(state);
                e.area.queue_draw();
                return;
            }
            if let Some(draft) = state.draft.take()
                && (draft.tool != Tool::Blur
                    || ((draft.end.0 - draft.start.0).abs() >= 1.
                        && (draft.end.1 - draft.start.1).abs() >= 1.))
                && ((draft.tool == Tool::Pen && draft.points.len() > 1)
                    || (draft.end.0 - draft.start.0).hypot(draft.end.1 - draft.start.1) >= 1.)
            {
                state.history.push(draft);
            }
            drop(state);
            e.area.queue_draw();
        });
        let e = editor.clone();
        drag.connect_cancel(move |_, _| {
            let mut state = e.state.borrow_mut();
            state.draft = None;
            if let Some(region) = state.region_drag.take() {
                state.history.undo();
                state.selection = Some(region);
            }
            if let Some((i, original)) = state.moving.take() {
                state.history.items[i] = original;
            }
            drop(state);
            e.area.queue_draw();
        });
        editor.area.add_controller(drag);
        editor.area.set_cursor_from_name(Some("default"));
        let keys = gtk::EventControllerKey::new();
        let e = editor.clone();
        let parent = window.downgrade();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        keys.connect_key_pressed(move |_, key, keycode, modifiers| {
            if let Some(w) = parent.upgrade()
                && let Some(focus) = gtk::prelude::GtkWindowExt::focus(&w)
                && focus.is::<gtk::Editable>()
            {
                return gtk::glib::Propagation::Proceed;
            }
            if key == gdk::Key::Delete || key == gdk::Key::BackSpace {
                let mut state = e.state.borrow_mut();
                if let Some(i) = state.selected.take()
                    && i < state.history.items.len()
                {
                    state.history.checkpoint();
                    state.history.items.remove(i);
                }
                drop(state);
                e.area.queue_draw();
                return gtk::glib::Propagation::Stop;
            }
            if key == gdk::Key::Escape {
                if let Some(w) = parent.upgrade() {
                    w.close();
                }
                return gtk::glib::Propagation::Stop;
            }
            if key == gdk::Key::Return || key == gdk::Key::KP_Enter {
                e.action("both");
                return gtk::glib::Propagation::Stop;
            }
            if !modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                return gtk::glib::Propagation::Proceed;
            }
            // GDK Wayland supplies XKB hardware codes (evdev + 8), independent of layout.
            if let Some(w) = parent.upgrade()
                && let Some(focus) = gtk::prelude::GtkWindowExt::focus(&w)
                && focus.is::<gtk::Editable>()
            {
                return gtk::glib::Propagation::Proceed;
            }
            let Some(action) =
                physical_shortcut(keycode, modifiers.contains(gdk::ModifierType::SHIFT_MASK))
            else {
                return gtk::glib::Propagation::Proceed;
            };
            e.action(action);
            gtk::glib::Propagation::Stop
        });
        // Bubble phase lets editable widgets consume their standard shortcuts first.
        window.add_controller(keys);
        editor
    }

    fn text_dialog(&self, _parent: &gtk::ApplicationWindow, annotation: Annotation) {
        let popover = gtk::Popover::new();
        popover.set_parent(&self.area);
        if let Some(t) = self.state.borrow().transform(&self.area) {
            popover.set_pointing_to(Some(&gdk::Rectangle::new(
                (t.x + annotation.start.0 * t.scale) as i32,
                (t.y + annotation.start.1 * t.scale) as i32,
                1,
                1,
            )));
        }
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let entry = gtk::Entry::builder()
            .placeholder_text("Введите текст…")
            .width_chars(24)
            .build();
        let accept = icon_button("done", "Добавить текст — Enter");
        content.append(&entry);
        content.append(&accept);
        popover.set_child(Some(&content));
        let e = self.clone();
        let pop = popover.clone();
        let field = entry.clone();
        let commit = Rc::new(move || {
            if !field.text().trim().is_empty() {
                let mut a = annotation.clone();
                a.text = field.text().to_string();
                e.state.borrow_mut().history.push(a);
                e.area.queue_draw();
            }
            pop.popdown();
        });
        let save = commit.clone();
        accept.connect_clicked(move |_| save());
        entry.connect_activate(move |_| commit());
        // Stop Enter from also triggering screenshot export on the parent window.
        let keys = gtk::EventControllerKey::new();
        keys.connect_key_pressed(|_, key, _, _| {
            if key == gdk::Key::Return || key == gdk::Key::KP_Enter {
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
        popover.add_controller(keys);
        let area = self.area.clone();
        area.set_focusable(true);
        popover.connect_closed(move |p| {
            let p = p.clone();
            let area = area.clone();
            gtk::glib::idle_add_local_once(move || {
                p.unparent();
                area.grab_focus();
            });
        });
        popover.popup();
        entry.grab_focus();
    }

    pub fn load(&self, png: &[u8]) -> Result<(), String> {
        let image = cairo::ImageSurface::create_from_png(&mut std::io::Cursor::new(png))
            .map_err(|e| format!("Не удалось декодировать PNG: {e}"))?;
        let source = png.to_vec();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = (|| -> Result<_, String> {
                let source =
                    cairo::ImageSurface::create_from_png(&mut std::io::Cursor::new(source))
                        .map_err(|e| e.to_string())?;
                let mut blurred = blur_image(&source)?;
                let (width, height, stride) = (blurred.width(), blurred.height(), blurred.stride());
                let pixels = blurred.data().map_err(|e| e.to_string())?.to_vec();
                Ok((pixels, width, height, stride))
            })();
            let _ = sender.send(result);
        });
        let editor = self.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(30), move || match receiver
            .try_recv()
        {
            Ok(result) => {
                match result.and_then(|(pixels, width, height, stride)| {
                    cairo::ImageSurface::create_for_data(
                        pixels,
                        cairo::Format::ARgb32,
                        width,
                        height,
                        stride,
                    )
                    .map_err(|e| e.to_string())
                }) {
                    Ok(image) => editor.state.borrow_mut().blurred = Some(image),
                    Err(error) => {
                        editor
                            .status
                            .set_text(&format!("Не удалось подготовить размытие: {error}"));
                        editor.status.set_visible(true);
                    }
                }
                editor.area.queue_draw();
                gtk::glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => gtk::glib::ControlFlow::Continue,
            Err(_) => gtk::glib::ControlFlow::Break,
        });
        // Upload the immutable screenshot once. Pointer motion only redraws annotations and masks.
        let mut texture_surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, image.width(), image.height())
                .map_err(|e| e.to_string())?;
        {
            let cr = cairo::Context::new(&texture_surface).map_err(|e| e.to_string())?;
            cr.set_source_surface(&image, 0., 0.)
                .map_err(|e| e.to_string())?;
            cr.paint().map_err(|e| e.to_string())?;
        }
        let stride = texture_surface.stride() as usize;
        let pixels = texture_surface.data().map_err(|e| e.to_string())?.to_vec();
        let format = if cfg!(target_endian = "little") {
            gdk::MemoryFormat::B8g8r8a8Premultiplied
        } else {
            gdk::MemoryFormat::A8r8g8b8Premultiplied
        };
        let texture = gdk::MemoryTexture::new(
            image.width(),
            image.height(),
            format,
            &gtk::glib::Bytes::from_owned(pixels),
            stride,
        );
        self.background.set_paintable(Some(&texture));
        self.status.set_text(&format!(
            "{} × {} пикселей · снимок ещё не сохранён",
            image.width(),
            image.height()
        ));
        let mut state = self.state.borrow_mut();
        state.image = Some(image);
        state.selected = None;
        state.moving = None;
        state.history.undo.clear();
        state.history.items.clear();
        state.history.redo.clear();
        state.draft = None;
        drop(state);
        self.area.queue_draw();
        Ok(())
    }

    fn action(&self, action: &str) {
        if action == "undo" || action == "redo" {
            let mut state = self.state.borrow_mut();
            state.selected = None;
            if action == "undo" {
                state.history.undo();
            } else {
                state.history.redo();
            }
            if state.history.region.is_some() {
                state.selection = state.history.region;
            }
            drop(state);
            self.area.queue_draw();
            return;
        }
        if self.busy.get() {
            return;
        }
        let png = match self.state.borrow().png() {
            Ok(png) => png,
            Err(error) => {
                self.status.set_visible(true);
                self.status.set_text(&error);
                return;
            }
        };
        self.status.set_visible(true);
        self.busy.set(true);
        self.status.set_text("Выполняется…");
        let copy = action == "copy";
        let close_after = action == "both";
        let both = close_after || action == "savecopy";
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = if both {
                let saved = save_png(&png);
                let copied = crate::capture::clipboard(png);
                match (saved, copied) {
                    (Ok(path), Ok(())) => {
                        Ok(format!("Сохранено: {} · скопировано", path.display()))
                    }
                    (Ok(path), Err(e)) => Err(format!("Сохранено: {}. Буфер: {e}", path.display())),
                    (Err(e), Ok(())) => Err(format!("Скопировано. Сохранение: {e}")),
                    (Err(a), Err(b)) => Err(format!("{a}; {b}")),
                }
            } else if copy {
                crate::capture::clipboard(png).map(|_| "PNG скопирован в буфер обмена".to_owned())
            } else {
                save_png(&png).map(|path| format!("Сохранено: {}", path.display()))
            };
            let _ = tx.send(result);
        });
        let e = self.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(40), move || {
            match rx.try_recv() {
                Ok(result) => {
                    e.busy.set(false);
                    if close_after && result.is_ok() {
                        if let Some(app) = gtk::gio::Application::default() {
                            app.quit();
                        }
                        return gtk::glib::ControlFlow::Break;
                    }
                    e.status
                        .set_text(&result.unwrap_or_else(|error| format!("Ошибка: {error}")));
                    gtk::glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => gtk::glib::ControlFlow::Continue,
                Err(_) => {
                    e.busy.set(false);
                    e.status.set_text("Рабочий поток неожиданно завершился");
                    gtk::glib::ControlFlow::Break
                }
            }
        });
    }
}

// Separable sliding-window blur, cached once per screenshot. Work is linear in pixels.
fn blur_image(source: &cairo::ImageSurface) -> Result<cairo::ImageSurface, String> {
    let (width, height) = (source.width() as usize, source.height() as usize);
    let mut image =
        cairo::ImageSurface::create(cairo::Format::ARgb32, source.width(), source.height())
            .map_err(|e| e.to_string())?;
    {
        let cr = cairo::Context::new(&image).map_err(|e| e.to_string())?;
        cr.set_source_surface(source, 0., 0.)
            .map_err(|e| e.to_string())?;
        cr.paint().map_err(|e| e.to_string())?;
    }
    let stride = image.stride() as usize;
    {
        let mut data = image.data().map_err(|e| e.to_string())?;
        let mut scratch = vec![0_u8; data.len()];
        for _ in 0..2 {
            blur_pass(&data, &mut scratch, width, height, stride, false);
            blur_pass(&scratch, &mut data, width, height, stride, true);
        }
    }
    image.mark_dirty();
    Ok(image)
}

fn blur_pass(
    input: &[u8],
    output: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    vertical: bool,
) {
    let radius = 8_isize;
    let (lines, length, step, line_step) = if vertical {
        (width, height, stride, 4)
    } else {
        (height, width, 4, stride)
    };
    for line in 0..lines {
        for channel in 0..4 {
            let offset = line * line_step + channel;
            let at =
                |position: isize| offset + position.clamp(0, length as isize - 1) as usize * step;
            let mut sum: u32 = (-radius..=radius).map(|p| input[at(p)] as u32).sum();
            for position in 0..length {
                output[offset + position * step] = (sum / (radius as u32 * 2 + 1)) as u8;
                sum -= input[at(position as isize - radius)] as u32;
                sum += input[at(position as isize + radius + 1)] as u32;
            }
        }
    }
}

fn save_png(png: &[u8]) -> Result<std::path::PathBuf, String> {
    let directory = gtk::glib::home_dir().join("Pictures/screenshots");
    save_png_in(png, &directory)
}

fn save_png_in(png: &[u8], directory: &std::path::Path) -> Result<std::path::PathBuf, String> {
    use std::{
        fs,
        io::Write,
        time::{SystemTime, UNIX_EPOCH},
    };
    fs::create_dir_all(directory)
        .map_err(|e| format!("Не удалось создать {}: {e}", directory.display()))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    for suffix in 0..100 {
        let path = directory.join(format!(
            "Screenshot-{timestamp}-{}-{suffix}.png",
            std::process::id()
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                if let Err(error) = file.write_all(png).and_then(|_| file.sync_all()) {
                    let _ = fs::remove_file(&path);
                    return Err(format!("Ошибка сохранения: {error}"));
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("Не удалось сохранить {}: {error}", path.display())),
        }
    }
    Err("Не удалось подобрать свободное имя файла".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transformed_text_pixels_stay_inside_selection_frame() {
        for matrix in [
            [2.5, 0., 0., 1.5, 20., 10.],
            [1.4, 0.7, -0.2, 0.4, 90., 20.],
            [0.15, -0.1, 0.5, 2., 90., 100.],
        ] {
            let annotation = Annotation {
                tool: Tool::Text,
                start: (10., 20.),
                end: (10., 20.),
                color: (1., 0., 0., 1.),
                width: 4.,
                text: "lol123".into(),
                points: vec![],
                matrix,
            };
            let (x, y, w, h) = annotation.bounds();
            let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 600, 400).unwrap();
            {
                let cr = cairo::Context::new(&surface).unwrap();
                draw_annotation(&cr, &annotation, None).unwrap();
            }
            let stride = surface.stride() as usize;
            let data = surface.data().unwrap();
            let alpha = if cfg!(target_endian = "little") { 3 } else { 0 };
            let mut count = 0;
            for py in 0..400 {
                for px in 0..600 {
                    if data[py * stride + px * 4 + alpha] > 0 {
                        count += 1;
                        assert!(px as f64 >= x.floor() - 1. && px as f64 <= (x + w).ceil() + 1.);
                        assert!(py as f64 >= y.floor() - 1. && py as f64 <= (y + h).ceil() + 1.);
                    }
                }
            }
            assert!(count > 0);
        }
    }

    #[test]
    fn blur_frame_matches_mask_in_both_drag_directions() {
        for (start, end) in [((20., 30.), (80., 60.)), ((80., 60.), (20., 30.))] {
            let annotation = Annotation {
                tool: Tool::Blur,
                start,
                end,
                color: (0., 0., 0., 1.),
                width: 4.,
                text: String::new(),
                points: vec![start],
                matrix: [1., 0., 0., 1., 0., 0.],
            };
            assert_eq!(annotation.raw_bounds(), (20., 30., 60., 30.));
            let handles = annotation.handles(1.);
            assert_eq!(handles[0], (20., 30.));
            assert_eq!(handles[4], (80., 60.));
        }
    }

    #[test]
    fn selection_handles_rotate_with_object() {
        let object = Annotation {
            tool: Tool::Rectangle,
            start: (20., 30.),
            end: (60., 50.),
            color: (1., 0., 0., 1.),
            width: 2.,
            text: String::new(),
            points: vec![],
            matrix: [1., 0., 0., 1., 0., 0.],
        };
        let original = object.handles(1.);
        let rotated = object.transformed([0., 1., -1., 0., 100., 0.]).handles(1.);
        for (before, after) in original.into_iter().zip(rotated) {
            assert!((after.0 - (100. - before.1)).abs() < 0.001);
            assert!((after.1 - before.0).abs() < 0.001);
        }
    }

    #[test]
    fn blur_softens_edges_and_respects_mask() {
        let source = cairo::ImageSurface::create(cairo::Format::ARgb32, 80, 40).unwrap();
        {
            let cr = cairo::Context::new(&source).unwrap();
            cr.set_source_rgb(0., 0., 0.);
            cr.paint().unwrap();
            cr.set_source_rgb(1., 1., 1.);
            cr.rectangle(40., 0., 40., 40.);
            cr.fill().unwrap();
        }
        let blurred = blur_image(&source).unwrap();
        let mut result = cairo::ImageSurface::create(cairo::Format::ARgb32, 80, 40).unwrap();
        {
            let cr = cairo::Context::new(&result).unwrap();
            cr.set_source_surface(&source, 0., 0.).unwrap();
            cr.paint().unwrap();
            draw_annotation(
                &cr,
                &Annotation {
                    tool: Tool::Blur,
                    start: (30., 10.),
                    end: (50., 30.),
                    color: (1., 0., 0., 1.),
                    width: 4.,
                    text: String::new(),
                    points: vec![],
                    matrix: [1., 0., 0., 1., 0., 0.],
                },
                Some(&blurred),
            )
            .unwrap();
        }
        let stride = result.stride() as usize;
        let data = result.data().unwrap();
        let channel = if cfg!(target_endian = "little") { 0 } else { 1 };
        let value = data[20 * stride + 39 * 4 + channel];
        assert!(value > 0 && value < 255);
        assert_eq!(data[5 * stride + 39 * 4 + channel], 0);
        assert_eq!(data[5 * stride + 40 * 4 + channel], 255);
    }

    #[test]
    fn region_and_objects_undo_together() {
        let mut history = History::default();
        history.push(1);
        history.region = Some((10., 20., 100., 80.));
        history.checkpoint();
        history.items[0] = 2;
        history.region = Some((30., 40., 100., 80.));
        history.undo();
        assert_eq!(history.items, vec![1]);
        assert_eq!(history.region, Some((10., 20., 100., 80.)));
        history.redo();
        assert_eq!(history.items, vec![2]);
        assert_eq!(history.region, Some((30., 40., 100., 80.)));
    }
    #[test]
    fn edit_delete_history() {
        let mut h = History::default();
        h.push(10);
        h.push(20);
        h.checkpoint();
        h.items[0] = 30;
        h.undo();
        assert_eq!(h.items, vec![10, 20]);
        h.redo();
        assert_eq!(h.items, vec![30, 20]);
        h.checkpoint();
        h.items.remove(0);
        h.undo();
        assert_eq!(h.items, vec![30, 20]);
    }
    #[test]
    fn shortcuts_use_physical_keys() {
        assert_eq!(physical_shortcut(39, false), Some("save"));
        assert_eq!(physical_shortcut(54, false), Some("copy"));
        assert_eq!(physical_shortcut(52, false), Some("undo"));
        assert_eq!(physical_shortcut(52, true), Some("redo"));
        assert_eq!(physical_shortcut(38, false), None);
    }
    #[test]
    fn png_export_and_history() {
        let image = cairo::ImageSurface::create(cairo::Format::ARgb32, 160, 100).unwrap();
        let cr = cairo::Context::new(&image).unwrap();
        cr.set_source_rgb(1., 1., 1.);
        cr.paint().unwrap();
        let mut state = State {
            image: Some(image),
            blurred: None,
            selection: None,
            selecting: false,
            anchor: None,
            history: History {
                items: vec![],
                redo: vec![],
                undo: vec![],
                region: None,
            },
            selected: None,
            moving: None,
            handle: None,
            region_drag: None,
            draft: None,
            tool: Tool::Rectangle,
            color: (1., 0., 0., 1.),
            width: 4.,
        };
        let original = state.png().unwrap();
        for tool in [
            Tool::Arrow,
            Tool::Line,
            Tool::Rectangle,
            Tool::Ellipse,
            Tool::Text,
        ] {
            state.history.push(Annotation {
                tool,
                start: (10., 10.),
                end: (120., 80.),
                color: state.color,
                width: state.width,
                text: "Тест".into(),
                matrix: [1., 0., 0., 1., 0., 0.],
                points: vec![(10., 10.), (120., 80.)],
            });
        }
        let annotated = state.png().unwrap();
        assert_ne!(annotated, original);
        let decoded =
            cairo::ImageSurface::create_from_png(&mut std::io::Cursor::new(&annotated)).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (160, 100));
        for _ in 0..state.history.items.len() {
            state.history.undo();
        }
        assert_eq!(state.png().unwrap(), original);
        for _ in 0..state.history.redo.len() {
            state.history.redo();
        }
        assert_eq!(state.png().unwrap(), annotated);
    }
    #[test]
    fn export_only_selected_region() {
        let image = cairo::ImageSurface::create(cairo::Format::ARgb32, 200, 100).unwrap();
        let mut state = State {
            image: Some(image),
            blurred: None,
            selection: Some((30., 20., 60., 40.)),
            selecting: false,
            anchor: None,
            history: History {
                items: vec![],
                redo: vec![],
                undo: vec![],
                region: None,
            },
            selected: None,
            moving: None,
            handle: None,
            region_drag: None,
            draft: None,
            tool: Tool::Pen,
            color: (1., 0., 0., 1.),
            width: 4.,
        };
        let png = state.png().unwrap();
        let result = cairo::ImageSurface::create_from_png(&mut std::io::Cursor::new(png)).unwrap();
        assert_eq!((result.width(), result.height()), (60, 40));
        state.selecting = true;
        assert!(state.png().is_err());
    }

    #[test]
    fn save_unique_and_directory_error() {
        let dir = std::env::temp_dir().join(format!("umbrellascreen-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let first = save_png_in(b"first", &dir).unwrap();
        let second = save_png_in(b"second", &dir).unwrap();
        assert_ne!(first, second);
        assert_eq!(std::fs::read(&first).unwrap(), b"first");
        assert!(save_png_in(b"fail", &first).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn undo_redo_branch() {
        let mut h = History::default();
        h.push(1);
        h.push(2);
        h.undo();
        assert_eq!(h.items, vec![1]);
        h.redo();
        assert_eq!(h.items, vec![1, 2]);
        h.undo();
        h.push(3);
        h.redo();
        assert_eq!(h.items, vec![1, 3]);
        h.undo();
        h.undo();
        h.undo();
        assert!(h.items.is_empty());
    }
    #[test]
    fn hidpi_letterbox_coordinates() {
        let t = Transform::fit((1000., 800.), (2000., 1000.));
        assert_eq!(t.point(500., 400., false), Some((1000., 500.)));
        assert_eq!(t.point(500., 10., false), None);
        assert_eq!(t.point(1200., 800., true), Some((2000., 1000.)));
        let t = Transform::fit((800., 600.), (1200., 900.));
        assert_eq!(t.point(400., 300., false), Some((600., 450.)));
    }
}
