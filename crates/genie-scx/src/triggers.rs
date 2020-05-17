use crate::Result;
use crate::UnitTypeID;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{read_opt_u32, read_str, write_str};
use std::convert::TryInto;
use std::io::{Read, Write};

/// A trigger condition, describing when a trigger can fire.
#[derive(Debug, Default, Clone)]
pub struct TriggerCondition {
    condition_type: i32,
    properties: Vec<i32>,
}

impl TriggerCondition {
    /// Read a trigger condition from an input stream, with the given trigger system version.
    pub fn read_from(mut input: impl Read, version: f64) -> Result<Self> {
        let condition_type = input.read_i32::<LE>()?;
        let num_properties = if version > 1.0 {
            input.read_i32::<LE>()?
        } else {
            13
        };
        let mut properties = Vec::with_capacity(num_properties as usize);
        for _ in 0..num_properties {
            properties.push(input.read_i32::<LE>()?);
        }

        while properties.len() < 18 {
            properties.push(-1);
        }

        Ok(Self {
            condition_type,
            properties,
        })
    }

    /// Write this trigger condition to an output stream, with the given trigger system version.
    pub fn write_to(&self, mut output: impl Write, version: f64) -> Result<()> {
        output.write_i32::<LE>(self.condition_type)?;
        if version > 1.0 {
            output.write_i32::<LE>(self.properties.len() as i32)?;
            for value in &self.properties {
                output.write_i32::<LE>(*value)?;
            }
        } else {
            for i in 0..13 {
                output.write_i32::<LE>(*self.properties.get(i).unwrap_or(&0))?;
            }
        }

        Ok(())
    }

    /// Get the "amount" value for this trigger condition.
    pub fn amount(&self) -> i32 {
        self.properties[0]
    }

    /// Set the "amount" value for this trigger condition.
    pub fn set_amount(&mut self, amount: i32) {
        self.properties[0] = amount;
    }

    /// Get the "resource" value for this trigger condition.
    pub fn resource(&self) -> i32 {
        self.properties[1]
    }

    /// Set the "resource" value for this trigger condition.
    pub fn set_resource(&mut self, resource: i32) {
        self.properties[1] = resource;
    }

    /// Get the "primary object" value for this trigger condition.
    pub fn primary_object(&self) -> i32 {
        self.properties[2]
    }

    /// Set the "primary object" value for this trigger condition.
    pub fn set_primary_object(&mut self, primary_object: i32) {
        self.properties[2] = primary_object;
    }

    /// Get the "secondary object" value for this trigger condition.
    pub fn secondary_object(&self) -> i32 {
        self.properties[3]
    }

    /// Set the "secondary object" value for this trigger condition.
    pub fn set_secondary_object(&mut self, secondary_object: i32) {
        self.properties[3] = secondary_object;
    }

    /// Get the raw "unit type" value for this trigger condition.
    pub fn raw_unit_type(&self) -> i32 {
        self.properties[4]
    }

    /// Set the raw "unit type" value for this trigger condition.
    pub fn set_raw_unit_type(&mut self, unit_type: i32) {
        self.properties[4] = unit_type;
    }

    /// Get the "unit type" value for this trigger condition.
    pub fn unit_type(&self) -> UnitTypeID {
        self.properties[4].try_into().unwrap()
    }

    /// Set the "unit type" value for this trigger condition.
    pub fn set_unit_type(&mut self, unit_type: UnitTypeID) {
        self.properties[4] = unit_type.try_into().unwrap();
    }

    /// Get the "player ID" value for this trigger condition.
    pub fn player_id(&self) -> i32 {
        self.properties[5]
    }

    /// Set the "player ID" value for this trigger condition.
    pub fn set_player_id(&mut self, player_id: i32) {
        self.properties[5] = player_id;
    }

    /// Get the "Tech ID" value for this trigger condition.
    pub fn technology_id(&self) -> i32 {
        self.properties[6]
    }

    /// Set the "Tech ID" value for this trigger condition.
    pub fn set_technology_id(&mut self, technology_id: i32) {
        self.properties[6] = technology_id;
    }

    /// Get the "Timer" value for this trigger condition.
    pub fn timer(&self) -> i32 {
        self.properties[7]
    }

    /// Set the "Timer" value for this trigger condition.
    pub fn set_timer(&mut self, timer: i32) {
        self.properties[7] = timer;
    }

    /// Get the "Trigger ID" value for this trigger condition.
    pub fn trigger_id(&self) -> i32 {
        self.properties[8]
    }

