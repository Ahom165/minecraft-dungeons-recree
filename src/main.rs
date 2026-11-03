//! Point d'entrée : fenêtre winit, boucle d'événements, mode QA (captures).

mod atlas;
mod camera;
mod game;
mod gfx;
mod hud;
mod input;
mod level;
mod mobs;
mod particles;
mod player;
mod rng;
mod text;
mod ui_atlas;
mod vertex;

use glam::Vec3;
use std::sync::Arc;
use input::Input;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Arguments : [--qa <dossier>] pour générer les captures de contrôle.
fn qa_dir() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--qa")
        .and_then(|i| args.get(i + 1).cloned())
}

fn main() {
    let qa = qa_dir();
    let event_loop = EventLoop::new().expect("boucle d'événements impossible");
    let mut size = if qa.is_some() { (1280.0, 720.0) } else { (1600.0, 900.0) };
    if let Ok(w) = std::env::var("MCD_SIZE") {
        let parts: Vec<f64> = w.split('x').filter_map(|v| v.parse().ok()).collect();
        if parts.len() == 2 {
            size = (parts[0], parts[1]);
        }
    }
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("Minecraft Dungeons — Recréation (Creeper Woods)")
            .with_inner_size(winit::dpi::LogicalSize::new(size.0, size.1))
            .with_resizable(true)
            .build(&event_loop)
            .expect("fenêtre impossible"),
    );

    // Niveau (construit une seule fois, partagé entre renderer et jeu)
    let level = level::Level::build();
    let static_verts = level.static_verts.clone();
    let world_atlas = atlas::build("assets/textures");
    if !world_atlas.used_names.is_empty() {
        eprintln!("Textures utilisateur chargées : {:?}", world_atlas.used_names);
    }
    // QA : dump CPU des mips de l'atlas (MCD_DUMP_CPU=1), pour comparer au GPU.
    if std::env::var("MCD_DUMP_CPU").is_ok() {
        for mip in 0..=atlas::MIP_COUNT {
            let side = atlas::ATLAS_SIZE >> mip;
            let off = atlas::mip_offset(mip);
            let img = image::RgbaImage::from_raw(
                side as u32,
                side as u32,
                world_atlas.pixels[off..off + side * side * 4].to_vec(),
            )
            .expect("atlas CPU invalide");
            img.save(format!("/tmp/cpu_atlas_mip{mip}.png")).ok();
        }
    }
    let font_bytes = include_bytes!("../assets/fonts/DejaVuSans-Bold.ttf");
    let font = fontdue::Font::from_bytes(font_bytes.as_slice(), fontdue::FontSettings::default())
        .expect("police embarquée invalide");
    let mut ui_atlas = ui_atlas::UiAtlas::new(&font);

    let mut renderer = gfx::Renderer::new(window.clone(), &static_verts, &ui_atlas.pixels, &world_atlas.pixels);
    upload_dirty_glyphs(&mut renderer, &mut ui_atlas);

    let mut game = game::Game::new(level);
    let mut input = Input::new();
    let mut last = std::time::Instant::now();

    if let Some(dir) = qa {
        if std::env::var("MCD_DUMP_BUF").is_ok() {
            renderer.dump_static_buffer("/tmp/static_gpu.bin", (static_verts.len() * 80) as u64);
        }
        if std::env::var("MCD_DUMP_TEX").is_ok() {
            renderer.dump_world_texture("/tmp/world_tex_gpu.png");
        }
        run_qa(&mut game, &mut renderer, &mut ui_atlas, &font, &dir);
        return;
    }

    window.request_redraw();
    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(sz) => {
                        renderer.resize(sz.width, sz.height);
                        if sz.width > 0 && sz.height > 0 {
                            game.cam.set_aspect(sz.width as f32 / sz.height as f32);
                        }
                    }
                    WindowEvent::KeyboardInput { event: ke, .. } => {
                        if ke.state == ElementState::Pressed
                            && ke.physical_key == PhysicalKey::Code(KeyCode::F2)
                        {
                            screenshot(&mut renderer, &game, &mut ui_atlas, &font, "capture.png");
                        }
                        input.key_event(ke.state, ke.physical_key);
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        input.mouse_event(state, button);
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    let now = std::time::Instant::now();
                    let dt = (now - last).as_secs_f32().min(0.05);
                    last = now;
                    game.update(dt, &mut input);
                    draw_frame(&mut renderer, &game, &mut ui_atlas, &font, false);
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .expect("boucle interrompue");
}

fn upload_dirty_glyphs(renderer: &mut gfx::Renderer, ui_atlas: &mut ui_atlas::UiAtlas) {
    if !ui_atlas.dirty_glyphs.is_empty() {
        renderer.update_ui_atlas(&ui_atlas.pixels);
        ui_atlas.dirty_glyphs.clear();
    }
}

