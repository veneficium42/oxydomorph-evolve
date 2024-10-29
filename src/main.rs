use std::mem::offset_of;

use bevy::{
    color::palettes::css::{RED, WHITE},
    gizmos::config,
    math::VectorSpace,
    prelude::*,
    render::render_resource::encase::matrix,
    window::WindowResolution,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

mod biomorph;
use biomorph::{Biomorph, Config, Matrix};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Oxydomorph Evolve".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .init_resource::<BiomorphState>()
        .init_gizmo_group::<GizmoConfig>()
        .add_systems(Startup, setup)
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(Update, (ui_system, update))
        .run();
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct GizmoConfig {}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Default, Resource)]
struct BiomorphState {
    config: Config,
    matrix: Matrix,
    is_generated: bool,
}

fn ui_system(mut biomorph_state: ResMut<BiomorphState>, mut contexts: EguiContexts) {
    egui::Window::new("Configuration").show(contexts.ctx_mut(), |ui| {
        ui.add_enabled_ui(!biomorph_state.is_generated, |ui| {
            ui.horizontal(|ui| {
                ui.label("Columns");
                ui.add(egui::Slider::new(&mut biomorph_state.config.columns, 1..=5));
            });

            ui.horizontal(|ui| {
                ui.label("Rows");
                ui.add(egui::Slider::new(&mut biomorph_state.config.rows, 1..=5));
            });
        });

        ui.horizontal(|ui| {
            if ui
                .add_enabled(!biomorph_state.is_generated, egui::Button::new("Generate"))
                .clicked()
            {
                biomorph_state.matrix = Matrix::initial_setup(&biomorph_state.config);
                /*                 biomorph_state.matrix.develop(0);
                 */
                for i in 0..(&biomorph_state.config.rows * &biomorph_state.config.columns) {
                    biomorph_state.matrix.develop(i);
                }
                biomorph_state.is_generated = true;
            }

            if ui.add(egui::Button::new("Reset")).clicked() {
                *biomorph_state = BiomorphState::default();
            }
        });
    });
}

fn update(biomorph_state: Res<BiomorphState>, mut gizmos: Gizmos, windows: Query<&Window>) {
    let resolution = &windows.single().resolution;
    let config = &biomorph_state.config;
    let matrix = &biomorph_state.matrix;

    let cell_size = Vec2::new(
        resolution.width() / (config.columns as f32),
        resolution.height() / (config.rows as f32),
    );

    gizmos
        .grid_2d(
            Vec2::ZERO,
            0.0,
            UVec2::new(config.columns as u32, config.rows as u32),
            cell_size,
            WHITE,
        )
        .outer_edges();

    if biomorph_state.is_generated {
        for i in 0..(config.columns * config.rows) {
            let biomorph = &matrix.biomorphs[i];
            //TODO  write offset to render each biomorph in its own little compartment
            let offset = Vec2::new(
                ((i % config.columns) as f32 - 1.0) * cell_size.x,
                (i.div_ceil(config.rows) as f32 - 2.0) * cell_size.y,
            );
            /* let offset = Vec2::ZERO; */

            let (min, max) = biomorph.bounding_box();
            let size = max - min;
            let center = biomorph.center();

            //TODO: resize biomorphs so they fit into their own lil space :3

            let scale = cell_size / size.xy() * 0.9;

            //gizmos.rect_2d(Vec2::ZERO + offset, 0.0, size.xy() * scale, RED);

            for segment in &biomorph.segment_list {
                gizmos.line_2d(
                    ((segment.start.xy().as_vec2() - center.xy()) * scale) + offset,
                    ((segment.end.xy().as_vec2() - center.xy()) * scale) + offset,
                    WHITE,
                );
            }
        }
    }
}
