#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StartingResources {
    None = -1,
    Standard = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    UltraHigh = 4,
    Infinite = 5,
    Random = 6,
}

impl From<i32> for StartingResources {
    fn from(val: i32) -> Self {
        match val {
            -1 => StartingResources::None,
            0 => StartingResources::Standard,
            1 => StartingResources::Low,
            2 => StartingResources::Medium,
            3 => StartingResources::High,
            4 => StartingResources::UltraHigh,
            5 => StartingResources::Infinite,
            6 => StartingResources::Random,
            _ => unimplemented!("Don't know any starting resource with value {}", val),
        }
    }
}

impl Default for StartingResources {
    fn default() -> Self {
        StartingResources::Standard
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VictoryType {
    Standard = 0,
    Conquest = 1,
    Exploration = 2,
    Ruins = 3,
    Artifacts = 4,
    Discoveries = 5,
    Gold = 6,
    TimeLimit = 7,
    Score = 8,
    Standard2 = 9,
    Regicide = 10,
    LastManStanding = 11,
}

impl From<u32> for VictoryType {
    fn from(val: u32) -> Self {
        match val {
            0 => VictoryType::Standard,
            1 => VictoryType::Conquest,
            2 => VictoryType::Exploration,
            3 => VictoryType::Ruins,
            4 => VictoryType::Artifacts,
            5 => VictoryType::Discoveries,
            6 => VictoryType::Gold,
            7 => VictoryType::TimeLimit,
            8 => VictoryType::Score,
            9 => VictoryType::Standard2,
            10 => VictoryType::Regicide,
            11 => VictoryType::LastManStanding,
            _ => unimplemented!("Don't know any victory type with value {}", val),
        }
    }
}

impl Default for VictoryType {
    fn default() -> Self {
        VictoryType::Standard
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Difficulty {
    Easiest = 4,
    Easy = -1,
    /// Age of Empires 2: Definitive Edition only.
    Extreme = 5,
    Hard = 1,
    Hardest = 0,
    Moderate = 2,
    Standard = 3,
}

impl From<u32> for Difficulty {
    fn from(val: u32) -> Self {
        match val {
            0 => Difficulty::Hardest,
            1 => Difficulty::Hard,
            2 => Difficulty::Moderate,
            3 => Difficulty::Standard,
            4 => Difficulty::Easiest,
            5 => Difficulty::Extreme,
            _ => unimplemented!("Don't know any difficulty with value {}", val),
        }
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        Difficulty::Standard
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MapSize {
    Tiny,
    Small,
    Medium,
    Normal,
    Large,
    Giant,
    Ludicrous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MapType {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceLevel {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Age {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility {
    Normal,
    Explored,
    AllVisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameMode {
    RM = 0,
    Regicide = 1,
    DM = 2,
    Scenario = 3,
    Campaign = 4,
    KingOfTheHill = 5,
    WonderRace = 6,
    DefendTheWonder = 7,
    TurboRandom = 8,
    CaptureTheRelic = 10,
    SuddenDeath = 11,
    BattleRoyale = 12,
    EmpireWars = 13,
}

impl From<u32> for GameMode {
    fn from(n: u32) -> Self {
        match n {
            0 => GameMode::RM,
            1 => GameMode::Regicide,
            2 => GameMode::DM,
            3 => GameMode::Scenario,
            4 => GameMode::Campaign,
            5 => GameMode::KingOfTheHill,
            6 => GameMode::WonderRace,
            7 => GameMode::DefendTheWonder,
            8 => GameMode::TurboRandom,
            10 => GameMode::CaptureTheRelic,
            11 => GameMode::SuddenDeath,
            12 => GameMode::BattleRoyale,
            13 => GameMode::EmpireWars,
            _ => unimplemented!("Don't know any game mode with value {}", n),
        }
    }
}

impl Default for GameMode {
    fn default() -> Self {
        GameMode::RM
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameSpeed {
    Slow,
    Casual,
    Normal,
    Fast,
}
