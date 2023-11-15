use bevy_egui::egui::{Ui, RichText, Color32};
use sixteenbit_encoding::types::{ColorIndex, Palette};



pub fn color_index(ui: &mut Ui, selected_color: &mut ColorIndex, alt: ColorIndex, text: &str, palette: &Palette<u8>) {
    let color = if alt == ColorIndex::Empty {
        Color32::from_rgb(90, 75, 75)
    } else {
        let col = palette[alt];
        Color32::from_rgb(col.0[0], col.0[1], col.0[2])
    };
    ui.radio_value(
        selected_color,
        alt,
        RichText::new(text).color(color)
    );
}