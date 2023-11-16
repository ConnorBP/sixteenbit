use bevy::prelude::*;
use crate::{TOTAL_PIXELS, EDITOR_SIZE, PixelData};
use sixteenbit_encoding::encodings::OneByteRle;

pub struct EncoderPlugin;

impl Plugin for EncoderPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app
        .init_resource::<RLEncodedBytes>()
        .init_resource::<RLEncodedString>()
        // .add_systems(Startup, setup_encoder)
        .add_systems(Update, update_encoding);
    }
}

/// The most recently generated encoding of the canvas,
/// stored as a raw byte vec
#[derive(Resource, Default)]
pub struct RLEncodedBytes(pub Vec<u8>);

/// The most recently generated encoding of the canvas,
/// displayed as a hex string.
#[derive(Resource, Default)]
pub struct RLEncodedString(pub String);

// pub fn setup_encoder(mut commands: Commands) {
//     commands.spawn()
// }

/// Encodes the canvas view with RLE when pixels change
pub fn update_encoding(
    new_pixels: Res<PixelData<TOTAL_PIXELS,EDITOR_SIZE>>,
    mut encoded_bytes: ResMut<RLEncodedBytes>,
    mut encoded_string: ResMut<RLEncodedString>,
) {
    if new_pixels.is_changed() {
        // encode new pixels
        let encoder: OneByteRle = (&new_pixels.0).into();

        encoded_bytes.0 = encoder.bytes();
        encoded_string.0 = hex::encode(&encoded_bytes.0);
    }
}