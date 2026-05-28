//! Browser localStorage persistence for save/load.

use coredumped_core::save::SaveData;
use coredumped_core::world::World;
use web_sys::window;

fn local_storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}

pub fn save_world(slot: u32, world: &World) -> Result<(), String> {
    let storage = local_storage().ok_or("localStorage unavailable")?;
    let data = world.to_save_data();
    let json = serde_json::to_string(&data).map_err(|e| e.to_string())?;
    storage
        .set_item(&format!("save-slot-{}", slot), &json)
        .map_err(|_| "storage write failed".to_string())
}

pub fn load_world(slot: u32) -> Result<World, String> {
    let storage = local_storage().ok_or("localStorage unavailable")?;
    let json = storage
        .get_item(&format!("save-slot-{}", slot))
        .map_err(|_| "storage read failed")?
        .ok_or("no save found")?;
    let data: SaveData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(World::from_save_data(&data))
}

pub fn has_save(slot: u32) -> bool {
    local_storage()
        .and_then(|s| s.get_item(&format!("save-slot-{}", slot)).ok())
        .flatten()
        .is_some()
}

pub fn delete_save(slot: u32) -> Result<(), String> {
    let storage = local_storage().ok_or("localStorage unavailable")?;
    storage
        .remove_item(&format!("save-slot-{}", slot))
        .map_err(|_| "storage delete failed".to_string())
}
