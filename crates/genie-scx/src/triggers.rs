use crate::Result;
use crate::UnitTypeID;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{read_opt_u32, write_i32_str, write_opt_i32_str, ReadStringsExt, StringKey};
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
    pub fn unit_type(&self) -> Option<UnitTypeID> {
        match self.properties[4] {
            -1 => None,
            unit_type => Some(unit_type.try_into().unwrap()),
        }
    }

    /// Set the "unit type" value for this trigger condition.
    pub fn set_unit_type(&mut self, unit_type: Option<UnitTypeID>) {
        self.properties[4] = match unit_type {
            Some(unit_type) => i32::from(unit_type),
            None => -1,
        };
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
    pub fn object_type(&self) -> Option<UnitTypeID> {
        match self.properties[14] {
            -1 => None,
            object_type => Some(object_type.try_into().unwrap()),
        }
    }

    /// Set the "Object Type" value for this trigger condition.
    pub fn set_object_type(&mut self, object_type: Option<UnitTypeID>) {
        self.properties[14] = match object_type {
            Some(object_type) => i32::from(object_type),
            None => -1,
        };
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

        let chat_text = input.read_u32_length_prefixed_str()?;
        let audio_file = input.read_u32_length_prefixed_str()?;
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
    pub fn write_to(&self, mut output: impl Write, version: f64) -> Result<()> {
        output.write_i32::<LE>(self.effect_type)?;
        output.write_i32::<LE>(self.properties.len() as i32)?;
        for value in &self.properties {
            output.write_i32::<LE>(*value)?;
        }

        write_opt_i32_str(&mut output, &self.chat_text)?;
        write_opt_i32_str(&mut output, &self.audio_file)?;

        if version > 1.1 {
            for i in 0..self.num_objects() {
                output.write_i32::<LE>(*self.objects.get(i as usize).unwrap_or(&-1))?;
            }
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
    pub fn unit_type(&self) -> Option<UnitTypeID> {
        match self.properties[6] {
            -1 => None,
            unit_type => Some(unit_type.try_into().unwrap()),
        }
    }

    /// Set the "Unit Type" value for this trigger effect.
    pub fn set_unit_type(&mut self, unit_type: Option<UnitTypeID>) {
        self.properties[6] = match unit_type {
            Some(unit_type) => i32::from(unit_type),
            None => -1,
        };
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
    pub fn object_type(&self) -> Option<UnitTypeID> {
        match self.properties[21] {
            -1 => None,
            object_type => Some(object_type.try_into().unwrap()),
        }
    }

    /// Set the "Object Type" value for this trigger effect.
    pub fn set_object_type(&mut self, object_type: Option<UnitTypeID>) {
        self.properties[21] = match object_type {
            Some(object_type) => i32::from(object_type),
            None => -1,
        };
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
    short_description_id: Option<StringKey>,
    short_description: Option<String>,
    display_short_description: bool,
    short_description_state: u8,
    mute_objective: bool,
    name: Option<String>,
    effects: Vec<TriggerEffect>,
    effect_order: Vec<i32>,
    conditions: Vec<TriggerCondition>,
    condition_order: Vec<i32>,
    make_header: bool,
}

impl Trigger {
    /// Read a trigger from an input stream, with the given trigger system version.
    pub fn read_from(mut input: impl Read, version: f64) -> Result<Self> {
        let enabled = input.read_i32::<LE>()? != 0;
        let looping = input.read_i8()? != 0;
        let name_id = input.read_i32::<LE>()?;
        let is_objective = input.read_i8()? != 0;
        let objective_order = input.read_i32::<LE>()?;

        let mut make_header = false;
        let mut short_description_id = None;
        let mut short_description_state = 0;
        let mut display_short_description = false;
        let mut mute_objective = false;
        let start_time;
        if version >= 1.8 {
            make_header = input.read_u8()? != 0;
            short_description_id = read_opt_u32(&mut input)?;
            display_short_description = input.read_u8()? != 0;
            short_description_state = input.read_u8()?;
            start_time = input.read_u32::<LE>()?;
            mute_objective = input.read_u8()? != 0;
        } else {
            start_time = input.read_u32::<LE>()?;
        }

        let description = input.read_u32_length_prefixed_str()?;
        let name = input.read_u32_length_prefixed_str()?;
        let short_description = if version >= 1.8 {
            input.read_u32_length_prefixed_str()?
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
            short_description_id,
            short_description,
            display_short_description,
            short_description_state,
            mute_objective,
            name,
            effects,
            effect_order,
            conditions,
            condition_order,
            make_header,
        })
    }

    /// Write this trigger condition to an output stream, with the given trigger system version.
    pub fn write_to(&self, mut output: impl Write, version: f64) -> Result<()> {
        output.write_i32::<LE>(if self.enabled { 1 } else { 0 })?;
        output.write_i8(if self.looping { 1 } else { 0 })?;
        output.write_i32::<LE>(self.name_id)?;
        output.write_i8(if self.is_objective { 1 } else { 0 })?;
        output.write_i32::<LE>(self.objective_order)?;

        if version >= 1.8 {
            output.write_u8(if self.make_header { 1 } else { 0 })?;
            write_opt_string_key(&mut output, &self.short_description_id)?;
            output.write_u8(if self.display_short_description { 1 } else { 0 })?;
            output.write_u8(self.short_description_state)?;
            output.write_u32::<LE>(self.start_time)?;
            output.write_u8(if self.mute_objective { 1 } else { 0 })?;
        } else {
            output.write_u32::<LE>(self.start_time)?;
        }

        write_opt_i32_str(&mut output, &self.description)?;
        write_opt_i32_str(&mut output, &self.name)?;
        if version >= 1.8 {
            write_opt_i32_str(&mut output, &self.short_description)?;
        }

        output.write_u32::<LE>(self.effects.len() as u32)?;
        for effect in &self.effects {
            effect.write_to(&mut output, version)?;
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
    enabled_techs: Vec<u32>,
    variable_values: Vec<u32>,
    variable_names: Vec<String>,
}

impl Default for TriggerSystem {
    fn default() -> Self {
        Self {
            version: 1.6,
            objectives_state: 0,
            triggers: vec![],
            trigger_order: vec![],
            enabled_techs: vec![],
            variable_values: vec![0; 256],
            variable_names: vec![String::new(); 256],
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

        let mut variable_values = vec![];
        let mut enabled_techs = vec![];
        let mut variable_names = vec![];

        if version >= 2.2 {
            variable_values.resize(256, 0);
            input.read_u32_into::<LE>(&mut variable_values)?;
            enabled_techs = {
                let num_enabled_techs = input.read_u32::<LE>()?;
                let mut enabled_techs = vec![0; num_enabled_techs as usize];
                input.read_u32_into::<LE>(&mut enabled_techs)?;
                enabled_techs
            };
            variable_names = {
                let num_var_names = input.read_u32::<LE>()?;
                let mut variable_names = vec![String::new(); 256];
                for _ in 0..num_var_names {
                    let id = input.read_u32::<LE>()?;
                    assert!(
                        id < 256,
                        "Unexpected variable number, this is probably a genie-scx bug"
                    );
                    variable_names[id as usize] =
                        input.read_u32_length_prefixed_str()?.unwrap_or_default();
                }
                variable_names
            };
        }

        Ok(Self {
            version,
            objectives_state,
            triggers,
            trigger_order,
            enabled_techs,
            variable_values,
            variable_names,
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
        if version >= 2.2 {
            let padded_values = self
                .variable_values
                .iter()
                .cloned()
                .chain(std::iter::repeat(0))
                .take(256);
            for value in padded_values {
                output.write_u32::<LE>(value)?;
            }
            output.write_u32::<LE>(self.enabled_techs.len() as u32)?;
            for id in &self.enabled_techs {
                output.write_u32::<LE>(*id)?;
            }

            let custom_names = self
                .variable_names
                .iter()
                .enumerate()
                .filter(|(_index, name)| !name.is_empty());
            let num_custom_names = custom_names.clone().count();
            output.write_u32::<LE>(num_custom_names as u32)?;
            for (index, name) in custom_names {
                output.write_u32::<LE>(index as u32)?;
                write_i32_str(&mut output, &name)?;
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

fn write_opt_string_key(mut output: impl Write, opt_key: &Option<StringKey>) -> Result<()> {
    use std::io::{Error, ErrorKind};
    output.write_u32::<LE>(if let Some(key) = opt_key {
        key.try_into()
            .map_err(|err| Error::new(ErrorKind::InvalidData, err))?
    } else {
        0xFFFF_FFFF
    })?;
    Ok(())
}
