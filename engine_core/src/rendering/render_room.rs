// engine_core/src/rendering/render_entities.rs
use crate::{
    animation::animation_system::CurrentFrame, assets::{
        asset_manager::AssetManager, 
        sprite::Sprite
    }, 
    constants::*, 
    ecs::{
        component::*, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, lighting::{
        light::Light, 
        light_system::LightSystem
    }, 
    tiles::tile::TileSprite, 
    world::room::Room
};
use std::collections::BTreeMap;
use macroquad::prelude::*;

pub fn render_entities(
    world_ecs: &WorldEcs,
    room: &Room,
    asset_manager: &mut AssetManager,
    lighting: &mut LightSystem,
    render_cam: &Camera2D,
) {
    // Cache the needed stores
    let sprite_store = world_ecs.get_store::<Sprite>();
    let frame_store = world_ecs.get_store::<CurrentFrame>();

    // Organize entities by layer
    let layer_map = collect_layer_map(world_ecs, room);

    // Contains the cameras needed for each pass
    let cams = lighting.render_cams(render_cam);

    // Draw each blocking texture in black to a white mask
    set_camera(&cams.mask_cam);
    clear_background(WHITE);
    gl_use_default_material();

    for (_z, (entities, lights)) in layer_map {
        // Reset before using them
        lighting.clear_light_buffers();

        // Draw geometry to the scene render target
        set_camera(&cams.scene_cam);

        for (entity, pos) in entities {
            draw_entity(
            world_ecs,
            asset_manager,
            frame_store,
            sprite_store,
            entity,
            *pos,
            );
        }
        
        let darkness = 0.0f32; // TODO expose to editor

        // Ambient pass
        lighting.ambient_mat.set_texture("tex", lighting.scene_rt.texture.clone());
        lighting.ambient_mat.set_uniform("Darkness", darkness);

        set_camera(&cams.ambient_cam);

        gl_use_material(&lighting.ambient_mat);
        draw_texture_ex(
            &lighting.scene_rt.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                ..Default::default()
            },
        );
        gl_use_default_material();
        
        // Spotlight pass
        if !lights.is_empty() {
            let light_count = lights.len(); 

            for i in 0..light_count {
                let (entity_pos, l) = &lights[i];
                let world_pos = entity_pos.position + l.pos;

                lighting.pos[i] = render_cam.world_to_screen(world_pos);
                lighting.radius[i] = world_distance_to_screen(render_cam, l.radius);
                lighting.spread[i] = world_distance_to_screen(render_cam, l.radius);
                lighting.color[i] = l.color;
                lighting.intensity[i] = l.intensity;
                lighting.alpha[i] = l.alpha;
                lighting.brightness[i] = l.brightness;
            }

            lighting.spot_mat.set_texture("tex", lighting.scene_rt.texture.clone());
            lighting.spot_mat.set_texture("light_mask", lighting.mask_rt.texture.clone());
            lighting.spot_mat.set_uniform("LightCount", light_count as i32);
            lighting.spot_mat.set_uniform_array("LightPos", &lighting.pos);
            lighting.spot_mat.set_uniform_array("LightColor", &lighting.color);
            lighting.spot_mat.set_uniform_array("LightIntensity", &lighting.intensity);
            lighting.spot_mat.set_uniform_array("LightRadius", &lighting.radius);
            lighting.spot_mat.set_uniform_array("LightSpread", &lighting.spread);
            lighting.spot_mat.set_uniform_array("LightAlpha", &lighting.alpha);
            lighting.spot_mat.set_uniform_array("LightBrightness", &lighting.brightness);
            lighting.spot_mat.set_uniform("ScreenWidth", screen_width());
            lighting.spot_mat.set_uniform("ScreenHeight", screen_height());
            lighting.spot_mat.set_uniform("Darkness", darkness);

            set_camera(&cams.spot_cam);

            gl_use_material(&lighting.spot_mat);
            draw_texture_ex(
                &lighting.spot_rt.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    ..Default::default()
                },
            );
            gl_use_default_material();
        }

        // Composite
        lighting.composite_mat.set_texture("ambient_tex", lighting.ambient_rt.texture.clone());
        lighting.composite_mat.set_texture("spot_tex", lighting.spot_rt.texture.clone());
        lighting.composite_mat.set_texture("glow_tex", lighting.glow_rt.texture.clone());

        // Draw everything
        set_default_camera();
        gl_use_material(&lighting.composite_mat);
        draw_texture_ex(
            &lighting.scene_rt.texture, // any of the three works for size
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                ..Default::default()
            },
        );
        gl_use_default_material();
        set_camera(render_cam);
    }
}

