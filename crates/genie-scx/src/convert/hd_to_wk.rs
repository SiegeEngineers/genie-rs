use super::ConvertError;
use crate::{Scenario, ScenarioObject, Tile, Trigger, UnitTypeID};
use nohash_hasher::IntMap;

/// Convert an HD Edition scenario to a WololoKingdoms-compatible one.
///
/// Maps HD unit IDs and terrain IDs to their WK equivalents.
pub struct HDToWK {
    object_ids_map: IntMap<i32, UnitTypeID>,
    terrain_ids_map: IntMap<u8, u8>,
}

impl Default for HDToWK {
    fn default() -> Self {
        let object_ids_map = [
            // NOTE: These are special to make the tech tree work
            (1103, 529), // Fire Galley, Fire Ship
            (529, 1103), // Fire Ship, Fire Galley
            (1104, 527), // Demolition Raft, Demolition Ship
            (527, 1104), // Demolition Ship, Demolition Raft
            (1001, 106), // Organ Gun, INFIL_D
            (1003, 114), // Elite Organ Gun, LNGBT_D
            (1006, 183), // Elite Caravel, TMISB
            (1007, 203), // Camel Archer, VDML
            (1009, 208), // Elite Camel Archer, TWAL
            (1010, 223), // Genitour, VFREP_D
            (1012, 230), // Elite Genitour, VMREP_D
            (1013, 260), // Gbeto, OLD-FISH3
            (1015, 418), // Elite Gbeto, TROCK
            (1016, 453), // Shotel Warrior, DOLPH4
            (1018, 459), // Elite Shotel Warrior, FISH5
            (1103, 467), // Fire Ship, Nonexistent
            (1105, 494), // Siege Tower, CVLRY_D
            (1104, 653), // Demolition Ship, HFALS_D
            (947, 699),  // Cutting Mangonel, HSUBO_D
            (948, 701),  // Cutting Onager, HWOLF_D
            (1079, 732), // Genitour placeholder, HKHAN_D
            (1021, 734), // Feitoria, Nonexistent
            (1120, 760), // Ballista Elephant, BHUSK_D
            (1155, 762), // Imperial Skirmisher, BHUSKX_D
            (1134, 766), // Elite Battle Ele, UPLUM_D
            (1132, 774), // Battle Elephant, UCONQ_D
            (1131, 782), // Elite Rattan Archer, HPOPE_D
            (1129, 784), // Rattan Archer, HWITCH_D
            (1128, 811), // Elite Arambai, HEROBOAR_D
            (1126, 823), // Arambai, BOARJ_D
            (1125, 830), // Elite Karambit, UWAGO_D
            (1123, 836), // Karambit, HORSW_D
            (946, 848),  // Noncut Ballista Elephant, TDONK_D
            (1004, 861), // Caravel, mkyby_D
            (1122, 891), // Elite Ballista Ele, SGTWR_D
        ]
        .iter()
        .map(|(a, b)| (*a, UnitTypeID::from(*b)))
        .collect();

        let terrain_ids_map = [
            (38, 33), // Snow Road, Snow Dirt
            (45, 38), // Cracked Earth, Snow Road
            (54, 11), // Mangrove Terrain
            (55, 20), // Mangrove Forest
            (50, 41), // Acacia Forest
            (49, 16), // Baobab Forest
            (11, 3),  // Dirt 2, Dirt 3
            (16, 0),  // Grass-ish, Grass
            (20, 19), // Oak Forest, Pine Forest
        ]
        .iter()
        .map(|(a, b)| (*a, *b))
        .collect();

        Self {
            object_ids_map,
            terrain_ids_map,
        }
    }
}

impl HDToWK {
    /// Convert an object from HD Edition to WK.
    ///
    /// This updates the object type IDs.
    fn convert_object(&self, object: &mut ScenarioObject) {
        if let Some(new_type) = self.object_ids_map.get(&object.object_type.into()) {
            object.object_type = *new_type;
        }
    }

    /// Convert a trigger from HD Edition to WK.
    ///
    /// This updates the object type IDs in trigger conditions and effects.
    fn convert_trigger(&self, trigger: &mut Trigger) {
        trigger.conditions_unordered_mut().for_each(|cond| {
            if let Some(new_type) = self.object_ids_map.get(&cond.unit_type().into()) {
                cond.set_unit_type(*new_type);
            }
            if let Some(new_type) = self.object_ids_map.get(&cond.object_type().into()) {
                cond.set_object_type(*new_type);
            }
        });
        trigger.effects_unordered_mut().for_each(|effect| {
            if let Some(new_type) = self.object_ids_map.get(&effect.unit_type().into()) {
                effect.set_unit_type(*new_type);
            }
            if let Some(new_type) = self.object_ids_map.get(&effect.object_type().into()) {
                effect.set_object_type(*new_type);
            }
        });
    }

    fn convert_terrain(&self, tile: &mut Tile) {
        if let Some(new_type) = self.terrain_ids_map.get(&tile.terrain) {
            tile.terrain = *new_type;
        }
    }

    /// Convert a scenario from HD to WK in-place.
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
