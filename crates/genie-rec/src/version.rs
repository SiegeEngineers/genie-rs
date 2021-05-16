use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Display};
use std::io::Read;

/// the variant of AoE2 game
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GameVariant {
    /// A trial version, either AoC or AoE
    Trial,
    /// Age of Kings
    AgeOfKings,
    /// Age of Conquerors
    AgeOfConquerors,
    /// User Patch
    UserPatch,
    /// Forgotten Empires mod
    ForgottenEmpires,
    /// AoE2:HD release
    HighDefinition,
    /// AoE2:DE release
    DefinitiveEdition,
}

pub const TRIAL_VERSION: GameVersion = GameVersion(*b"TRL 9.3\0");
pub const AGE_OF_KINGS_VERSION: GameVersion = GameVersion(*b"VER 9.3\0");
pub const AGE_OF_CONQUERORS_VERSION: GameVersion = GameVersion(*b"VER 9.4\0");
pub const FORGOTTEN_EMPIRES_VERSION: GameVersion = GameVersion(*b"VER 9.5\0");

// So last known AoC version is 11.76
// Since all versions only have a precision of 2
// We can save do + 0.01
pub const HD_SAVE_VERSION: f32 = 11.77;

pub const DE_SAVE_VERSION: f32 = 12.97;

use GameVariant::*;

impl GameVariant {
    pub fn resolve_variant(version: &GameVersion, sub_version: f32) -> Option<GameVariant> {
        // taken from https://github.com/goto-bus-stop/recanalyst/blob/master/src/Analyzers/VersionAnalyzer.php

        Some(match *version {
            // Either AOC or AOK trial, just return Trial :shrug:
            TRIAL_VERSION => Trial,
            AGE_OF_KINGS_VERSION => AgeOfKings,
            AGE_OF_CONQUERORS_VERSION if sub_version >= DE_SAVE_VERSION => DefinitiveEdition,
            AGE_OF_CONQUERORS_VERSION if sub_version >= HD_SAVE_VERSION => HighDefinition,
            AGE_OF_CONQUERORS_VERSION => AgeOfConquerors,
            FORGOTTEN_EMPIRES_VERSION => ForgottenEmpires,
            // UserPatch uses VER 9.<N>\0 where N is anything between 8 and F
            GameVersion([b'V', b'E', b'R', b' ', b'9', b'.', b'8'..=b'F', b'\0']) => UserPatch,
            _ => return None,
        })
    }

    pub fn is_original(&self) -> bool {
        matches!(self, Trial | AgeOfKings | AgeOfConquerors)
    }

    pub fn is_mod(&self) -> bool {
        matches!(self, ForgottenEmpires | UserPatch)
    }

    pub fn is_update(&self) -> bool {
        matches!(self, HighDefinition | DefinitiveEdition)
    }
}

/// A bit of a weird comparing check
/// It follows the hierarchy of what game is based on what
///
/// Thus AgeOfConquerors is bigger than AgeOfKings and HighDefinition is bigger than AgeOfConquers etc.
///
/// The confusing part is around HighDefinition and UserPatch
/// UserPatch is neither bigger, smaller or equal to the -new- editions created by MSFT and vice versa
impl PartialOrd for GameVariant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // quick return for equal stuff :)
        if other == self {
            return Some(Ordering::Equal);
        }

        // Try to not use Ord operators here, to make sure we don't fall in weird recursive traps
        let is_mod = self.is_mod() || other.is_mod();
        let update = self.is_update() || other.is_update();
        let original = self.is_original() || other.is_original();

        // Can't compare between user patch and hd and up
        if is_mod && update {
            return None;
        }

        if original && (is_mod || update) {
            return Some(if self.is_original() {
                Ordering::Less
            } else {
                Ordering::Greater
            });
        }

        // So this part is a bit confusing
        // but basically we removed all comparisons that are between e.g. mod, update and original
        // and we removed all comparisons that are equal
        // so the only comparison left is within their own class
        Some(match self {
            // Trial is only compared to AoK and AoC, and is the first version, thus always less
            Trial => Ordering::Less,
            // AoK can only be greater if compared against trial
            AgeOfKings if other == &Trial => Ordering::Greater,
            AgeOfKings => Ordering::Less,
            // AoC will always be greater
            AgeOfConquerors => Ordering::Greater,
            // Can we compare UP and FE???
            UserPatch | ForgottenEmpires => return None,
            // HD can only be compared to DE, and vice versa
            HighDefinition => Ordering::Less,
            DefinitiveEdition => Ordering::Greater,
        })
    }
}

/// The game data version string. In practice, this does not really reflect the game version.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GameVersion([u8; 8]);

impl From<&[u8; 8]> for GameVersion {
    fn from(val: &[u8; 8]) -> Self {
        GameVersion(*val)
    }
}