fn draw_entity(
    world_ecs: &WorldEcs,
    asset_manager: &mut AssetManager,
    frame_store: &ComponentStore<CurrentFrame>,
    sprite_store: &ComponentStore<Sprite>,
    entity: Entity,
    pos: Position,
) {
    let (width, height) = sprite_dimensions(world_ecs, asset_manager, entity);

    // Animate/Draw sprite
    if let Some(cf) = frame_store.get(entity) && asset_manager.contains(cf.sprite_id) {
        // Source rect = column/row * frame size
        let src = Rect::new(
            cf.col as f32 * cf.frame_size.x,
            cf.row as f32 * cf.frame_size.y,
            cf.frame_size.x,
            cf.frame_size.y,
        );
        let tex = asset_manager.get_texture_from_id(cf.sprite_id);
        draw_texture_ex(
            tex,
            pos.position.x + cf.offset.x,
            pos.position.y + cf.offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(width, height)),
                source: Some(src),
                ..Default::default()
            },
        );
        return;
    } else if let Some(sprite) = sprite_store.get(entity) {
        // No animation
        if asset_manager.contains(sprite.sprite_id) {
            let tex = asset_manager.get_texture_from_id(sprite.sprite_id);
            draw_texture_ex(
                tex,
                pos.position.x,
                pos.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(width, height)),
                    ..Default::default()
                },
            );
            return;
        }
    }

    // Fallback placeholder (no sprite or missing texture)
    draw_entity_placeholder(pos.position);
}

pub fn highlight_selected_entity(
    world_ecs: &WorldEcs,
    entity: Entity,
    asset_manager: &mut AssetManager,
    color: Color
) {
    let pos = match world_ecs.get_store::<Position>().get(entity) {
        Some(p) => p,
        None => return,
    };

    let (width, height) = sprite_dimensions(world_ecs, asset_manager, entity);

    draw_rectangle_lines(pos.position.x, pos.position.y, width, height, 2.0, color);
}

pub fn sprite_dimensions(
    world_ecs: &WorldEcs,
    asset_manager: &AssetManager,
    entity: Entity,
) -> (f32, f32) {
    let from_anim = world_ecs
        .get_store::<CurrentFrame>()
        .get(entity)
        .map(|cf| (cf.frame_size.x, cf.frame_size.y));

    let from_sprite = || {
        world_ecs
            .get_store::<Sprite>()
            .get(entity)
            .and_then(|spr| asset_manager.texture_size(spr.sprite_id))
    };

    from_anim.or_else(from_sprite).unwrap_or((TILE_SIZE, TILE_SIZE))
}

pub fn draw_entity_placeholder(pos: Vec2) {
    draw_rectangle(
        pos.x,
        pos.y,
        TILE_SIZE,
        TILE_SIZE,
        GREEN,
    );
}

/// Sorts entites by their z-layer and filters out entities that 
/// should not be drawn. BTreeMap automatically sorts keys.
fn collect_layer_map<'a>(
    world_ecs: &'a WorldEcs,
    room: &Room,
) -> BTreeMap<i32, (Vec<(Entity, &'a Position)>, Vec<(Position, &'a Light)>)> {
    let mut map: BTreeMap<i32, (Vec<(Entity, &Position)>, Vec<(Position, &Light)>)> = BTreeMap::new();

    let pos_store = world_ecs.get_store::<Position>();
    let tile_store = world_ecs.get_store::<TileSprite>();
    let cam_store = world_ecs.get_store::<RoomCamera>();
    let room_store = world_ecs.get_store::<CurrentRoom>();
    let layer_store = world_ecs.get_store::<Layer>();
    let light_store = world_ecs.get_store::<Light>();

    for (entity, pos) in &pos_store.data {
        // Skip tiles & camera
        if tile_store.get(*entity).is_some() || cam_store.get(*entity).is_some() {
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

        // Default layer is 0 if missing
        let z = layer_store
            .get(*entity)
            .map_or(0, |l| l.z);

        let entry = map.entry(z).or_default();
        entry.0.push((*entity, pos));

        // If the entity also has a Light component
        if let Some(l) = light_store.get(*entity) {
            entry.1.push((*pos, l));
        }
    }

    map
}

pub fn world_distance_to_screen(cam: &Camera2D, distance: f32) -> f32 {
    let scale = cam.zoom.x * screen_width() * 0.5; 
    (distance * scale).abs()
}