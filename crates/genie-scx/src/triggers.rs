use crate::Result;
use crate::UnitTypeID;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{read_str, write_str};
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
    pub fn from<R: Read>(input: &mut R, version: f64) -> Result<Self> {
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
    pub fn write_to<W: Write>(&self, output: &mut W, version: f64) -> Result<()> {
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

    pub fn amount(&self) -> i32 {
        self.properties[0]
    }

    pub fn set_amount(&mut self, amount: i32) {
        self.properties[0] = amount;
    }

    pub fn resource(&self) -> i32 {
        self.properties[1]
    }

    pub fn set_resource(&mut self, resource: i32) {
        self.properties[1] = resource;
    }

    pub fn primary_object(&self) -> i32 {
        self.properties[2]
    }

    pub fn set_primary_object(&mut self, primary_object: i32) {
        self.properties[2] = primary_object;
    }

    pub fn secondary_object(&self) -> i32 {
        self.properties[3]
    }

    pub fn set_secondary_object(&mut self, secondary_object: i32) {
        self.properties[3] = secondary_object;
    }

    pub fn raw_unit_type(&self) -> i32 {
        self.properties[4]
    }

    pub fn set_raw_unit_type(&mut self, unit_type: i32) {
        self.properties[4] = unit_type;
    }

    pub fn unit_type(&self) -> Option<UnitTypeID> {
        match self.properties[4] {
            -1 => None,
            unit_type => Some(unit_type.try_into().unwrap()),
        }
    }

    pub fn set_unit_type(&mut self, unit_type: Option<UnitTypeID>) {
        self.properties[4] = match unit_type {
            Some(unit_type) => i32::from(unit_type),
            None => -1,
        };
    }

    pub fn player_id(&self) -> i32 {
        self.properties[5]
    }

    pub fn set_player_id(&mut self, player_id: i32) {
        self.properties[5] = player_id;
    }

    pub fn technology_id(&self) -> i32 {
        self.properties[6]
    }

    pub fn set_technology_id(&mut self, technology_id: i32) {
        self.properties[6] = technology_id;
    }

    pub fn timer(&self) -> i32 {
        self.properties[7]
    }

    pub fn set_timer(&mut self, timer: i32) {
        self.properties[7] = timer;
    }

    pub fn trigger_id(&self) -> i32 {
        self.properties[8]
    }

    pub fn set_trigger_id(&mut self, trigger_id: i32) {
        self.properties[8] = trigger_id;
    }

    pub fn area(&self) -> (i32, i32, i32, i32) {
        (
            self.properties[9],
            self.properties[10],
            self.properties[11],
            self.properties[12],
        )
    }

    pub fn set_area(&mut self, area: (i32, i32, i32, i32)) {
        self.properties[9] = area.0;
        self.properties[10] = area.1;
        self.properties[11] = area.2;
        self.properties[12] = area.3;
    }

    pub fn unit_group(&self) -> i32 {
        self.properties[13]
    }

    pub fn set_unit_group(&mut self, unit_group: i32) {
        self.properties[13] = unit_group;
    }

    pub fn object_type(&self) -> Option<UnitTypeID> {
        match self.properties[14] {
            -1 => None,
            object_type => Some(object_type.try_into().unwrap()),
        }
    }

    pub fn set_object_type(&mut self, object_type: Option<UnitTypeID>) {
        self.properties[14] = match object_type {
            Some(object_type) => i32::from(object_type),
            None => -1,
        };
    }

    pub fn ai_signal(&self) -> i32 {
        self.properties[15]
    }

    pub fn set_ai_signal(&mut self, ai_signal: i32) {
        self.properties[15] = ai_signal;
    }

    pub fn inverted(&self) -> bool {
        self.properties[16] == 1
    }

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
    pub fn from<R: Read>(input: &mut R, version: f64) -> Result<Self> {
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
        let chat_text = read_str(input, len)?;
        let len = input.read_i32::<LE>()? as usize;
        let audio_file = read_str(input, len)?;
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
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.effect_type)?;
        output.write_i32::<LE>(self.properties.len() as i32)?;
        for value in &self.properties {
            output.write_i32::<LE>(*value)?;
        }

        Ok(())
    }

    pub fn ai_goal(&self) -> i32 {
        self.properties[0]
    }

    pub fn set_ai_goal(&mut self, ai_goal: i32) {
        self.properties[0] = ai_goal;
    }

    pub fn amount(&self) -> i32 {
        self.properties[1]
    }

    pub fn set_amount(&mut self, amount: i32) {
        self.properties[1] = amount;
    }

    pub fn resource(&self) -> i32 {
        self.properties[2]
    }

    pub fn set_resource(&mut self, resource: i32) {
        self.properties[2] = resource;
    }

    pub fn diplomacy(&self) -> i32 {
        self.properties[3]
    }

    pub fn set_diplomacy(&mut self, diplomacy: i32) {
        self.properties[3] = diplomacy;
    }

    pub fn num_objects(&self) -> i32 {
        self.properties[4]
    }

    pub fn set_num_objects(&mut self, num_objects: i32) {
        self.properties[4] = num_objects;
    }

    pub fn object_id(&self) -> i32 {
        self.properties[5]
    }

    pub fn set_object_id(&mut self, object_id: i32) {
        self.properties[5] = object_id;
    }

    pub fn unit_type(&self) -> Option<UnitTypeID> {
        match self.properties[6] {
            -1 => None,
            unit_type => Some(unit_type.try_into().unwrap()),
        }
    }

    pub fn set_unit_type(&mut self, unit_type: Option<UnitTypeID>) {
        self.properties[6] = match unit_type {
            Some(unit_type) => i32::from(unit_type),
            None => -1,
        };
    }

    pub fn source_player_id(&self) -> i32 {
        self.properties[7]
    }

    pub fn set_source_player_id(&mut self, source_player_id: i32) {
        self.properties[7] = source_player_id;
    }

    pub fn target_player_id(&self) -> i32 {
        self.properties[8]
    }

    pub fn set_target_player_id(&mut self, target_player_id: i32) {
        self.properties[8] = target_player_id;
    }

    pub fn technology_id(&self) -> i32 {
        self.properties[9]
    }

    pub fn set_technology_id(&mut self, technology_id: i32) {
        self.properties[9] = technology_id;
    }

    pub fn text_id(&self) -> i32 {
        self.properties[10]
    }

    pub fn set_text_id(&mut self, text_id: i32) {
        self.properties[10] = text_id;
    }

    pub fn sound_id(&self) -> i32 {
        self.properties[11]
    }

    pub fn set_sound_id(&mut self, sound_id: i32) {
        self.properties[11] = sound_id;
    }

    pub fn timer(&self) -> i32 {
        self.properties[12]
    }

    pub fn set_timer(&mut self, timer: i32) {
        self.properties[12] = timer;
    }

    pub fn trigger_id(&self) -> i32 {
        self.properties[13]
    }

    pub fn set_trigger_id(&mut self, trigger_id: i32) {
        self.properties[13] = trigger_id;
    }

    pub fn location(&self) -> (i32, i32) {
        (self.properties[14], self.properties[15])
    }

    pub fn set_location(&mut self, location: (i32, i32)) {
        self.properties[14] = location.0;
        self.properties[15] = location.1;
    }

    pub fn area(&self) -> (i32, i32, i32, i32) {
        (
            self.properties[16],
            self.properties[17],
            self.properties[18],
            self.properties[19],
        )
    }

    pub fn set_area(&mut self, area: (i32, i32, i32, i32)) {
        self.properties[16] = area.0;
        self.properties[17] = area.1;
        self.properties[18] = area.2;
        self.properties[19] = area.3;
    }

    pub fn object_group(&self) -> i32 {
        self.properties[20]
    }

    pub fn set_object_group(&mut self, object_group: i32) {
        self.properties[20] = object_group;
    }

    pub fn object_type(&self) -> Option<UnitTypeID> {
        match self.properties[21] {
            -1 => None,
            object_type => Some(object_type.try_into().unwrap()),
        }
    }

    pub fn set_object_type(&mut self, object_type: Option<UnitTypeID>) {
        self.properties[21] = match object_type {
            Some(object_type) => i32::from(object_type),
            None => -1,
        };
    }

    pub fn line_id(&self) -> i32 {
        self.properties[22]
    }

    pub fn set_line_id(&mut self, line_id: i32) {
        self.properties[22] = line_id;
    }

    pub fn stance(&self) -> i32 {
        self.properties[23]
    }

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
    name: Option<String>,
    effects: Vec<TriggerEffect>,
    effect_order: Vec<i32>,
    conditions: Vec<TriggerCondition>,
    condition_order: Vec<i32>,
}

