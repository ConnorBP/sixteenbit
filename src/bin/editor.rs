use std::ops::{Index, IndexMut, DerefMut};

use bevy::{prelude::*, window::PrimaryWindow, render::camera::{ScalingMode, Viewport}};
use bevy_egui::{EguiPlugin, EguiContexts, egui::{self, FontId, FontFamily, Slider, TextEdit}};
use image::{init_picture_render, update_pixels, encoder::{EncoderPlugin, RLEncodedString, RLEncodedBytes, RLEncoderSettings, RLEncodedStringSubmission}};
use sixteenbit_encoding::types::{ColorIndex, PaletteCollection, IndexedImage};
use utils::world_to_grid;
use widgets::{color_index, tool_selector};

mod image;
mod widgets;
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
const TOTAL_PIXELS: usize = EDITOR_SIZE * EDITOR_SIZE;
const GRID_WIDTH: f32 = 0.05;

// #[derive(Resource, Deref, DerefMut)]
// struct OriginalCameraTransform(Transform);

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

#[derive(Resource, Default, PartialEq)]
pub enum CursorType {
    #[default]
    Move,
    Pencil(ColorIndex),
    FillBucket(ColorIndex),
    Eraser,
}

#[derive(Default)]
struct MoveOp {
    pub move_start: Vec2,
    pub move_end: Vec2,
}

#[derive(Resource, Default)]
pub struct PixelData<const N: usize, const W: usize> {
    /// Keeps track of image move in progress
    move_op: Option<MoveOp>,
    pub pixels: IndexedImage<N,W>,
}

impl<const N: usize, const W: usize> PixelData<N,W> {
    /// starts a move operation
    pub fn start_move(&mut self, cursor_start_pos: Vec2) {
        self.move_op = Some(MoveOp {
            move_start: cursor_start_pos,
            move_end: cursor_start_pos
        });
    }

    pub fn update_move(&mut self, cursor_pos: Vec2) {
        match &mut self.move_op {
            Some(op) => {
                op.move_end = cursor_pos;
            },
            _ => {
                warn!("update_move called outside of valid move operation");
            },
        }
    }

    /// Cancels a move operation without applying it
    pub fn cancel_move(&mut self) {
        self.move_op = None;
    }

    /// ends and commits the move operation
    pub fn end_move(&mut self, cursor_pos: Vec2) {
        // apply the final transformation based on the final delta
        match &self.move_op {
            Some(mv_op) => {
                let mut delta = cursor_pos - mv_op.move_start;
                delta.y = -delta.y;
                let (offset_x,offset_y) = {
                    let rounded = delta.round();
                    (rounded.x as i32, rounded.y as i32)
                };
                self.pixels.shift(offset_x,offset_y);
            },
            _=> {},
        }
        // finally clear the move op so the renderer stops outputting offset pixels
        self.move_op = None;
    }

}

#[derive(Resource, Default)]
pub struct PalettesData(PaletteCollection<u8>);

#[derive(Resource, Default)]
pub struct EditorSettings {
    pub selected_color: ColorIndex,
    pub selected_palette: u8,
}

impl<const N: usize, const W: usize> Index<(usize, usize)> for PixelData<N,W> {
    type Output = ColorIndex;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.pixels.index(index)
    }
}

impl<const N: usize, const W: usize> IndexMut<(usize, usize)> for PixelData<N,W> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut ColorIndex {
        self.pixels.index_mut(index)
    }
}

// marker component for our main camera view
#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "RLE Pixel Editor (Pending Cool Name)".into(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),// set pixel art render mode,
        EguiPlugin,
    ))
    .init_resource::<OccupiedScreenSpace>()
    .init_resource::<CursorWorldCoords>()
    .init_resource::<PalettesData>()
    .insert_resource(EditorSettings {
        selected_color: ColorIndex::Dark,
        selected_palette: 0,
    })
    .insert_resource(CursorType::Pencil(ColorIndex::Dark))
    .insert_resource(PixelData {
        pixels: IndexedImage::<TOTAL_PIXELS,EDITOR_SIZE>::new(),
        ..default()
    })
    .add_systems(Startup, 
        (
            init_picture_render,
            setup_system,
            setup_grid,
        )
    )
    .add_systems(Update, (
        update_camera_transform_system,
        ui_controls_system,
        cursor_system,
        update_pen_color,
        input_system.after(cursor_system),
        update_pixels.after(input_system),
    ))
    .add_plugins(EncoderPlugin)
    .run();
}

