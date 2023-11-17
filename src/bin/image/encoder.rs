use bevy::prelude::*;
use crate::{TOTAL_PIXELS, EDITOR_SIZE, PixelData};
use sixteenbit_encoding::encodings::{OneByteRle, rle_on_indexed};

pub struct EncoderPlugin;

impl Plugin for EncoderPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app
        .init_resource::<RLEncodedBytes>()
        .init_resource::<RLEncodedString>()
        .init_resource::<RLEncoderSettings>()
        .init_resource::<RLEncodedStringSubmission>()
        .add_systems(Update,
            (
            update_encoding,
            update_rle_input,
            )
        );
    }
}

#[derive(Resource, Default)]
pub struct RLEncoderSettings {
    /// defines how many rows to trim off the top of the image before encoding.
    /// Useful for drawing in place assets that we know will always have a fixed offset
    /// Ex. Pants
    pub vertical_trim: u8,
}

/// The most recently generated encoding of the canvas,
/// stored as a raw byte vec
#[derive(Resource, Default)]
pub struct RLEncodedBytes(pub OneByteRle);

/// When this is set, we decode the RLE bytes from the string
/// and then we apply them to the current canvas.
#[derive(Resource, Default)]
pub struct RLEncodedStringSubmission(pub Option<String>);

/// The most recently generated encoding of the canvas,
/// displayed as a hex string.
#[derive(Resource, Default)]
pub struct RLEncodedString(pub String);

/// Encodes the canvas view with RLE when pixels change
pub fn update_encoding(
    mut new_pixels: ResMut<PixelData<TOTAL_PIXELS,EDITOR_SIZE>>,
    mut encoded_bytes: ResMut<RLEncodedBytes>,
    mut encoded_string: ResMut<RLEncodedString>,
    rle_encoder_settings: Res<RLEncoderSettings>,
) {
    if new_pixels.is_changed()
    || rle_encoder_settings.is_changed()
    {
        // apply the trim value
        new_pixels.pixels.vertical_trim = rle_encoder_settings.vertical_trim;
        // encode new pixels
        let encoder: OneByteRle = (&new_pixels.pixels).into();

        encoded_bytes.0 = encoder;
        encoded_string.0 = hex::encode(&encoded_bytes.0.bytes);
    }
}

/// Writes pixels from RLE strings onto the canvas when RLEncodedStringSubmission is Some
pub fn update_rle_input(
    rle_encoder_settings: Res<RLEncoderSettings>,
    new_rle: Res<RLEncodedStringSubmission>,
    mut canvas_indexed_pixels: ResMut<PixelData<TOTAL_PIXELS,EDITOR_SIZE>>,
) {
    if new_rle.is_changed() {
        if let Some(hex_str) = &new_rle.0 {
            match hex::decode(hex_str.trim_matches(' '))
            .map(OneByteRle::new_with_bytes) {
                Ok(Some(decoder)) => {
                    // apply the RLE bytes to our indexed canvas
                    rle_on_indexed(
                        &mut canvas_indexed_pixels.pixels,
                        &decoder,
                        rle_encoder_settings.vertical_trim,
                        false // overlap input onto canvas
                    );
                },
                Err(e) => {
                    error!("RLE Decode Error: {e}");
                }
                _=> {
                    // decode failed
                    warn!("RLE Decode Failed.");
                }
            }
        }
    }
}