impl Trigger {
    /// Read a trigger from an input stream, with the given trigger system version.
    pub fn from<R: Read>(input: &mut R, version: f64) -> Result<Self> {
        let enabled = input.read_i32::<LE>()? != 0;
        let looping = input.read_i8()? != 0;
        let name_id = input.read_i32::<LE>()?;
        let is_objective = input.read_i8()? != 0;
        let objective_order = input.read_i32::<LE>()?;
        let start_time = input.read_u32::<LE>()?;

        let description_length = input.read_u32::<LE>()? as usize;
        let description = read_str(input, description_length)?;

        let name_length = input.read_u32::<LE>()? as usize;
        let name = read_str(input, name_length)?;

        let num_effects = input.read_i32::<LE>()?;
        let mut effects = vec![];
        let mut effect_order = vec![];
        for _ in 0..num_effects {
            effects.push(TriggerEffect::from(input, version)?);
        }
        for _ in 0..num_effects {
            effect_order.push(input.read_i32::<LE>()?);
        }

        let num_conditions = input.read_i32::<LE>()?;
        let mut conditions = vec![];
        let mut condition_order = vec![];
        for _ in 0..num_conditions {
            conditions.push(TriggerCondition::from(input, version)?);
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
            name,
            effects,
            effect_order,
            conditions,
            condition_order,
        })
    }

    /// Write this trigger condition to an output stream, with the given trigger system version.
    pub fn write_to<W: Write>(&self, output: &mut W, version: f64) -> Result<()> {
        output.write_i32::<LE>(if self.enabled { 1 } else { 0 })?;
        output.write_i8(if self.looping { 1 } else { 0 })?;
        output.write_i32::<LE>(self.name_id)?;
        output.write_i8(if self.is_objective { 1 } else { 0 })?;
        output.write_i32::<LE>(self.objective_order)?;
        output.write_u32::<LE>(self.start_time)?;

        if let Some(descr) = &self.description {
            output.write_u32::<LE>(descr.len().try_into().unwrap())?;
            write_str(output, descr)?;
        } else {
            output.write_u32::<LE>(0)?;
        }
        if let Some(name) = &self.name {
            output.write_u32::<LE>(name.len().try_into().unwrap())?;
            write_str(output, name)?;
        } else {
            output.write_u32::<LE>(0)?;
        }

        output.write_u32::<LE>(self.effects.len() as u32)?;
        for effect in &self.effects {
            effect.write_to(output)?;
        }
        for order in &self.effect_order {
            output.write_i32::<LE>(*order)?;
        }
        output.write_u32::<LE>(self.conditions.len() as u32)?;
        for condition in &self.conditions {
            condition.write_to(output, version)?;
        }
        for order in &self.condition_order {
            output.write_i32::<LE>(*order)?;
        }

        Ok(())
    }

    pub fn conditions(&self) -> impl Iterator<Item = &TriggerCondition> {
        self.condition_order
            .iter()
            .map(move |index| &self.conditions[*index as usize])
    }

    pub fn conditions_unordered_mut(&mut self) -> impl Iterator<Item = &mut TriggerCondition> {
        self.conditions.iter_mut()
    }

    pub fn effects(&self) -> impl Iterator<Item = &TriggerEffect> {
        self.effect_order
            .iter()
            .map(move |index| &self.effects[*index as usize])
    }

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
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let version = input.read_f64::<LE>()?;
        let objectives_state = if version >= 1.5 { input.read_i8()? } else { 0 };

        let num_triggers = input.read_i32::<LE>()?;

        let mut triggers = vec![];
        let mut trigger_order = vec![];
        for _ in 0..num_triggers {
            triggers.push(Trigger::from(input, version)?);
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

    pub fn write_to<W: Write>(&self, output: &mut W, version: f64) -> Result<()> {
        output.write_f64::<LE>(version)?;
        if version >= 1.5 {
            output.write_i8(self.objectives_state)?;
        }
        output.write_u32::<LE>(self.triggers.len().try_into().unwrap())?;
        for trigger in &self.triggers {
            trigger.write_to(output, version)?;
        }
        if version >= 1.4 {
            for order in &self.trigger_order {
                output.write_i32::<LE>(*order)?;
            }
        }
        Ok(())
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
