use std::collections::HashMap;
use crate::{
    Scenario,
    ScenarioObject,
    Trigger,
    Tile,
};
use super::ConvertError;

pub struct AoCToWK {
    object_ids_map: HashMap<i32, i32>,
    terrain_ids_map: HashMap<i8, i8>,
}

impl Default for AoCToWK {
    fn default() -> Self {
        let object_ids_map = [
            // NOTE: These are special to make the tech tree work
            (1103, 529), // Fire Galley, Fire Ship
            (529, 1103), // Fire Ship, Fire Galley
            (1104, 527), // Demolition Raft, Demolition Ship
            (527, 1104), // Demolition Ship, Demolition Raft
        ].iter().map(|(a, b)| (*a, *b)).collect();

        let terrain_ids_map = [
            (11, 3), // Dirt 2, Dirt 3
            (16, 0), // Grass-ish, Grass
            (20, 19), // Oak Forest, Pine Forest
        ].iter().map(|(a, b)| (*a, *b)).collect();

        Self {
            object_ids_map,
            terrain_ids_map,
        }
    }
}

impl AoCToWK {
    /// Convert an object from AoC to WK.
    ///
    /// This updates the object type IDs.
    fn convert_object(&self, object: &mut ScenarioObject) {
        if let Some(new_type) = self.object_ids_map.get(&i32::from(object.object_type)) {
            object.object_type = (*new_type) as i16;
        }
    }

    /// Convert a trigger from AoC to WK.
    ///
    /// This updates the object type IDs in trigger conditions and effects.
    fn convert_trigger(&self, trigger: &mut Trigger) {
        trigger.conditions_unordered_mut().for_each(|cond| {
            if let Some(new_type) = self.object_ids_map.get(&cond.unit_type()) {
                cond.set_unit_type(*new_type);
            }
            if let Some(new_type) = self.object_ids_map.get(&cond.object_type()) {
                cond.set_object_type(*new_type);
            }
        });
        trigger.effects_unordered_mut().for_each(|effect| {
            if let Some(new_type) = self.object_ids_map.get(&effect.unit_type()) {
                effect.set_unit_type(*new_type);
            }
            if let Some(new_type) = self.object_ids_map.get(&effect.object_type()) {
                effect.set_object_type(*new_type);
            }
        });
    }

    /// Convert a terrain tile from AoC to WK.
    fn convert_terrain(&self, tile: &mut Tile) {
        if let Some(new_type) = self.terrain_ids_map.get(&tile.terrain) {
            tile.terrain = *new_type;
        }
    }

    /// Convert a scenario from AoC to WK in-place.
    pub fn convert(&self, scen: &mut Scenario) -> Result<(), ConvertError> {
        for object in scen.objects_mut() {
            self.convert_object(object);
        }

        for tile in scen.map_mut().tiles_mut() {
            self.convert_terrain(tile);
        }

        if let Some(trigger_system) = scen.triggers_mut() {
            for trigger in trigger_system.triggers_unordered_mut() {
                self.convert_trigger(trigger);
            }
        }

        Ok(())
    }
}