/// I am very lazy :)
impl From<&[u8; 7]> for GameVersion {
    fn from(val: &[u8; 7]) -> Self {
        let mut whole = [0; 8];
        whole[..7].copy_from_slice(val);
        GameVersion(whole)
    }
}

impl Default for GameVersion {
    fn default() -> Self {
        Self([0; 8])
    }
}

impl Debug for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", std::str::from_utf8(&self.0).unwrap())
    }
}

impl Display for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.0).unwrap())
    }
}

impl GameVersion {
    /// Read the game version string from an input stream.
    pub fn read_from<R: Read>(input: &mut R) -> crate::Result<Self> {
        let mut game_version = [0; 8];
        input.read_exact(&mut game_version)?;
        Ok(Self(game_version))
    }
}

#[cfg(test)]
mod test {
    use crate::GameVariant::*;
    use crate::*;

    #[test]
    pub fn test_game_variant_resolution() {
        assert_eq!(
            Some(Trial),
            GameVariant::resolve_variant(&TRIAL_VERSION, 0.0)
        );

        assert_eq!(
            Some(AgeOfKings),
            GameVariant::resolve_variant(&AGE_OF_KINGS_VERSION, 0.0)
        );

        assert_eq!(
            Some(AgeOfConquerors),
            GameVariant::resolve_variant(&AGE_OF_CONQUERORS_VERSION, 0.0)
        );

        assert_eq!(
            Some(HighDefinition),
            GameVariant::resolve_variant(&AGE_OF_CONQUERORS_VERSION, HD_SAVE_VERSION)
        );

        assert_eq!(
            Some(DefinitiveEdition),
            GameVariant::resolve_variant(&AGE_OF_CONQUERORS_VERSION, DE_SAVE_VERSION)
        );

        assert_eq!(
            Some(UserPatch),
            GameVariant::resolve_variant(&b"VER 9.A".into(), 0.0),
        );

        assert_eq!(
            Some(ForgottenEmpires),
            GameVariant::resolve_variant(&b"VER 9.5".into(), 0.0),
        );
    }

    #[test]
    pub fn test_game_variant_comparison() {
        // Am I going add all cases here? WHO KNOWS
        assert!(Trial < AgeOfKings);
        assert!(AgeOfKings > Trial);
        assert!(Trial < AgeOfConquerors);
        assert!(AgeOfConquerors > Trial);
        assert!(Trial < ForgottenEmpires);
        assert!(ForgottenEmpires > Trial);
        assert!(Trial < UserPatch);
        assert!(UserPatch > Trial);
        assert!(Trial < HighDefinition);
        assert!(HighDefinition > Trial);
        assert!(Trial < DefinitiveEdition);
        assert!(DefinitiveEdition > Trial);

        assert!(AgeOfKings < AgeOfConquerors);
        assert!(AgeOfConquerors > AgeOfKings);
        assert!(AgeOfKings < ForgottenEmpires);
        assert!(ForgottenEmpires > AgeOfKings);
        assert!(AgeOfKings < UserPatch);
        assert!(UserPatch > AgeOfKings);
        assert!(AgeOfKings < HighDefinition);
        assert!(HighDefinition > AgeOfKings);
        assert!(AgeOfKings < DefinitiveEdition);
        assert!(DefinitiveEdition > AgeOfKings);

        assert!(AgeOfConquerors < ForgottenEmpires);
        assert!(ForgottenEmpires > AgeOfConquerors);
        assert!(AgeOfConquerors < UserPatch);
        assert!(UserPatch > AgeOfConquerors);
        assert!(AgeOfConquerors < HighDefinition);
        assert!(HighDefinition > AgeOfConquerors);
        assert!(AgeOfConquerors < DefinitiveEdition);
        assert!(DefinitiveEdition > AgeOfConquerors);
        assert!(DefinitiveEdition >= AgeOfConquerors);

        assert_eq!(false, ForgottenEmpires < UserPatch);
        assert_eq!(false, UserPatch > ForgottenEmpires);
        assert_eq!(false, ForgottenEmpires < HighDefinition);
        assert_eq!(false, HighDefinition > ForgottenEmpires);
        assert_eq!(false, ForgottenEmpires < DefinitiveEdition);
        assert_eq!(false, DefinitiveEdition > ForgottenEmpires);

        assert_eq!(false, UserPatch < HighDefinition);
        assert_eq!(false, HighDefinition > UserPatch);
        assert_eq!(false, UserPatch < DefinitiveEdition);
        assert_eq!(false, DefinitiveEdition > UserPatch);

        assert!(HighDefinition < DefinitiveEdition);
        assert!(DefinitiveEdition > HighDefinition);
        // yes i was
    }
}