fn input_system(
    mut pixels: ResMut<PixelData<TOTAL_PIXELS,EDITOR_SIZE>>,
    buttons: Res<Input<MouseButton>>,
    cursor: Res<CursorWorldCoords>,
    cursor_type: Res<CursorType>,
    mut egui: EguiContexts,
) {
    // don't handle input thats being sent to egui
    let ctx = egui.ctx_mut();
    if ctx.is_using_pointer()
    || ctx.is_pointer_over_area()
    || ctx.wants_pointer_input()
    {return;}

    
    match cursor_type.as_ref() {
        CursorType::Move => {
            // start move operation
            // and store starting position
            if buttons.just_pressed(MouseButton::Left) {
                pixels.start_move(cursor.0);
            } else if buttons.pressed(MouseButton::Left) {
                // update current pos on the move event
                pixels.update_move(cursor.0);
            }

            // end move operation
            if buttons.just_released(MouseButton::Left) {
                pixels.end_move(cursor.0)
            }

            // cancel move op with right click
            if buttons.just_pressed(MouseButton::Right) {
                pixels.cancel_move();
            }

        },
        _ => {},
    }

    if buttons.pressed(MouseButton::Left) {
        match world_to_grid(cursor.0) {
            Some((x,y)) => {
                match cursor_type.as_ref() {
                    CursorType::Pencil(p) => {
                        pixels[(x,y)] = *p;
                        eprintln!("Placing pixel at: {}/{}", x, y);

                    },
                    CursorType::Eraser => {
                        pixels[(x,y)] = ColorIndex::Empty;
                    },
                    CursorType::FillBucket(p) => {
                        // TODO impl fill algo
                        pixels[(x,y)] = *p;
                    },
                    _ => {},
                }
            },
            _=> {}
        }
    }
}

fn update_pen_color(
    editor_settings: Res<EditorSettings>,
    mut cursor_type: ResMut<CursorType>,
) {
    if editor_settings.is_changed() {
        match cursor_type.as_mut() {
            CursorType::Pencil(c) => {
                *c = editor_settings.selected_color;
            },
            CursorType::FillBucket(c) => {
                *c = editor_settings.selected_color;
            },
            _=>{},
        }
    }
}

