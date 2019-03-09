use std::io::{Read, Write, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use crate::util::*;

#[derive(Debug)]
pub struct TriggerCondition {
    condition_type: i32,
    properties: Vec<i32>,
}

impl TriggerCondition {
    pub fn from<R: Read>(input: &mut R, version: f64) -> Result<Self> {
        let condition_type = input.read_i32::<LE>()?;
        let num_properties = if version > 1.0 {
            input.read_i32::<LE>()?
        } else { 13 };
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

    pub fn unit_type(&self) -> i32 {
        self.properties[4]
    }

    pub fn set_unit_type(&mut self, unit_type: i32) {
        self.properties[4] = unit_type;
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

    pub fn object_type(&self) -> i32 {
        self.properties[14]
    }

    pub fn set_object_type(&mut self, object_type: i32) {
        self.properties[14] = object_type;
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

#[derive(Debug)]
pub struct TriggerEffect {
    effect_type: i32,
    properties: Vec<i32>,
    chat_text: Option<String>,
    audio_file: Option<String>,
    objects: Vec<i32>,
}

impl TriggerEffect {
    pub fn from<R: Read>(input: &mut R, version: f64) -> Result<Self> {
        let effect_type = input.read_i32::<LE>()?;
        let num_properties = if version > 1.0 {
            input.read_i32::<LE>()?
        } else { 16 };
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

    pub fn unit_type(&self) -> i32 {
        self.properties[6]
    }

    pub fn set_unit_type(&mut self, unit_type: i32) {
        self.properties[6] = unit_type;
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
        (
            self.properties[14],
            self.properties[15],
        )
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

    pub fn object_type(&self) -> i32 {
        self.properties[21]
    }

    pub fn set_object_type(&mut self, object_type: i32) {
        self.properties[21] = object_type;
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

#[derive(Debug)]
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

    pub fn conditions(&self) -> impl Iterator<Item = &TriggerCondition> {
        self.condition_order.iter()
            .map(move |index| &self.conditions[*index as usize])
    }

    pub fn effects(&self) -> impl Iterator<Item = &TriggerEffect> {
        self.effect_order.iter()
            .map(move |index| &self.effects[*index as usize])
    }
}

#[derive(Debug)]
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
        let objectives_state = if version >= 1.5 {
            input.read_i8()?
        } else {
            0
        };

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
        // num triggers
        output.write_u32::<LE>(0)?;
        Ok(())
    }
}
