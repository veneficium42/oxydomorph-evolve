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
                biomorph_state.matrix = Matrix::initial_setup(&biomorph_state.config);
                /*                 biomorph_state.matrix.develop(0);
                 */
                for i in 0..n_biomorphs {
                    biomorph_state.matrix.develop(i);
                }
                biomorph_state.is_generated = true;
            }

            if ui.add(egui::Button::new("Reset")).clicked() {
                *biomorph_state = BiomorphState::default();
            }

            if ui
                .add_enabled(biomorph_state.is_generated, egui::Button::new("Develop"))
                .clicked()
            {
                biomorph_state.matrix.reproduce(0);
            }
        });
    });
}

fn update(biomorph_state: Res<BiomorphState>, mut gizmos: Gizmos, windows: Query<&Window>) {
    let resolution = &windows.single().resolution;
    let config = &biomorph_state.config;
    let matrix = &biomorph_state.matrix;

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
            //TODO  write offset to render each biomorph in its own little compartment
            let cell_cord = UVec2::new(
                (i % grid.x) as u32,
                // i = x + y*xmax
                (i / grid.x) as u32,
            );

            let relative_offset = cell_cord.as_vec2() - ((grid - UVec2::ONE).as_vec2() * 0.5);

            let offset = relative_offset * cell_size;

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
