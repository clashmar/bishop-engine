// engine_core/src/rendering/render_room.rs
use crate::prelude::*;
use std::collections::{BTreeMap, HashMap};
use macroquad::prelude::*;

/// Draws everything needed for the given room.
pub fn render_room(
    ecs: &Ecs,
    room: &Room,
    asset_manager: &mut AssetManager,
    render_system: &mut RenderSystem,
    render_cam: &Camera2D,
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
    grid_size: f32,
) {
    let render_start = std::time::Instant::now();

    // Cache the needed stores
    let sprite_store = ecs.get_store::<Sprite>();
    let frame_store = ecs.get_store::<CurrentFrame>();
    let transform_store = ecs.get_store::<Transform>();

    // Organize entities by layer
    let mut layer_map = collect_interpolated_layer_map(
        ecs,
        room,
        asset_manager,
        alpha,
        prev_positions,
        grid_size,
    );

    if layer_map.is_empty() {
        layer_map.insert(0, (Vec::new(), Vec::new()));
    }

    // Clear composite textures before each run
    render_system.clear_cam(&render_system.scene_comp_rt.clone());
    render_system.clear_cam(&render_system.final_comp_rt.clone());

    // Draw each blocking texture in black onto a white background
    // To be implemented but it needs to happen BEFORE the loop
    render_system.init_mask_cam();

    // Flag for the tilemap
    let mut first_pass = true;
    let tilemap = &room.current_variant().tilemap;

    for (_z, (entities, glows)) in layer_map {
        // Scene cam needs to draw to the current render cam
        render_system.clear_scene_cam(render_cam);

        // Draw the tilemap as the first layer
        if first_pass {
            tilemap.draw(asset_manager, room.position, grid_size);
            first_pass = false;
        }

        // Draw all entities
        for (entity, pos) in entities {
            draw_entity(
                ecs,
                asset_manager,
                frame_store,
                sprite_store,
                transform_store,
                entity,
                pos,
                grid_size,
            );
        }

        // Render the darkened scene
        render_system.run_ambient_pass(room.darkness);

        // Render glow per layer
        render_system.run_glow_pass(render_cam, glows, asset_manager);

        // Render the undarkened room for lighting pass
        render_system.run_undarkened_pass();

        // Combine scene renders
        render_system.run_scene_pass();
    }

    // Lighting pass
    let lights = collect_lights(ecs, room, alpha, prev_positions);
    render_system.run_spotlight_pass(render_cam, lights, room.darkness);

    // Composite the final render
    render_system.run_final_pass();

    render_system.render_time_ms = render_start.elapsed().as_secs_f32() * 1000.0;
}

fn draw_entity(
    ecs: &Ecs,
    asset_manager: &mut AssetManager,
    frame_store: &ComponentStore<CurrentFrame>,
    sprite_store: &ComponentStore<Sprite>,
    transform_store: &ComponentStore<Transform>,
    entity: Entity,
    pos: Vec2,
    grid_size: f32,
) {
    // If this is a player proxy, render using Player's visual components
    let visual_entity = if ecs.has::<PlayerProxy>(entity) {
        ecs.get_player_entity().unwrap_or(entity)
    } else {
        entity
    };

    let (width, height) = entity_dimensions(ecs, asset_manager, visual_entity, grid_size);

    // Get pivot from transform (default to BottomCenter)
    let pivot = transform_store
        .get(entity)
        .map(|t| t.pivot)
        .unwrap_or(Pivot::BottomCenter);

    // Calculate pivot-adjusted draw position
    let draw_base = pivot_adjusted_position(pos, vec2(width, height), pivot);

    // Animate/Draw sprite (use visual_entity for sprite lookup)
    if let Some(cf) = frame_store.get(visual_entity) && asset_manager.contains(cf.sprite_id) {
        let tex = asset_manager.get_texture_from_id(cf.sprite_id);

        let frame_w = cf.frame_size.x;
        let frame_h = cf.frame_size.y;

        let src = Rect::new(
            cf.col as f32 * frame_w,
            cf.row as f32 * frame_h,
            frame_w,
            frame_h,
        );

        // Floor to be sure
        let draw_x = (draw_base.x + cf.offset.x).floor();
        let draw_y = (draw_base.y + cf.offset.y).floor();

        draw_texture_ex(
            tex,
            draw_x,
            draw_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(frame_w, frame_h)),
                source: Some(src),
                flip_x: cf.flip_x,
                ..Default::default()
            },
        );
        return;
    } else if let Some(sprite) = sprite_store.get(visual_entity) {
        // No animation
        if asset_manager.contains(sprite.sprite) {
            let tex = asset_manager.get_texture_from_id(sprite.sprite);
            draw_texture_ex(
                tex,
                draw_base.x,
                draw_base.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(width, height)),
                    ..Default::default()
                },
            );
            return;
        }
    }

    // Don't draw placeholders for these components
    if ecs.has_any::<(Light, Glow)>(visual_entity) {
        return;
    }

    // Fallback placeholder (no sprite or missing texture)
    draw_entity_placeholder(draw_base, grid_size);
}

/// Highlight a selected entity with a colored outline.
pub fn highlight_selected_entity(
    ecs: &Ecs,
    entity: Entity,
    asset_manager: &mut AssetManager,
    color: Color,
    grid_size: f32,
) {
    let transform = match ecs.get_store::<Transform>().get(entity) {
        Some(t) => t,
        None => return,
    };

    // If this is a proxy, use Player's visual components for dimensions
    let visual_entity = if ecs.has::<PlayerProxy>(entity) {
        ecs.get_player_entity().unwrap_or(entity)
    } else {
        entity
    };

    let (width, height) = entity_dimensions(ecs, asset_manager, visual_entity, grid_size);
    let draw_pos = pivot_adjusted_position(transform.position, vec2(width, height), transform.pivot);

    draw_rectangle_lines(draw_pos.x, draw_pos.y, width, height, 2.0, color);
}

