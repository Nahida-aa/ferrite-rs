use anyhow::Result;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

use crate::network::{Network, NetworkEvent};
use crate::render::Renderer;
use crate::server::ServerHandle;

#[derive(Debug)]
enum Action {
    Connect(String, bool),
    Quit,
}

pub struct AppState {
    window: Option<Window>,
    renderer: Option<Renderer>,
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    runtime: Runtime,
    network: Option<(Network, Option<ServerHandle>)>,
    connected: bool,
    connecting: bool,
    last_error: Option<String>,
    player_pos: Option<(f64, f64, f64)>,
    actions: Vec<Action>,
}

impl AppState {
    pub fn queue_connect(&mut self, address: String, with_server: bool) {
        self.actions.push(Action::Connect(address, with_server));
    }

    pub fn new() -> Result<Self> {
        let runtime = Runtime::new()?;
        let egui_ctx = egui::Context::default();
        Ok(Self {
            window: None,
            renderer: None,
            egui_ctx,
            egui_state: None,
            runtime,
            network: None,
            connected: false,
            connecting: false,
            last_error: None,
            player_pos: None,
            actions: Vec::new(),
        })
    }

    pub fn handle_event(
        &mut self,
        event: Event<()>,
        target: &EventLoopWindowTarget<()>,
    ) {
        target.set_control_flow(winit::event_loop::ControlFlow::Poll);

        match event {
            Event::Resumed => {
                self.init_window(target);
            }
            Event::AboutToWait => {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => self.on_redraw(),
                other => self.on_window_event(other, target),
            },
            _ => {}
        }
    }

    fn init_window(&mut self, target: &EventLoopWindowTarget<()>) {
        let window = winit::window::WindowBuilder::new()
            .with_title("Ferrite")
            .build(target)
            .unwrap();

        let egui_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::viewport::ViewportId::ROOT,
            &window,
            None,
            None,
        );
        self.egui_state = Some(egui_state);

        let renderer = pollster::block_on(Renderer::new(&window));
        self.renderer = Some(renderer);
        window.request_redraw();
        self.window = Some(window);
    }

    fn on_window_event(&mut self, event: WindowEvent, target: &EventLoopWindowTarget<()>) {
        let Some(window) = &self.window else { return };
        let Some(egui_state) = &mut self.egui_state else { return };

        let consumed = egui_state.on_window_event(window, &event);
        if consumed.consumed {
            return;
        }

        match event {
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.renderer {
                    r.resize(size.width, size.height);
                }
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self) {
        self.poll_network();
        self.process_actions();

        let raw_input = {
            let w = match &self.window {
                Some(w) => w,
                None => return,
            };
            let s = match &mut self.egui_state {
                Some(s) => s,
                None => return,
            };
            s.take_egui_input(w)
        };

        let egui_ctx = self.egui_ctx.clone();
        let mut full_output = egui_ctx.run(raw_input, |ctx| {
            self.render_ui(ctx);
        });

        let Some(window) = &self.window else { return };
        let Some(egui_state) = &mut self.egui_state else { return };

        let sf = window.scale_factor() as f32;
        if (full_output.pixels_per_point - sf).abs() > 0.01 {
            full_output.pixels_per_point = sf;
        }

        egui_state.handle_platform_output(
            window,
            std::mem::take(&mut full_output.platform_output),
        );

        if let Some(r) = &mut self.renderer {
            r.render(&self.egui_ctx, &full_output, self.connected);
        }
    }

    fn process_actions(&mut self) {
        let actions: Vec<Action> = self.actions.drain(..).collect();
        for action in actions {
            match action {
                Action::Connect(addr, sv) => self.do_connect(&addr, sv),
                Action::Quit => {}
            }
        }
    }

    fn do_connect(&mut self, address: &str, with_server: bool) {
        if self.connecting || self.connected {
            return;
        }
        self.last_error = None;

        let server = if with_server {
            match ServerHandle::spawn() {
                Ok(s) => {
                    tracing::info!("Local server started");
                    Some(s)
                }
                Err(e) => {
                    tracing::error!("Start server: {e}");
                    return;
                }
            }
        } else {
            None
        };

        let username = format!("FerritePlayer_{}", std::process::id());
        let (network, _join) = Network::connect(self.runtime.handle(), address, &username);
        self.network = Some((network, server));
        self.connecting = true;
    }

    fn disconnect(&mut self) {
        self.network = None;
        self.connecting = false;
        self.connected = false;
        self.player_pos = None;
    }

    fn poll_network(&mut self) {
        let event = {
            let (net, _) = match &mut self.network {
                Some(n) => n,
                None => return,
            };
            match net.try_recv() {
                Ok(Some(e)) => Some(e),
                Ok(None) => None,
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    tracing::info!("Network task ended");
                    return self.disconnect();
                }
                Err(mpsc::error::TryRecvError::Empty) => None,
            }
        };
        match event {
            Some(NetworkEvent::Connected) => {
                tracing::info!("Connected!");
                self.connecting = false;
                self.connected = true;
            }
            Some(NetworkEvent::Disconnected(r)) => {
                tracing::info!("Disconnected: {r}");
                self.last_error = Some(r);
                self.disconnect();
            }
            Some(NetworkEvent::PlayerPosition(x, y, z)) => {
                self.player_pos = Some((x, y, z));
            }
            None => {}
        }
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        if self.connected {
            self.render_ingame(ctx);
        } else if self.connecting {
            self.render_connecting(ctx);
        } else {
            self.render_menu(ctx);
        }
    }

    fn render_menu(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(120.0);
                ui.heading("Ferrite");
                ui.add_space(20.0);

                if ui.button("Single Player").clicked() {
                    self.actions.push(Action::Connect("127.0.0.1:25565".into(), true));
                }
                if ui.button("Multi Player").clicked() {}
                if ui.button("Quit").clicked() {
                    self.actions.push(Action::Quit);
                }

                if let Some(err) = &self.last_error {
                    ui.add_space(20.0);
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                }
            });
        });
    }

    fn render_connecting(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(160.0);
                ui.heading("Connecting...");
                ui.add_space(10.0);
                ui.add(egui::Spinner::default());
            });
        });
    }

    fn render_ingame(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("hud")
            .frame(egui::Frame::none().fill(egui::Color32::from_black_alpha(150)).inner_margin(8.0))
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                ui.label(egui::RichText::new("Ferrite").strong());
                if let Some((x, y, z)) = self.player_pos {
                    ui.separator();
                    ui.label(format!("XYZ: {:.1} / {:.1} / {:.1}", x, y, z));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(egui::Color32::GREEN, "● Connected");
                });
            });
        });

        // Draw crosshair
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let rect = ui.max_rect();
            let center = rect.center();
            let color = egui::Color32::from_white_alpha(200);
            let stroke = egui::Stroke::new(2.0, color);
            let len = 10.0;
            
            ui.painter().line_segment([center - egui::vec2(len, 0.0), center + egui::vec2(len, 0.0)], stroke);
            ui.painter().line_segment([center - egui::vec2(0.0, len), center + egui::vec2(0.0, len)], stroke);
            
            // Draw placeholder health bar at the bottom
            let health_center = egui::pos2(center.x, rect.max.y - 40.0);
            ui.painter().text(
                health_center,
                egui::Align2::CENTER_CENTER,
                "♥♥♥♥♥♥♥♥♥♥",
                egui::FontId::proportional(24.0),
                egui::Color32::RED,
            );
        });
    }
}
