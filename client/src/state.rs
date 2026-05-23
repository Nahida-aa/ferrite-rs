use anyhow::Result;
use bevy::prelude::*;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

use crate::network::{Network, NetworkEvent as NetMsg};
use crate::render::Renderer;
use crate::server::ServerHandle;

// ── Bevy Resources (data that will live in the ECS World) ──

#[derive(Resource)]
struct NetworkRes {
    inner: Option<(Network, Option<ServerHandle>)>,
    connected: bool,
    connecting: bool,
}

#[derive(Resource)]
struct EcsRuntime(Runtime);

#[derive(Resource)]
struct PlayerRes {
    position: Option<(f64, f64, f64)>,
}

#[derive(Resource)]
struct UiRes {
    last_error: Option<String>,
}

// ── Systems ──

fn poll_network_system(
    mut net: ResMut<NetworkRes>,
    mut player: ResMut<PlayerRes>,
    mut ui: ResMut<UiRes>,
) {
    let event = {
        let (net_inner, _) = match &mut net.inner {
            Some(n) => n,
            None => return,
        };
        match net_inner.try_recv() {
            Ok(Some(e)) => Some(e),
            Ok(None) => None,
            Err(mpsc::error::TryRecvError::Disconnected) => {
                tracing::info!("Network task ended");
                net.inner = None;
                net.connected = false;
                net.connecting = false;
                player.position = None;
                return;
            }
            Err(mpsc::error::TryRecvError::Empty) => None,
        }
    };
    match event {
        Some(NetMsg::Connected) => {
            tracing::info!("Connected!");
            net.connecting = false;
            net.connected = true;
        }
        Some(NetMsg::Disconnected(r)) => {
            tracing::info!("Disconnected: {r}");
            ui.last_error = Some(r);
            net.inner = None;
            net.connected = false;
            net.connecting = false;
            player.position = None;
        }
        Some(NetMsg::PlayerPosition(x, y, z)) => {
            player.position = Some((x, y, z));
        }
        None => {}
    }
}

// ── AppState (temporary shell around World + winit) ──

pub struct AppState {
    window: Option<Window>,
    renderer: Option<Renderer>,
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,

    world: World,
    schedule: Schedule,

    // egui closures can't borrow self, so buffer actions here
    pending_connect: Vec<(String, bool)>,
}

impl AppState {
    pub fn queue_connect(&mut self, address: String, with_server: bool) {
        self.pending_connect.push((address, with_server));
    }

    pub fn new() -> Result<Self> {
        let mut world = World::new();

        world.insert_resource(NetworkRes {
            inner: None,
            connected: false,
            connecting: false,
        });
        world.insert_resource(PlayerRes { position: None });
        world.insert_resource(UiRes { last_error: None });
        world.insert_resource(EcsRuntime(Runtime::new()?));

        let mut schedule = Schedule::default();
        schedule.add_systems(poll_network_system);

        let egui_ctx = egui::Context::default();
        Ok(Self {
            window: None,
            renderer: None,
            egui_ctx,
            egui_state: None,
            world,
            schedule,
            pending_connect: Vec::new(),
        })
    }

    pub fn handle_event(
        &mut self,
        event: winit::event::Event<()>,
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
        // 1. Process pending connections (from egui button clicks)
        let connects = std::mem::take(&mut self.pending_connect);
        if !connects.is_empty() {
            let runtime_handle = self.world.resource::<EcsRuntime>().0.handle().clone();
            for (addr, sv) in connects {
                connect_to_server(&mut self.world, &runtime_handle, &addr, sv);
            }
        }

        // 2. Run ECS schedule (poll network, etc.)
        self.schedule.run(&mut self.world);

        // 3. Read state for UI rendering
        let connected = self.world.resource::<NetworkRes>().connected;
        let connecting = self.world.resource::<NetworkRes>().connecting;
        let player_pos = self.world.resource::<PlayerRes>().position;
        let last_error = self.world.resource::<UiRes>().last_error.clone();

        // 4. Render egui
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
        let connects = &mut self.pending_connect;
        let mut full_output = egui_ctx.run(raw_input, |ctx| {
            render_ui(ctx, connected, connecting, player_pos, &last_error, connects);
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

        // 5. Render 3D
        if let Some(r) = &mut self.renderer {
            r.render(&self.egui_ctx, &full_output, connected);
        }
    }
}

// ── Connection helper ──

fn connect_to_server(
    world: &mut World,
    runtime_handle: &tokio::runtime::Handle,
    address: &str,
    start_server: bool,
) {
    {
        let net = world.resource::<NetworkRes>();
        if net.connecting || net.connected {
            return;
        }
    }
    {
        let mut ui = world.resource_mut::<UiRes>();
        ui.last_error = None;
    }

    let server = if start_server {
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
    let (network, _join) = Network::connect(runtime_handle, address, &username);
    let mut net = world.resource_mut::<NetworkRes>();
    net.inner = Some((network, server));
    net.connecting = true;
}

// ── UI rendering (pure functions, no AppState binding) ──

fn render_ui(
    ctx: &egui::Context,
    connected: bool,
    connecting: bool,
    player_pos: Option<(f64, f64, f64)>,
    last_error: &Option<String>,
    connects: &mut Vec<(String, bool)>,
) {
    if connected {
        render_ingame(ctx, player_pos);
    } else if connecting {
        render_connecting(ctx);
    } else {
        render_menu(ctx, last_error, connects);
    }
}

fn render_menu(ctx: &egui::Context, last_error: &Option<String>, connects: &mut Vec<(String, bool)>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(120.0);
            ui.heading("Ferrite");
            ui.add_space(20.0);

            if ui.button("Single Player").clicked() {
                connects.push(("127.0.0.1:25565".to_string(), true));
            }
            if ui.button("Multi Player").clicked() {}
            if ui.button("Quit").clicked() {
                std::process::exit(0);
            }

            if let Some(err) = last_error {
                ui.add_space(20.0);
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            }
        });
    });
}

fn render_connecting(ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(160.0);
            ui.heading("Connecting...");
            ui.add_space(10.0);
            ui.add(egui::Spinner::default());
        });
    });
}

fn render_ingame(ctx: &egui::Context, player_pos: Option<(f64, f64, f64)>) {
    egui::TopBottomPanel::top("hud")
        .frame(egui::Frame::none().fill(egui::Color32::from_black_alpha(150)).inner_margin(8.0))
        .show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
            ui.label(egui::RichText::new("Ferrite").strong());
            if let Some((x, y, z)) = player_pos {
                ui.separator();
                ui.label(format!("XYZ: {:.1} / {:.1} / {:.1}", x, y, z));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.colored_label(egui::Color32::GREEN, "● Connected");
            });
        });
    });

    egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
        let rect = ui.max_rect();
        let center = rect.center();
        let color = egui::Color32::from_white_alpha(200);
        let stroke = egui::Stroke::new(2.0, color);
        let len = 10.0;

        ui.painter().line_segment([center - egui::vec2(len, 0.0), center + egui::vec2(len, 0.0)], stroke);
        ui.painter().line_segment([center - egui::vec2(0.0, len), center + egui::vec2(0.0, len)], stroke);

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