/// Get the dimensions of an entity for rendering.
pub fn entity_dimensions(
    ecs: &Ecs,
    asset_manager: &AssetManager,
    entity: Entity,
    grid_size: f32,
) -> (f32, f32) {
    let from_anim = ecs
        .get_store::<CurrentFrame>()
        .get(entity)
        .map(|cf| (cf.frame_size.x, cf.frame_size.y));

    let from_sprite = || {
        ecs.get_store::<Sprite>()
            .get(entity)
            .and_then(|sprite| asset_manager.texture_size(sprite.sprite))
    };

    let from_glow = || {
        ecs.get_store::<Glow>()
            .get(entity)
            .and_then(|glow| asset_manager.texture_size(glow.sprite_id))
    };

    from_anim
        .or_else(from_sprite)
        .or_else(from_glow)
        .unwrap_or((grid_size, grid_size))
}

/// Draw a placeholder for an entity without a sprite.
pub fn draw_entity_placeholder(pos: Vec2, grid_size: f32) {
    draw_rectangle(pos.x, pos.y, grid_size, grid_size, GREEN);
}

/// Sorts entites by their z-layer, filters out entities that should not be
/// drawn and interpolates the draw positions. BTreeMap automatically sorts keys.
fn collect_interpolated_layer_map<'a>(
    ecs: &'a Ecs,
    room: &Room,
    asset_manager: &AssetManager,
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
    grid_size: f32,
) -> BTreeMap<i32, (Vec<(Entity, Vec2)>, Vec<(&'a Glow, Vec2)>)> {
    let mut map: BTreeMap<i32, (Vec<(Entity, Vec2)>, Vec<(&Glow, Vec2)>)> = BTreeMap::new();

    let trans_store = ecs.get_store::<Transform>();
    let cam_store = ecs.get_store::<RoomCamera>();
    let room_store = ecs.get_store::<CurrentRoom>();
    let layer_store = ecs.get_store::<Layer>();
    let glow_store = ecs.get_store::<Glow>();

    for (entity, transform) in &trans_store.data {
        // Skip invisible entities
        if !transform.visible {
            continue;
        }

        // Skip camera
        if cam_store.get(*entity).is_some() {
            continue;
        }

        // Filter by current room
        if let Some(cr) = room_store.get(*entity) {
            if cr.0 != room.id {
                continue;
            }
        } else {
            continue;
        }

        // Interpolate the draw position
        let draw_pos =
            interpolate_draw_position(*entity, transform.position, alpha, prev_positions);

        // Default layer is 0 if missing
        let z = layer_store.get(*entity).map_or(0, |l| l.z);

        let entry = map.entry(z).or_default();
        entry.0.push((*entity, draw_pos));

        // If the entity also has a Glow component, apply pivot to glow position
        if let Some(glow) = glow_store.get(*entity) {
            let glow_size = asset_manager
                .texture_size(glow.sprite_id)
                .map(|(w, h)| vec2(w, h))
                .unwrap_or(vec2(grid_size, grid_size));

            let glow_draw_pos = pivot_adjusted_position(draw_pos, glow_size, transform.pivot);
            entry.1.push((glow, glow_draw_pos));
        }
    }

    // There always needs to be at least one layer otherwise nothing will be drawn
    if map.is_empty() {
        map.insert(0, (Vec::new(), Vec::new()));
    }

    map
}

fn collect_lights(
    ecs: &Ecs,
    room: &Room, 
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
) -> Vec<(Vec2, Light)> {
    let mut lights: Vec<(Vec2, Light)> = Vec::new();

    let light_store = ecs.get_store::<Light>();
    let trans_store = ecs.get_store::<Transform>();
    let room_store = ecs.get_store::<CurrentRoom>();

    for (entity, light) in &light_store.data {
        // Filter by current room
        if let Some(current_room) = room_store.get(*entity) {
            if current_room.0 != room.id {
                continue;
            }
        } else {
            continue;
        }

        if let Some(pos) = trans_store.get(*entity) {
            // Skip invisible entities
            if !pos.visible {
                continue;
            }
            // Interpolate the draw position
            let draw_pos = interpolate_draw_position(
                *entity, 
                pos.position, 
                alpha, 
                prev_positions
            );

            lights.push((draw_pos, *light));
        }
    }

    lights
}

/// Returns the interpolated draw position or the current position.
fn interpolate_draw_position(
    entity: Entity,
    current_pos: Vec2, 
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
) -> Vec2 {
    if let Some(prev_map) = prev_positions {
        if let Some(prev_pos) = prev_map.get(&entity) {
            onscreen_debug!("{prev_pos}");
            let interpolated = lerp(*prev_pos, current_pos, alpha).round();
            onscreen_debug!("{interpolated}");
            interpolated
        }
        else {
            current_pos
        }
    } else {
        current_pos
    }
}

#[inline]
pub fn lerp(prev_pos: Vec2, current_pos: Vec2, alpha: f32) -> Vec2 {
    prev_pos * (1.0 - alpha) + current_pos * alpha
}

/// Calculates draw position adjusted for pivot.
/// Returns the top-left corner where the texture should be drawn.
#[inline]
pub fn pivot_adjusted_position(entity_pos: Vec2, texture_size: Vec2, pivot: Pivot) -> Vec2 {
    let offset = pivot.as_normalized();
    vec2(
        entity_pos.x - texture_size.x * offset.x,
        entity_pos.y - texture_size.y * offset.y,
    )
}