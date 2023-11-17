use bevy::{prelude::*, render::render_resource::{TextureDimension, Extent3d, TextureFormat}};
use image::{ImageBuffer, Pixel, RgbaImage, Rgba};
use sixteenbit_encoding::types::ColorIndex;
use crate::{EDITOR_SIZE, TOTAL_PIXELS, PixelData, PalettesData, EditorSettings};

pub mod encoder;

/// multiplier value defining how many sub pixels each of
///  our real pixels will have on the screen rendered version
const EXTRA_DISPLAY_PIXELS_MUL: usize = 6;

// handles drawing of the canvas when we edit it

#[derive(Resource)]
pub struct CanvasImage(RgbaImage, Handle<Image>);

/// System to initialize our canvas graphic
pub fn init_picture_render(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let img = ImageBuffer::from_pixel(
        (EDITOR_SIZE * EXTRA_DISPLAY_PIXELS_MUL) as u32,
       (EDITOR_SIZE * EXTRA_DISPLAY_PIXELS_MUL) as u32,
       image::Rgba([0,0,0,0])
   );

   let size = Extent3d {
        width: (EDITOR_SIZE * EXTRA_DISPLAY_PIXELS_MUL) as u32,
        height: (EDITOR_SIZE * EXTRA_DISPLAY_PIXELS_MUL) as u32,
        ..default()
    };

   let bevy_img = Image::new(
        size,
        TextureDimension::D2,
        img.to_vec(),
        TextureFormat::Rgba8UnormSrgb
    );
    // fill image with zero
    // img.resize(size);

    let canvas_handle = images.add(bevy_img);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some((
                EDITOR_SIZE as f32,
                EDITOR_SIZE as f32
            ).into()),
            ..default()
        },
        transform: Transform::from_translation(Vec3::ZERO),
        texture: canvas_handle.clone(),
        ..default()
    });

    commands.insert_resource(CanvasImage(
        img,
        canvas_handle
    ));
}

/// Re-draws the canvas view when pixels change
pub fn update_pixels(
    new_pixels: Res<PixelData<TOTAL_PIXELS,EDITOR_SIZE>>,
    palette: Res<PalettesData>,
    settings: Res<EditorSettings>,
    mut buffer: ResMut<CanvasImage>,
    mut images: ResMut<Assets<Image>>,
) {
    if new_pixels.is_changed() {
        for (x,y,p) in buffer.0.enumerate_pixels_mut() {

            let scale = EXTRA_DISPLAY_PIXELS_MUL as f32;

            let (offset_x, offset_y) = match &new_pixels.move_op {
                Some(mv_op) => {
                    let delta = (mv_op.move_end - mv_op.move_start) * scale;
                    (
                        (delta.x),
                        (-delta.y),
                    )
                },
                _=> {
                    (0.,0.)
                }
            };
            
            let sample_x = (((x as f32) - offset_x) / scale);
            let sample_y = (((y as f32) - offset_y) / scale);
            
            let color = if
            sample_x >= EDITOR_SIZE as f32
            || sample_x < 0.
            || sample_y >= EDITOR_SIZE as f32
            || sample_y < 0.
            {
                ColorIndex::Empty
            } else {
                // info!("Getting color from {sample_x} {sample_y}");
                new_pixels[(sample_x as usize, sample_y as usize)]
            };

            *p = match color {
                ColorIndex::Empty => {
                    Rgba([0,0,0,0])
                },
                color => {
                    palette.0[settings.selected_palette]
                        [color].to_rgba()
                },
            };
        }
        // push changes to bevy texture
        if let Some(img) = images.get_mut(&buffer.1) {
            img.data = buffer.0.to_vec()
        }
    }
}