    /// Set the "Trigger ID" value for this trigger condition.
    pub fn set_trigger_id(&mut self, trigger_id: i32) {
        self.properties[8] = trigger_id;
    }

    /// Get the area this trigger condition applies to.
    ///
    /// -1 values indicate no area is set.
    pub fn area(&self) -> (i32, i32, i32, i32) {
        (
            self.properties[9],
            self.properties[10],
            self.properties[11],
            self.properties[12],
        )
    }

    /// Set the area this trigger condition applies to.
    ///
    /// -1 values indicate no area is set.
    pub fn set_area(&mut self, area: (i32, i32, i32, i32)) {
        self.properties[9] = area.0;
        self.properties[10] = area.1;
        self.properties[11] = area.2;
        self.properties[12] = area.3;
    }

    /// Get the "unit group" value for this trigger condition.
    pub fn unit_group(&self) -> i32 {
        self.properties[13]
    }

    /// Set the "unit group" value for this trigger condition.
    pub fn set_unit_group(&mut self, unit_group: i32) {
        self.properties[13] = unit_group;
    }

    /// Get the "Object Type" value for this trigger condition.
    pub fn object_type(&self) -> UnitTypeID {
        self.properties[14].try_into().unwrap()
    }

    /// Set the "Object Type" value for this trigger condition.
    pub fn set_object_type(&mut self, object_type: UnitTypeID) {
        self.properties[14] = i32::from(object_type);
    }

    /// Get the "AI Signal" value for this trigger condition.
    pub fn ai_signal(&self) -> i32 {
        self.properties[15]
    }

    /// Set the "AI Signal" value for this trigger condition.
    pub fn set_ai_signal(&mut self, ai_signal: i32) {
        self.properties[15] = ai_signal;
    }

    /// Get the "Inverted" value for this trigger condition.
    pub fn inverted(&self) -> bool {
        self.properties[16] == 1
    }

    /// Set the "Inverted" value for this trigger condition.
    pub fn set_inverted(&mut self, inverted: i32) {
        self.properties[16] = inverted;
    }
}

/// A trigger effect, describing the response when a trigger fires.
#[derive(Debug, Default, Clone)]
pub struct TriggerEffect {
    effect_type: i32,
    properties: Vec<i32>,
    chat_text: Option<String>,
    audio_file: Option<String>,
    objects: Vec<i32>,
}

impl TriggerEffect {
    /// Read a trigger effect from an input stream, with the given trigger system version.
    pub fn read_from(mut input: impl Read, version: f64) -> Result<Self> {
        let effect_type = input.read_i32::<LE>()?;
        let num_properties = if version > 1.0 {
            input.read_i32::<LE>()?
        } else {
            16
        };
        let mut properties = Vec::with_capacity(num_properties as usize);
        for _ in 0..num_properties {
            properties.push(input.read_i32::<LE>()?);
        }

        while properties.len() < 24 {
            properties.push(-1);
        }

        let len = input.read_i32::<LE>()? as usize;
        let chat_text = read_str(&mut input, len)?;
        let len = input.read_i32::<LE>()? as usize;
        let audio_file = read_str(&mut input, len)?;
        let mut objects = vec![];

        if version > 1.1 {
            for _ in 0..properties[4] {
                objects.push(input.read_i32::<LE>()?);
            }
        } else {
            objects.push(properties[4]);
            properties[4] = 1;
        }

        Ok(Self {
            effect_type,
            properties,
            chat_text,
            audio_file,
            objects,
        })
    }

    /// Write a trigger effect to an output stream, with the given trigger system version.
    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_i32::<LE>(self.effect_type)?;
        output.write_i32::<LE>(self.properties.len() as i32)?;
        for value in &self.properties {
            output.write_i32::<LE>(*value)?;
        }

