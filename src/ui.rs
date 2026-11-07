use crate::{
    autostart,
    mapper::{DeviceInfo, Mapper, PedalReading},
    scaling::{MAX_PERCENT, MIN_PERCENT},
    settings::Settings,
};
use gpui::{
    Bounds, Context, FocusHandle, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent,
    Pixels, Point, Window, canvas, div, prelude::*, px, relative, rgb, rgba,
};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

pub type SharedModel = Rc<RefCell<Model>>;

pub struct Model {
    mapper: Mapper,
    settings: Settings,
    path: PathBuf,
    device: Option<DeviceInfo>,
    pedal: PedalReading,
    autostart: bool,
    error: Option<String>,
    pending_save: Option<Instant>,
    tray_available: bool,
}
impl Model {
    pub fn new(path: PathBuf, tray_available: bool) -> Self {
        let settings = Settings::load(&path);
        let mapper = Mapper::new(settings.brake_force);
        Self {
            settings,
            path,
            mapper,
            device: None,
            pedal: PedalReading {
                input: 0.,
                output: 0.,
            },
            autostart: autostart::is_enabled(),
            error: None,
            pending_save: None,
            tray_available,
        }
    }
    pub fn start(&self) {
        self.mapper.spawn(|| {});
    }

    pub fn refresh(&mut self, visible: bool) -> bool {
        if self
            .pending_save
            .is_some_and(|time| time.elapsed() >= Duration::from_millis(120))
        {
            self.save();
            return true;
        }
        if !visible {
            return false;
        }
        let device = self.mapper.device();
        let pedal = self.mapper.pedal();
        if self.device == device && self.pedal == pedal {
            return false;
        }
        self.device = device;
        self.pedal = pedal;
        true
    }
    pub fn save(&mut self) {
        self.pending_save = None;
        self.error = self
            .settings
            .save(&self.path)
            .err()
            .map(|error| error.to_string());
    }
    fn set_level(&mut self, value: i16) {
        let value = value.clamp(MIN_PERCENT.into(), MAX_PERCENT.into()) as u8;
        if self.settings.brake_force == value {
            return;
        }
        self.settings.brake_force = value;
        self.mapper.set_percent(value);
        self.pending_save = Some(Instant::now());
    }
    fn toggle_autostart(&mut self) {
        match autostart::set_enabled(!self.autostart) {
            Ok(()) => {
                self.autostart = autostart::is_enabled();
                self.error = None;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}
impl Drop for Model {
    fn drop(&mut self) {
        if self.pending_save.is_some() {
            self.save();
        }
    }
}

pub struct Panel {
    model: SharedModel,
    focus: FocusHandle,
    dragging: bool,
    meter_bounds: Bounds<Pixels>,
}
impl Panel {
    pub fn new(model: SharedModel, window: &mut Window, cx: &mut Context<Self>) -> Self {
        window.set_window_title("BrakeTune");
        let focus = cx.focus_handle();
        window.focus(&focus);
        Self {
            model,
            focus,
            dragging: false,
            meter_bounds: Bounds::default(),
        }
    }
    fn level(&self, value: i16, cx: &mut Context<Self>) {
        self.model.borrow_mut().set_level(value);
        cx.notify();
    }
    fn level_at(&self, position: Point<Pixels>, cx: &mut Context<Self>) {
        if self.meter_bounds.size.width > px(0.) {
            let fraction = (position.x - self.meter_bounds.left()) / self.meter_bounds.size.width;
            self.level((fraction * 100.).round() as i16, cx);
        }
    }
    fn button(
        &self,
        id: &'static str,
        label: &'static str,
        delta: i16,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .w(px(40.))
            .h(px(32.))
            .flex()
            .items_center()
            .justify_center()
            .border_1()
            .border_color(rgb(0x3a3b41))
            .rounded_sm()
            .bg(rgb(0x17181b))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(0x26272b)))
            .on_click(cx.listener(move |this, _, _, cx| {
                let level = this.model.borrow().settings.brake_force;
                this.level(i16::from(level) + delta, cx);
            }))
            .child(label)
    }
}
impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.borrow();
        let level = model.settings.brake_force;
        let connected = model.device.is_some();
        let input = if connected { model.pedal.input } else { 0. };
        let output = if connected { model.pedal.output } else { 0. };
        let device = model
            .device
            .as_ref()
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "No pedals connected".into());
        let error = model.error.clone();
        let autostart = model.autostart;
        let tray_available = model.tray_available;
        div().size_full().bg(rgb(0x111214)).flex().justify_center().child(
        div().id("panel").track_focus(&self.focus).size_full().max_w(px(400.)).flex().flex_col().gap(px(10.))
            .p(px(16.)).bg(rgb(0x111214)).text_color(rgb(0xececee)).font_family("DejaVu Sans").text_size(px(13.)).line_height(relative(1.4))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                let level = i16::from(this.model.borrow().settings.brake_force);
                let next = match event.keystroke.key.as_str() {
                    "up" | "right" | "k" | "l" => level + 1,
                    "down" | "left" | "j" | "h" => level - 1,
                    "pageup" => level + 10,
                    "pagedown" => level - 10,
                    "home" => MIN_PERCENT.into(),
                    "end" => MAX_PERCENT.into(),
                    _ => return,
                };
                this.level(next, cx);
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if this.dragging && event.pressed_button == Some(MouseButton::Left) { this.level_at(event.position, cx); }
                else { this.dragging = false; }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, _| this.dragging = false))
            .on_mouse_up_out(MouseButton::Left, cx.listener(|this, _, _, _| this.dragging = false))
            .child(div().flex().items_center().justify_between().h(px(24.)).flex_shrink_0()
                .child(div().id("titlebar").flex_1().cursor_pointer()
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
                    .child("BrakeTune"))
                .child(div().id("minimize").px_3().cursor_pointer()
                    .on_click(|_, window, _| window.minimize_window()).child("−"))
                .child(div().id("close").px_3().cursor_pointer()
                    .hover(|style| style.bg(rgb(0xc42b1c)))
                    .on_click(|_, window, _| window.remove_window()).child("×")))
            .child(div().flex().items_center().gap_2()
                .child(div().size(px(6.)).rounded_full().bg(rgb(if connected { 0x3ccf74 } else { 0x3a3b41 })))
                .child(device))
            .child(div().flex().justify_between().items_end()
                .child(div().child(div().text_color(rgb(0x8d8e95)).child("Brake force"))
                    .child(div().text_size(px(56.)).font_family("DejaVu Sans Mono").child(format!("{level}%"))))
                .child(div().flex().gap_2().pb_2().child(self.button("decrease", "−", -1, cx)).child(self.button("increase", "+", 1, cx))))
            .child(div().flex().flex_col().gap(px(6.))
                .child(div().id("force-meter").relative().h(px(56.)).w_full().bg(rgb(0x17181b)).rounded_sm().overflow_hidden().cursor_ew_resize()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, window, cx| {
                        window.focus(&this.focus); this.dragging = true; this.level_at(event.position, cx);
                    }))
                    .child(div().absolute().left(relative(f32::from(level) / 100.)).right_0().h_full().bg(rgb(0x26272b)))
                    .child(div().absolute().left_0().top_0().h_full().w(relative(input)).bg(rgb(0x70321e)))
                    .children((1..10).map(|tick| div().absolute().left(relative(tick as f32 / 10.)).h_full().w(px(1.)).bg(rgba(0xffffff0c))))
                    .child(div().absolute().left(relative(f32::from(level) / 100.)).ml(px(-2.)).h_full().w(px(2.)).bg(rgb(0xececee)))
                    .child(canvas({ let panel = cx.entity().downgrade(); move |bounds, _, cx| { let _ = panel.update(cx, |panel, _| panel.meter_bounds = bounds); } }, |_, _, _, _| {}).absolute().size_full()))
                .child(div().h(px(4.)).w_full().bg(rgb(0x0b0c0d)).child(div().h_full().w(relative(output)).bg(rgb(0xff5a1f))))
                .child(div().flex().justify_between().text_size(px(10.)).text_color(rgb(0x8d8e95)).children(["0", "25", "50", "75", "100"])))
            .child(div().flex().gap(px(48.)).pt_3().border_t_1().border_color(rgb(0x26272b))
                .children([("Game input", output, 0xff5a1f), ("Pedal input", input, 0xececee)].map(|(label, value, color)| {
                    div().flex().flex_col().gap_1().child(div().text_color(rgb(0x8d8e95)).child(label))
                        .child(div().text_size(px(24.)).text_color(rgb(color)).child(if connected { format!("{:.0}%", value * 100.) } else { "—".into() }))
                })))
            .child(div().text_size(px(12.)).text_color(rgb(0x8d8e95)).child("Drag the white marker to set brake force: pressing the pedal up to the marker gives full braking in game. Lower values need less pressure."))
            .children(error.map(|error| div().text_color(rgb(0xff6b5b)).child(error)))
            .child(div().id("autostart").flex().justify_between().items_center().py_3().border_t_1().border_b_1().border_color(rgb(0x26272b)).cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| { this.model.borrow_mut().toggle_autostart(); cx.notify(); }))
                .child(div().child("Start at login").child(div().text_size(px(11.)).text_color(rgb(0x8d8e95)).child(if tray_available { "Stays in the tray when the window is closed" } else { "Tray unavailable; closing the window quits" })))
                .child(div().text_color(rgb(if autostart { 0xff5a1f } else { 0x8d8e95 })).child(if autostart { "On" } else { "Off" })))
            .child(div().text_size(px(12.)).text_color(rgb(0x8d8e95)).child("In your game, select BrakeTune Pedals. The original pedals are hidden while BrakeTune runs."))
            .child(div().flex_1())
            .child(div().text_size(px(11.)).text_color(rgb(0x5c5d64)).child("Unofficial. Not affiliated with Fanatec.")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_controls_preserve_settings_and_clamp_force() {
        let dir = std::env::temp_dir().join(format!("braketune-model-{}", std::process::id()));
        let path = dir.join("settings.json");
        Settings { brake_force: 42 }.save(&path).unwrap();
        let mut model = Model::new(path.clone(), true);
        assert_eq!(model.settings.brake_force, 42);
        model.set_level(-100);
        assert_eq!(model.settings.brake_force, MIN_PERCENT);
        model.set_level(120);
        assert_eq!(model.settings.brake_force, MAX_PERCENT);
        model.set_level(65);
        assert_eq!(Settings::load(&path).brake_force, 42);
        model.save();
        assert_eq!(Settings::load(&path).brake_force, 65);
        assert!(model.error.is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn settings_write_failure_is_visible() {
        let dir = std::env::temp_dir().join(format!("braketune-save-error-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut model = Model::new(dir.clone(), true);
        model.set_level(50);
        model.save();
        assert!(model.error.is_some());
        assert_eq!(model.settings.brake_force, 50);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
