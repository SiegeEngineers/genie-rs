use crate::string_table::StringTable;
use crate::Result;
use byteorder::{ReadBytesExt, LE};
use std::fmt::{self, Debug};
use std::io::{Read};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GameVersion([u8; 8]);

impl Default for GameVersion {
    fn default() -> Self {
        Self([0; 8])
    }
}

impl Debug for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", std::str::from_utf8(&self.0).unwrap())
    }
}

impl GameVersion {
    pub fn from(mut input: impl Read) -> Result<Self> {
        let mut game_version = [0; 8];
        input.read_exact(&mut game_version)?;
        Ok(Self(game_version))
    }
}

#[derive(Debug, Default, Clone)]
pub struct AICommand {
    command_type: i32,
    id: u16,
    parameters: [i32; 4],
}

impl AICommand {
    pub fn from(mut input: impl Read) -> Result<Self> {
        let mut cmd = Self::default();
        cmd.command_type = input.read_i32::<LE>()?;
        cmd.id = input.read_u16::<LE>()?;
        input.read_u16::<LE>()?;
        for param in cmd.parameters.iter_mut() {
            *param = input.read_i32::<LE>()?;
        }
        Ok(cmd)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIListRule {
    in_use: bool,
    enable: bool,
    rule_id: u16,
    next_in_group: u16,
    facts: Vec<AICommand>,
    actions: Vec<AICommand>,
}

impl AIListRule {
    pub fn from(mut input: impl Read) -> Result<Self> {
        let mut rule = Self::default();
        rule.in_use = input.read_u32::<LE>()? != 0;
        rule.enable = input.read_u32::<LE>()? != 0;
        rule.rule_id = input.read_u16::<LE>()?;
        rule.next_in_group = input.read_u16::<LE>()?;
        let num_facts = input.read_u8()?;
        let num_facts_actions = input.read_u8()?;
        input.read_u16::<LE>()?;
        for i in 0..16 {
            let cmd = AICommand::from(&mut input)?;
            if i < num_facts {
                rule.facts.push(cmd);
            } else if i < num_facts_actions {
                rule.actions.push(cmd);
            }
        }
        Ok(rule)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIList {
    in_use: bool,
    id: i32,
    max_rules: u16,
    rules: Vec<AIListRule>,
}

impl AIList {
    pub fn from(mut input: impl Read) -> Result<Self> {
        let mut list = Self::default();
        list.in_use = input.read_u32::<LE>()? != 0;
        list.id = input.read_i32::<LE>()?;
        list.max_rules = input.read_u16::<LE>()?;
        let num_rules = input.read_u16::<LE>()?;
        input.read_u32::<LE>()?;
        for _ in 0..num_rules {
            list.rules.push(AIListRule::from(&mut input)?);
        }
        Ok(list)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIGroupTable {
    max_groups: u16,
    groups: Vec<u16>,
}

impl AIGroupTable {
    pub fn from(mut input: impl Read) -> Result<Self> {
        let mut table = Self::default();
        table.max_groups = input.read_u16::<LE>()?;
        let num_groups = input.read_u16::<LE>()?;
        input.read_u32::<LE>()?;
        for _ in 0..num_groups {
            table.groups.push(input.read_u16::<LE>()?);
        }
        Ok(table)
    }
}

#[derive(Debug, Clone)]
pub struct AIScripts {
    string_table: StringTable,
    lists: Vec<AIList>,
    groups: Vec<AIGroupTable>,
}

impl AIScripts {
    pub fn from(mut input: impl Read) -> Result<Self> {
        let string_table = StringTable::from(&mut input)?;
        let max_facts = input.read_u16::<LE>()?;
        let max_actions = input.read_u16::<LE>()?;
        let max_lists = input.read_u16::<LE>()?;

        let mut lists = vec![];
        for _ in 0..max_lists {
            lists.push(AIList::from(&mut input)?);
        }

        let mut groups = vec![];
        for _ in 0..max_lists {
            groups.push(AIGroupTable::from(&mut input)?);
        }

        let _language_save_version = input.read_f32::<LE>()?;
        let _language_version = input.read_f32::<LE>()?;

        Ok(AIScripts {
            string_table,
            lists,
            groups,
        })
    }
}

#[derive(Debug, Default)]
pub struct Header {
    game_version: GameVersion,
    save_version: f32,
    ai_scripts: Option<AIScripts>,
}

impl Header {
    pub fn from(mut input: impl Read) -> Result<Self> {
        let mut header = Header::default();
        header.game_version = GameVersion::from(&mut input)?;
        header.save_version = input.read_f32::<LE>()?;

        let includes_ai = input.read_u32::<LE>()? != 0;
        if includes_ai {
            header.ai_scripts = Some(AIScripts::from(&mut input)?);
        }

        Ok(header)
    }
}