fn draw_frame(
    renderer: &mut gfx::Renderer,
    game: &game::Game,
    ui_atlas: &mut ui_atlas::UiAtlas,
    font: &fontdue::Font,
    capture: bool,
) -> Option<image::RgbaImage> {
    upload_dirty_glyphs(renderer, ui_atlas);
    let mut dyn_verts = Vec::with_capacity(20_000);
    game.build_dynamic(&mut dyn_verts);
    let mut fx_alpha = Vec::new();
    let mut fx_add = Vec::new();
    if std::env::var("MCD_NO_FX").is_err() {
        game.particles.build_batches(&mut fx_alpha, &mut fx_add);
    }
    let mut ui_verts = Vec::with_capacity(20_000);
    let (w, h) = renderer.size;
    hud::draw(game, ui_atlas, font, w as f32, h as f32, &mut ui_verts);
    let fd = gfx::FrameDraw {
        dyn_verts,
        fx_alpha,
        fx_add,
        ui_verts,
        cam: game.cam,
        fog_color: Vec3::new(0.086, 0.125, 0.098),
        fog_density: 0.026,
        lights: game.collect_lights(),
        time: game.time,
    };
    renderer.render(&fd, capture)
}

fn screenshot(
    renderer: &mut gfx::Renderer,
    game: &game::Game,
    ui_atlas: &mut ui_atlas::UiAtlas,
    font: &fontdue::Font,
    path: &str,
) {
    if let Some(img) = draw_frame(renderer, game, ui_atlas, font, true) {
        let _ = img.save(path);
    }
}

/// Mode QA : génère une série de captures de contrôle puis quitte.
fn run_qa(
    game: &mut game::Game,
    renderer: &mut gfx::Renderer,
    ui_atlas: &mut ui_atlas::UiAtlas,
    font: &fontdue::Font,
    dir: &str,
) {
    std::fs::create_dir_all(dir).ok();
    let mut input = Input::new();
    let dt = 1.0 / 60.0;

    // 1) écran-titre
    for _ in 0..40 {
        game.update(dt, &mut input);
        draw_frame(renderer, game, ui_atlas, font, false);
    }
    save(renderer, game, ui_atlas, font, dir, "01_titre.png");

    // 2) début de mission
    game.start_mission();
    game.screen = game::Screen::Playing;
    for _ in 0..90 {
        game.update(dt, &mut input);
        draw_frame(renderer, game, ui_atlas, font, false);
    }
    save(renderer, game, ui_atlas, font, dir, "02_debut_mission.png");

    // 3) combat (mobs proches)
    let p = game.player.pos;
    for i in 0..6 {
        let a = i as f32 * 1.1;
        let kind = if i % 3 == 0 { mobs::MobKind::Skeleton } else { mobs::MobKind::Zombie };
        game.mobs.push(mobs::Mob::new(kind, p + Vec3::new(a.cos() * 4.0, 0.0, a.sin() * 4.0), i == 1));
    }
    for _ in 0..60 {
        game.update(dt, &mut input);
        draw_frame(renderer, game, ui_atlas, font, false);
    }
    save(renderer, game, ui_atlas, font, dir, "03_combat.png");

    // 4) carte
    game.screen = game::Screen::Map;
    game.update(dt, &mut input);
    draw_frame(renderer, game, ui_atlas, font, false);
    save(renderer, game, ui_atlas, font, dir, "04_carte.png");

    // 5) inventaire
    game.screen = game::Screen::Inventory;
    game.update(dt, &mut input);
    draw_frame(renderer, game, ui_atlas, font, false);
    save(renderer, game, ui_atlas, font, dir, "05_inventaire.png");

    // 6) arène du boss
    game.screen = game::Screen::Playing;
    game.player.pos = game.level.anchors.boss_arena + Vec3::new(4.0, 0.0, 6.0);
    game.cam.snap(game.player.pos);
    game.spawn_boss();
    for _ in 0..80 {
        game.update(dt, &mut input);
        draw_frame(renderer, game, ui_atlas, font, false);
    }
    save(renderer, game, ui_atlas, font, dir, "06_boss.png");

    // 7) village + cages
    game.screen = game::Screen::Playing;
    game.player.pos = game.level.anchors.village + Vec3::new(0.0, 0.0, 4.0);
    game.cam.snap(game.player.pos);
    for _ in 0..50 {
        game.update(dt, &mut input);
        draw_frame(renderer, game, ui_atlas, font, false);
    }
    save(renderer, game, ui_atlas, font, dir, "07_village.png");

    // 8) pont de lave
    game.player.pos = game.level.anchors.boss_arena + Vec3::new(0.0, 0.0, -8.0);
    game.cam.snap(game.player.pos);
    for _ in 0..50 {
        game.update(dt, &mut input);
        draw_frame(renderer, game, ui_atlas, font, false);
    }
    save(renderer, game, ui_atlas, font, dir, "08_lave.png");

    eprintln!("QA : captures écrites dans {dir}");
}

fn save(
    renderer: &mut gfx::Renderer,
    game: &game::Game,
    ui_atlas: &mut ui_atlas::UiAtlas,
    font: &fontdue::Font,
    dir: &str,
    name: &str,
) {
    if let Some(img) = draw_frame(renderer, game, ui_atlas, font, true) {
        img.save(format!("{dir}/{name}")).expect("écriture capture impossible");
    }
}
