use bevy::math::Vec2;

use crate::EDITOR_SIZE;



// takes in a grid position from 0 to editor_size and outputs a world coordinate
pub fn grid_to_world(grid_pos: (u32,u32)) -> Vec2 {
    Vec2::new(
        (grid_pos.0 as f32 - EDITOR_SIZE as f32 / 2.)+0.5,
        (grid_pos.1 as f32 - EDITOR_SIZE as f32 / 2.)+0.5,
    )
}

pub fn world_to_grid(world_pos: Vec2) -> Option<(usize,usize)> {
    
    let x = ((world_pos.x) + EDITOR_SIZE as f32 / 2.) as i32;
    // y is flipped for some reason
    let y = ((-world_pos.y) + EDITOR_SIZE as f32 / 2.) as i32;

    // return in bounds result or None
    if x < 0 || x >= EDITOR_SIZE as i32
    || y < 0 || y >= EDITOR_SIZE as i32 {
        return None;
    } else {
        return Some((x as usize, y as usize))
    }
}