        Ok(())
    }

    /// Get the "AI Goal" value for this trigger effect.
    pub fn ai_goal(&self) -> i32 {
        self.properties[0]
    }

    /// Set the "AI Goal" value for this trigger effect.
    pub fn set_ai_goal(&mut self, ai_goal: i32) {
        self.properties[0] = ai_goal;
    }

    /// Get the "Amount" value for this trigger effect.
    pub fn amount(&self) -> i32 {
        self.properties[1]
    }

    /// Set the "Amount" value for this trigger effect.
    pub fn set_amount(&mut self, amount: i32) {
        self.properties[1] = amount;
    }

    /// Get the "Resource" value for this trigger effect.
    pub fn resource(&self) -> i32 {
        self.properties[2]
    }

    /// Set the "Resource" value for this trigger effect.
    pub fn set_resource(&mut self, resource: i32) {
        self.properties[2] = resource;
    }

    /// Get the "Diplomacy" value for this trigger effect.
    pub fn diplomacy(&self) -> i32 {
        self.properties[3]
    }

    /// Set the "Diplomacy" value for this trigger effect.
    pub fn set_diplomacy(&mut self, diplomacy: i32) {
        self.properties[3] = diplomacy;
    }

    /// Get the "Number of Objects" value for this trigger effect.
    pub fn num_objects(&self) -> i32 {
        self.properties[4]
    }

    /// Set the "Number of Objects" value for this trigger effect.
    pub fn set_num_objects(&mut self, num_objects: i32) {
        self.properties[4] = num_objects;
    }

    /// Get the "Object ID" value for this trigger effect.
    pub fn object_id(&self) -> i32 {
        self.properties[5]
    }

    /// Set the "Object ID" value for this trigger effect.
    pub fn set_object_id(&mut self, object_id: i32) {
        self.properties[5] = object_id;
    }

    /// Get the "Unit Type" value for this trigger effect.
    pub fn unit_type(&self) -> UnitTypeID {
        self.properties[6].try_into().unwrap()
    }

    /// Set the "Unit Type" value for this trigger effect.
    pub fn set_unit_type(&mut self, unit_type: UnitTypeID) {
        self.properties[6] = i32::from(unit_type);
    }

    /// Get the "Source Player" value for this trigger effect.
    pub fn source_player_id(&self) -> i32 {
        self.properties[7]
    }

    /// Set the "Source Player" value for this trigger effect.
    pub fn set_source_player_id(&mut self, source_player_id: i32) {
        self.properties[7] = source_player_id;
    }

    /// Get the "Target Player" value for this trigger effect.
    pub fn target_player_id(&self) -> i32 {
        self.properties[8]
    }

    /// Set the "Target Player" value for this trigger effect.
    pub fn set_target_player_id(&mut self, target_player_id: i32) {
        self.properties[8] = target_player_id;
    }

    /// Get the "Tech ID" value for this trigger effect.
    pub fn technology_id(&self) -> i32 {
        self.properties[9]
    }

    /// Set the "Tech ID" value for this trigger effect.
    pub fn set_technology_id(&mut self, technology_id: i32) {
        self.properties[9] = technology_id;
    }

    /// Get the "Text ID" value for this trigger effect.
    pub fn text_id(&self) -> i32 {
        self.properties[10]
    }

    /// Set the "Text ID" value for this trigger effect.
    pub fn set_text_id(&mut self, text_id: i32) {
        self.properties[10] = text_id;
    }

    /// Get the "Sound ID" value for this trigger effect.
    pub fn sound_id(&self) -> i32 {
        self.properties[11]
    }

    /// Set the "Sound ID" value for this trigger effect.
    pub fn set_sound_id(&mut self, sound_id: i32) {
        self.properties[11] = sound_id;
    }

    /// Get the "Timer" value for this trigger effect.
    pub fn timer(&self) -> i32 {
        self.properties[12]
    }

    /// Set the "Timer" value for this trigger effect.
    pub fn set_timer(&mut self, timer: i32) {
        self.properties[12] = timer;
    }

    /// Get the "Trigger ID" value for this trigger effect.
    pub fn trigger_id(&self) -> i32 {
        self.properties[13]
    }

    /// Set the "Trigger ID" value for this trigger effect.
    pub fn set_trigger_id(&mut self, trigger_id: i32) {
        self.properties[13] = trigger_id;
    }

    /// Get the location this trigger effect applies to.
    pub fn location(&self) -> (i32, i32) {
        (self.properties[14], self.properties[15])
    }

    /// Set the location this trigger effect applies to.
    pub fn set_location(&mut self, location: (i32, i32)) {
        self.properties[14] = location.0;
        self.properties[15] = location.1;
    }

    /// Get the area this trigger effect applies to.
    pub fn area(&self) -> (i32, i32, i32, i32) {
        (
            self.properties[16],
            self.properties[17],
            self.properties[18],
            self.properties[19],
        )
    }

    /// Set the area this trigger effect applies to.
    pub fn set_area(&mut self, area: (i32, i32, i32, i32)) {
        self.properties[16] = area.0;
        self.properties[17] = area.1;
        self.properties[18] = area.2;
        self.properties[19] = area.3;
    }

    /// Get the "Object Group" value for this trigger effect.
    pub fn object_group(&self) -> i32 {
        self.properties[20]
    }

    /// Set the "Object Group" value for this trigger effect.
    pub fn set_object_group(&mut self, object_group: i32) {
        self.properties[20] = object_group;
    }

    /// Get the "Object Type" value for this trigger effect.
    pub fn object_type(&self) -> UnitTypeID {
        self.properties[21].try_into().unwrap()
    }

    /// Set the "Object Type" value for this trigger effect.
    pub fn set_object_type(&mut self, object_type: UnitTypeID) {
        self.properties[21] = i32::from(object_type);
    }

    /// Get the "Line ID" value for this trigger effect.
    pub fn line_id(&self) -> i32 {
        self.properties[22]
    }

    /// Set the "Line ID" value for this trigger effect.
    pub fn set_line_id(&mut self, line_id: i32) {
        self.properties[22] = line_id;
    }

    /// Get the "Stance" value for this trigger effect.
    pub fn stance(&self) -> i32 {
        self.properties[23]
    }

    /// Set the "Stance" value for this trigger effect.
    pub fn set_stance(&mut self, stance: i32) {
        self.properties[23] = stance;
    }
}

