// engine_core/src/script/script_system.rs
use crate::script::script::Script;
use crate::ecs::world_ecs::WorldEcs;
use crate::script::script_manager::ScriptManager;

pub fn run_scripts(
    dt: f32, 
    world_ecs: &mut WorldEcs, 
    script_manager: &mut ScriptManager
) -> mlua::Result<()> {
    let script_store = world_ecs.get_store_mut::<Script>();

    for (_entity, script) in script_store.data.iter_mut() {
        // Ensure the script table is loaded
        if script.table.is_none() && script.script_id.0 != 0 {
            script.load(script_manager)?
        }

        if let Some(update) = &script.update_fn {
            let table = script.table.as_ref().unwrap();
            update.call::<()>((table, dt))?
        }
    }

    Ok(())
}