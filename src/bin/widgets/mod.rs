use bevy_egui::egui::{Ui, RichText, Color32};
use sixteenbit_encoding::types::{ColorIndex, Palette};

use crate::CursorType;



pub fn color_index(
    ui: &mut Ui,
    selected_color: &mut ColorIndex,
    alt: ColorIndex,
    text: &str,
    palette: &Palette<u8>
) {
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

pub fn selector_button(
    ui: &mut Ui,
    selected_tool: &mut CursorType,
    alt: CursorType,
    text: &str,
) {

    let color = if alt == *selected_tool {
        Color32::from_rgb(75, 75, 100)
    } else {
        Color32::from_rgb(75, 75, 75)
    };

    if ui.button(
        RichText::new(text).color(color)
    ).clicked() {
        *selected_tool = alt;
    }
}

pub fn tool_selector(
    ui: &mut Ui,
    selected_tool: &mut CursorType,
    selected_color: & ColorIndex,
) {
    selector_button(
        ui,
        selected_tool,
        CursorType::Pencil(*selected_color),
        "✏ Pencil"
    );
    selector_button(
        ui,
        selected_tool,
        CursorType::Eraser,
        "🗑 Eraser"
    );
    selector_button(
        ui,
        selected_tool,
        CursorType::Move,
        "🤚 Move"
    );
}