/// A trigger, describing automatic interactive behaviours in a scenario.
#[derive(Debug, Clone)]
pub struct Trigger {
    enabled: bool,
    looping: bool,
    name_id: i32,
    is_objective: bool,
    objective_order: i32,
    start_time: u32,
    description: Option<String>,
    short_description: Option<String>,
    name: Option<String>,
    effects: Vec<TriggerEffect>,
    effect_order: Vec<i32>,
    conditions: Vec<TriggerCondition>,
    condition_order: Vec<i32>,
}

impl Trigger {
    /// Read a trigger from an input stream, with the given trigger system version.
    pub fn read_from(mut input: impl Read, version: f64) -> Result<Self> {
        let enabled = input.read_i32::<LE>()? != 0;
        let looping = input.read_i8()? != 0;
        let name_id = input.read_i32::<LE>()?;
        let is_objective = input.read_i8()? != 0;
        let objective_order = input.read_i32::<LE>()?;

        let start_time;
        if version >= 1.8 {
            start_time = 0;
            let _make_header = input.read_u8()?;
            let _short_string_id: Option<u32> = read_opt_u32(&mut input)?;
            let _display = input.read_u8()?;
            let mut _unknown = [0; 5];
            input.read_exact(&mut _unknown)?;
            let _mute = input.read_u8()?;
        } else {
            start_time = input.read_u32::<LE>()?;
        }

        let description = {
            let len = input.read_u32::<LE>()? as usize;
            read_str(&mut input, len)?
        };

        let name = {
            let len = input.read_u32::<LE>()? as usize;
            read_str(&mut input, len)?
        };

        let short_description = if version >= 1.8 {
            let len = input.read_u32::<LE>()? as usize;
            read_str(&mut input, len)?
        } else {
            None
        };

        let num_effects = input.read_i32::<LE>()?;
        let mut effects = vec![];
        let mut effect_order = vec![];
        for _ in 0..num_effects {
            effects.push(TriggerEffect::read_from(&mut input, version)?);
        }
        for _ in 0..num_effects {
            effect_order.push(input.read_i32::<LE>()?);
        }

        let num_conditions = input.read_i32::<LE>()?;
        let mut conditions = vec![];
        let mut condition_order = vec![];
        for _ in 0..num_conditions {
            conditions.push(TriggerCondition::read_from(&mut input, version)?);
        }
        for _ in 0..num_conditions {
            condition_order.push(input.read_i32::<LE>()?);
        }

        Ok(Trigger {
            enabled,
            looping,
            name_id,
            is_objective,
            objective_order,
            start_time,
            description,
            short_description,
            name,
            effects,
            effect_order,
            conditions,
            condition_order,
        })
    }