fn ui_controls_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut cursor_type: ResMut<CursorType>,
    mut editor_settings: ResMut<EditorSettings>,
    mut rle_encoder_settings: ResMut<RLEncoderSettings>,
    rle_encoded_string: Res<RLEncodedString>,
    mut rle_encoded_string_submit: ResMut<RLEncodedStringSubmission>,
    rle_encoded_bytes: Res<RLEncodedBytes>,
    palette: Res<PalettesData>,
    mut input_rle_string: Local<String>,
) {
    let ctx = contexts.ctx_mut();

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Settings panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Inspector panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
        // .resizable(true)
        .show(ctx, |ui| {
            ui.label("Tools Panel");
            
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
    
                    }
                    if ui.button("Save").clicked() {
    
                    }
                });

                if let Some(button_style) = ui.style_mut().text_styles.get_mut(&egui::style::TextStyle::Button) {
                    *button_style = FontId::new(24.0, FontFamily::Proportional);
                }

                let set = editor_settings.as_mut();

                ui.separator();
                tool_selector(
                    ui,
                    &mut cursor_type,
                    &set.selected_color
                );
                ui.separator();

                // color selector
                color_index(
                    ui,
                    &mut set.selected_color,
                    ColorIndex::Empty,
                    "❌",
                    &palette.0[set.selected_palette]
                );
                color_index(
                    ui,
                    &mut set.selected_color,
                    ColorIndex::Dark,
                    "🎩",
                    &palette.0[set.selected_palette]
                );
                color_index(
                    ui,
                    &mut set.selected_color,
                    ColorIndex::Bright,
                    "🌞",
                    &palette.0[set.selected_palette]
                );
                color_index(
                    ui,
                    &mut set.selected_color,
                    ColorIndex::Skin,
                    "👨",
                    &palette.0[set.selected_palette]
                );
                color_index(
                    ui,
                    &mut set.selected_color,
                    ColorIndex::ShirtAccent1,
                    "👕",
                    &palette.0[set.selected_palette]
                );
                color_index(
                    ui,
                    &mut set.selected_color,
                    ColorIndex::PantsAccent2,
                    "👖",
                    &palette.0[set.selected_palette]
                );
                color_index(
                    ui,
                    &mut set.selected_color,
                    ColorIndex::EyesAccent3,
                    "👀",
                    &palette.0[set.selected_palette]
                );
                color_index(
                    ui,
                    &mut set.selected_color,
                    ColorIndex::Accent4,
                    "🎨",
                    &palette.0[set.selected_palette]
                );
            });

            // this must be absolutely last
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            {
                
                egui::Grid::new("parent grid").min_col_width(200.).striped(true).show(ui, |ui| {
                    // First Col
                    ui.vertical(|ui| {
                        ui.heading("RLE Encoder");
                        let vt = ui.label("Vertical Trim").id;
                        if ui.add(Slider::new(
                            &mut rle_encoder_settings.bypass_change_detection().vertical_trim,
                            0..=(EDITOR_SIZE as u8 -1)
                        )).labelled_by(vt).changed() {
                            rle_encoder_settings.set_changed();
                        }
                        if rle_encoded_bytes.0.bytes.len() > 0 {
                            let header_bits = format!(
                                "Header bits: {:#b} encoded_width: {} left_offset: {}",
                                rle_encoded_bytes.0.bytes[0],
                                rle_encoded_bytes.0.header_width,
                                rle_encoded_bytes.0.header_offset
                            );
                            ui.label(header_bits);
                        }
                    });
                    // Second Col

                    
                    ui.vertical(|ui| {
                        ui.heading("RLE Decoder");
                        TextEdit::singleline(input_rle_string.deref_mut()).hint_text("Enter RLE Hex String").show(ui);
                    });

                    if ui.button("Apply").clicked() {
                        // submit string to be decoded
                        rle_encoded_string_submit.0 = Some(input_rle_string.clone());
                    }

                    // move to next row
                    ui.end_row();
                });
            }

            ui.horizontal(|ui| {
                ui.label(format!("Hex Encoded ({} Bytes): ", rle_encoded_bytes.0.bytes.len()));
                if ui.button("📋").on_hover_text("Click to copy").clicked() {
                    ui.output_mut(|out| {
                        out.copied_text = rle_encoded_string.0.clone()
                    });
                }
                ui.add(TextEdit::singleline(&mut rle_encoded_string.0.as_str()).desired_width(f32::INFINITY))
            });
            
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
                1.,
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
                1.,
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
            scaling_mode: ScalingMode::AutoMin { min_width: EDITOR_SIZE as f32, min_height: EDITOR_SIZE as f32 },
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

    let viewport_offset = if let Some(viewport) = &camera.viewport {
        let pos = viewport.physical_position;
        Vec2::new(pos.x as f32, pos.y as f32)
    } else {
        Vec2::ZERO
    };

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor-viewport_offset))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
        // eprintln!("World coords: {}/{}", world_position.x, world_position.y);
    }
}

fn update_camera_transform_system(
    occupied_screen_space: Res<OccupiedScreenSpace>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<&mut Camera, With<MainCamera>>,
) {
    let mut camera = match camera_query.get_single_mut() {
        Ok(camera) => camera,
        _ => unreachable!(),
    };

    let window = windows.single();

    // return if window is not ready yet
    if window.width() <= 0. || window.height() <= 0. {return;}

    let left_taken = occupied_screen_space.left;// window.width();
    let right_taken = occupied_screen_space.right;// / window.width();
    let top_taken = occupied_screen_space.top;// / window.height();
    let bottom_taken = occupied_screen_space.bottom;// / window.height();

    let view_size =  UVec2::new(
        (window.width() - (left_taken + right_taken)) as u32,
        (window.height() - (top_taken + bottom_taken)) as u32
    );

    // don't set viewport to 0
    if view_size.x == 0 || view_size.y == 0 {return;}

    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(left_taken as u32,top_taken as u32),
        physical_size: view_size,
        ..default()
    });
}