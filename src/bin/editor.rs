use std::ops::{Index, IndexMut};

use bevy::{prelude::*, window::PrimaryWindow, render::camera::ScalingMode, input::mouse::MouseButtonInput};
use bevy_egui::{EguiPlugin, EguiContexts, egui};
use image::{init_picture_render, update_pixels};
use sixteenbit_encoding::types::{ColorIndex, PaletteCollection};
use utils::world_to_grid;

mod image;
mod utils;

#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

const CAMERA_TARGET: Vec3 = Vec3::ZERO;
const EDITOR_SIZE: usize = 16;
const GRID_WIDTH: f32 = 0.05;

// #[derive(Resource, Deref, DerefMut)]
// struct OriginalCameraTransform(Transform);

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

#[derive(Resource, Default)]
enum CursorType {
    #[default]
    None,
    Pencil(ColorIndex),
    Eraser,
}

#[derive(Resource, Default)]
pub struct PixelData(Vec<ColorIndex>);

#[derive(Resource, Default)]
pub struct PalettesData(PaletteCollection<u8>);

#[derive(Resource, Default)]
pub struct EditorSettings {
    pub selected_palette: u8,
}

impl Index<usize> for PixelData {
    type Output = [ColorIndex];

    fn index(&self, index: usize) -> &Self::Output {
        &self.0
        .chunks(EDITOR_SIZE)
        .nth(index)
        // .map(|r|PixelRow(r.to_vec()) )
        .expect("getting pixel row")
        // .to_vec()
}
}

impl IndexMut<usize> for PixelData {

    fn index_mut(&mut self, index: usize) -> &mut [ColorIndex] {
        // &mut PixelRow(
            self.0
            .chunks_mut(EDITOR_SIZE)
            .nth(index)
            .expect("getting pixel row")
        // )
    }
}

// marker component for our main camera view
#[derive(Component)]
struct MainCamera;

fn main() {
    sixteenbit_encoding::hello();
    App::new()
    .add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest()),// set pixel art render mode,
        EguiPlugin,
    ))
    .init_resource::<OccupiedScreenSpace>()
    .init_resource::<CursorWorldCoords>()
    .init_resource::<PalettesData>()
    .init_resource::<EditorSettings>()
    .insert_resource(CursorType::Pencil(ColorIndex::Skin))
    .insert_resource(PixelData(vec![ColorIndex::Empty; EDITOR_SIZE*EDITOR_SIZE]))
    .add_systems(Startup, 
        (
            init_picture_render,
            setup_system,
            setup_grid,
        )
    )
    .add_systems(Update, (
        // update_camera_transform_system,
        ui_example_system,
        cursor_system,
        input_system.after(cursor_system),
        update_pixels.after(input_system),
    ))
    .run();
}

fn input_system(
    mut pixels: ResMut<PixelData>,
    buttons: Res<Input<MouseButton>>,
    cursor: Res<CursorWorldCoords>,
    cursor_type: Res<CursorType>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        info!("clicked");
        match world_to_grid(cursor.0) {
            Some((x,y)) => {
                match cursor_type.as_ref() {
                    CursorType::Pencil(p) => {
                        pixels[y][x] = *p;
                        eprintln!("Placing pixel at: {}/{}", x, y);

                    },
                    CursorType::Eraser => {
                        
                    }
                    _ => {},
                }
            },
            _=> {}
        }
    }
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    let ctx = contexts.ctx_mut();

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Left resizeable panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Right resizeable panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Top resizeable panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Bottom resizeable panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
}

fn setup_grid(
    mut commands: Commands,
) {
    // Horizontal lines
    for i in 0..=EDITOR_SIZE {
        commands.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                0.,
                i as f32 - EDITOR_SIZE as f32 / 2.,
                0.,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(EDITOR_SIZE as f32, GRID_WIDTH)),
                ..default()
            },
            ..default()
        });
    }

    // Vertical lines
    for i in 0..=EDITOR_SIZE {
        commands.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                i as f32 - EDITOR_SIZE as f32 / 2.,
                0.,
                0.,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(GRID_WIDTH, EDITOR_SIZE as f32)),
                ..default()
            },
            ..default()
        });
    }
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 5.0,
            subdivisions: 0,
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    let camera_pos = Vec3::new(0., 0., 5.0);
    let camera_transform =
        Transform::from_translation(camera_pos).looking_at(CAMERA_TARGET, Vec3::Y);
    // commands.insert_resource(OriginalCameraTransform(camera_transform));

    commands.spawn((Camera2dBundle {
        transform: camera_transform,
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin { min_width: 20., min_height: 20. },
            ..Default::default()
        },
        ..Default::default()
    },
    MainCamera,
    ));
}

fn cursor_system(
    mut mycoords: ResMut<CursorWorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
        // eprintln!("World coords: {}/{}", world_position.x, world_position.y);
    }
}

// fn update_camera_transform_system(
//     occupied_screen_space: Res<OccupiedScreenSpace>,
//     // original_camera_transform: Res<OriginalCameraTransform>,
//     windows: Query<&Window, With<PrimaryWindow>>,
//     mut camera_query: Query<(&Projection, &mut Transform)>,
// ) {
//     // let (camera_projection, mut transform) = match camera_query.get_single_mut() {
//     //     Ok((Projection::Perspective(projection), transform)) => (projection, transform),
//     //     _ => unreachable!(),
//     // };

//     // let distance_to_target = (CAMERA_TARGET - original_camera_transform.translation).length();
//     // let frustum_height = 2.0 * distance_to_target * (camera_projection.fov * 0.5).tan();
//     // let frustum_width = frustum_height * camera_projection.aspect_ratio;

//     // let window = windows.single();

//     // let left_taken = occupied_screen_space.left / window.width();
//     // let right_taken = occupied_screen_space.right / window.width();
//     // let top_taken = occupied_screen_space.top / window.height();
//     // let bottom_taken = occupied_screen_space.bottom / window.height();
//     // transform.translation = original_camera_transform.translation
//     //     + transform.rotation.mul_vec3(Vec3::new(
//     //         (right_taken - left_taken) * frustum_width * 0.5,
//     //         (top_taken - bottom_taken) * frustum_height * 0.5,
//     //         0.0,
//     //     ));
// }