    /// Write this trigger condition to an output stream, with the given trigger system version.
    pub fn write_to(&self, mut output: impl Write, version: f64) -> Result<()> {
        output.write_i32::<LE>(if self.enabled { 1 } else { 0 })?;
        output.write_i8(if self.looping { 1 } else { 0 })?;
        output.write_i32::<LE>(self.name_id)?;
        output.write_i8(if self.is_objective { 1 } else { 0 })?;
        output.write_i32::<LE>(self.objective_order)?;
        output.write_u32::<LE>(self.start_time)?;

        if let Some(descr) = &self.description {
            output.write_u32::<LE>(descr.len().try_into().unwrap())?;
            write_str(&mut output, descr)?;
        } else {
            output.write_u32::<LE>(0)?;
        }
        if let Some(name) = &self.name {
            output.write_u32::<LE>(name.len().try_into().unwrap())?;
            write_str(&mut output, name)?;
        } else {
            output.write_u32::<LE>(0)?;
        }

        output.write_u32::<LE>(self.effects.len() as u32)?;
        for effect in &self.effects {
            effect.write_to(&mut output)?;
        }
        for order in &self.effect_order {
            output.write_i32::<LE>(*order)?;
        }
        output.write_u32::<LE>(self.conditions.len() as u32)?;
        for condition in &self.conditions {
            condition.write_to(&mut output, version)?;
        }
        for order in &self.condition_order {
            output.write_i32::<LE>(*order)?;
        }

        Ok(())
    }

    /// Get the conditions in this trigger, in display order.
    pub fn conditions(&self) -> impl Iterator<Item = &TriggerCondition> {
        self.condition_order
            .iter()
            .map(move |index| &self.conditions[*index as usize])
    }

    /// Get the conditions in this trigger, unordered.
    pub fn conditions_unordered_mut(&mut self) -> impl Iterator<Item = &mut TriggerCondition> {
        self.conditions.iter_mut()
    }

    /// Get the effects in this trigger, in display order.
    pub fn effects(&self) -> impl Iterator<Item = &TriggerEffect> {
        self.effect_order
            .iter()
            .map(move |index| &self.effects[*index as usize])
    }

    /// Get the effects in this trigger, unordered.
    pub fn effects_unordered_mut(&mut self) -> impl Iterator<Item = &mut TriggerEffect> {
        self.effects.iter_mut()
    }
}

/// The trigger system maintains an ordered list  of triggers.
#[derive(Debug, Clone)]
pub struct TriggerSystem {
    version: f64,
    objectives_state: i8,
    triggers: Vec<Trigger>,
    trigger_order: Vec<i32>,
}

impl Default for TriggerSystem {
    fn default() -> Self {
        Self {
            version: 1.6,
            objectives_state: 0,
            triggers: vec![],
            trigger_order: vec![],
        }
    }
}

impl TriggerSystem {
    /// Read a trigger system from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let version = input.read_f64::<LE>()?;
        log::debug!("Trigger system version {}", version);
        let objectives_state = if version >= 1.5 { input.read_i8()? } else { 0 };

        let num_triggers = input.read_i32::<LE>()?;
        log::debug!("{} triggers", num_triggers);

        let mut triggers = vec![];
        let mut trigger_order = vec![];
        for _ in 0..num_triggers {
            triggers.push(Trigger::read_from(&mut input, version)?);
        }
        if version >= 1.4 {
            for _ in 0..num_triggers {
                trigger_order.push(input.read_i32::<LE>()?);
            }
        } else {
            for i in 0..num_triggers {
                trigger_order.push(i);
            }
        }

        Ok(Self {
            version,
            objectives_state,
            triggers,
            trigger_order,
        })
    }

    /// Write the trigger system to an output stream with the given system version.
    pub fn write_to(&self, mut output: impl Write, version: f64) -> Result<()> {
        output.write_f64::<LE>(version)?;
        if version >= 1.5 {
            output.write_i8(self.objectives_state)?;
        }
        output.write_u32::<LE>(self.triggers.len().try_into().unwrap())?;
        for trigger in &self.triggers {
            trigger.write_to(&mut output, version)?;
        }
        if version >= 1.4 {
            for order in &self.trigger_order {
                output.write_i32::<LE>(*order)?;
            }
        }
        Ok(())
    }

    /// Get the version of the trigger system data.
    pub fn version(&self) -> f64 {
        self.version
    }

    /// Get the number of triggers in the trigger system.
    pub fn num_triggers(&self) -> u32 {
        self.triggers.len() as u32
    }

    /// Iterate over all triggers, in order.
    pub fn triggers(&self) -> impl Iterator<Item = &Trigger> {
        self.trigger_order
            .iter()
            .map(move |index| &self.triggers[*index as usize])
    }

    /// Iterate over all triggers, mutably and unordered.
    pub fn triggers_unordered_mut(&mut self) -> impl Iterator<Item = &mut Trigger> {
        self.triggers.iter_mut()
    }
}
