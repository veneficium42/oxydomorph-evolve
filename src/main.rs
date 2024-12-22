use bevy::{color::palettes::css::*, prelude::*};
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
        .init_resource::<BiomorphMatrix>()
        .init_gizmo_group::<GizmoConfig>()
        .add_systems(Startup, setup)
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
    is_generated: bool,
    generation_counter: u32,
    selected_biomorph: usize,
}

#[derive(Default, Resource)]
struct BiomorphMatrix {
    matrix: Matrix,
}

fn ui_system(
    mut biomorph_state: ResMut<BiomorphState>,
    mut biomorph_matrix: ResMut<BiomorphMatrix>,
    mut contexts: EguiContexts,
) {
    let n_biomorphs = &biomorph_state.config.rows * &biomorph_state.config.columns;

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
                biomorph_matrix.matrix = Matrix::initial_setup(&biomorph_state.config);
                for i in 0..n_biomorphs {
                    biomorph_matrix.matrix.develop(i);
                }
                biomorph_state.is_generated = true;
            }

            if ui.add(egui::Button::new("Reset")).clicked() {
                *biomorph_state = BiomorphState::default();
                biomorph_matrix.matrix = Matrix::default();
            }

            if ui
                .add_enabled(biomorph_state.is_generated, egui::Button::new("Develop"))
                .clicked()
            {
                biomorph_matrix
                    .matrix
                    .reproduce(biomorph_state.selected_biomorph);
                biomorph_state.generation_counter += 1;
            }
        });
        ui.add_enabled(
            biomorph_state.is_generated,
            egui::Slider::new(&mut biomorph_state.selected_biomorph, 0..=(n_biomorphs - 1)),
        );

        ui.label(format!("Generation: {}", biomorph_state.generation_counter));
    });
}

fn update(
    mut commands: Commands,
    biomorph_state: Res<BiomorphState>,
    biomorph_matrix: Res<BiomorphMatrix>,
    mut gizmos: Gizmos,
    windows: Query<&Window>,
) {
    let resolution = &windows.single().resolution;
    let config = &biomorph_state.config;
    let matrix = &biomorph_matrix.matrix;

    let grid = UVec2::new(config.columns as u32, config.rows as u32);

    let cell_size = Vec2::new(
        resolution.width() / (grid.x as f32),
        resolution.height() / (grid.y as f32),
    );

    gizmos
        .grid_2d(Vec2::ZERO, 0.0, grid, cell_size, WHITE)
        .outer_edges();

    if biomorph_state.is_generated {
        for i in 0..(grid.x * grid.y) {
            let biomorph = &matrix.biomorphs[i as usize];

            let cell_cord = UVec2::new(
                (i % grid.x) as u32,
                // i = x + y*xmax
                (i / grid.x) as u32,
            );

            //this describes how many cells this must be moved from the center
            let relative_offset = cell_cord.as_vec2() - ((grid - UVec2::ONE).as_vec2() * 0.5);

            let offset = relative_offset * cell_size;

            let (min, max) = biomorph.bounding_box();
            let size = max - min;
            let center = biomorph.center();

            let scale = cell_size / size.xy() * 0.9;

            for segment in &biomorph.segment_list {
                gizmos.line_2d(
                    ((segment.start.xy().as_vec2() - center.xy()) * scale) + offset,
                    ((segment.end.xy().as_vec2() - center.xy()) * scale) + offset,
                    WHITE,
                );
            }
            if i == biomorph_state.selected_biomorph as u32 {
                gizmos.rect_2d(offset, 0., cell_size, RED);
            }
        }
    }
}
