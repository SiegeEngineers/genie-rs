//! genie-lang reads language files into a map of UTF-8 strings.
//! All three major language file types used by Age of Empires versions are
//! supported: DLLs, INI files, and HD Edition's key-value format.
//!
//! DLLs are used by the original games.
//! INI files are used for Voobly mods, and can be used with a standard
//! AoC installation through the aoc-language-ini mod.
//!
//! ## DLLs
//! ```rust
//! use genie_lang::LangFile;
//! use std::fs::File;
//! let f = File::open("./test/dlls/language_x1_p1.dll").unwrap();
//! let lang_file = LangFile::from_dll(f).unwrap();
//! assert_eq!(
//!     lang_file.get(30177),
//!     Some("Turbo Random Map - Buildings create units faster, villagers gather faster, build faster, and carry more."));
//! assert_eq!(
//!     lang_file.get(20156),
//!     Some("<b>Byzantines<b> \n\
//!           Defensive civilization \n\
//!           · Buildings +10% HPs Dark, +20% Feudal, \n +30% Castle, +40% Imperial Age \n\
//!           · Camels, skirmishers, Pikemen, Halberdiers cost -25% \n\
//!           · Fire ships +20% attack \n\
//!           · Advance to Imperial Age costs -33% \n\
//!           · Town Watch free \n\n\
//!           <b>Unique Unit:<b> Cataphract (cavalry) \n\n\
//!           <b>Unique Tech:<b> Logistica (Cataphracts cause trample damage) \n\n\
//!           <b>Team Bonus:<b> Monks +50% heal speed"));
//! ```
//!
//! ## INI files
//! ```rust
//! use genie_lang::LangFile;
//! use std::io::Cursor;
//! let text = br#"
//! 46523=The Uighurs will join if you kill Ornlu the wolf and return to tell the tale.
//! ; a comment
//! 46524=Uighurs: Yes, that is the pelt of the great wolf. We will join you, Genghis Khan. And to seal the agreement, we will give you the gift of flaming arrows!
//! "#;
//! let f = Cursor::new(&text[..]);
//! let lang_file = LangFile::from_ini(f).unwrap();
//! assert_eq!(
//!     lang_file.get(46523),
//!     Some("The Uighurs will join if you kill Ornlu the wolf and return to tell the tale."));
//! ```
//!
//! ## HD key-value files
//! ```rust
//! use genie_lang::LangFile;
//! use std::io::Cursor;
//! let text = br#"
//! 46523 "The Uighurs will join if you kill Ornlu the wolf and return to tell the tale."
//! 46524 "Uighurs: Yes, that is the pelt of the great wolf. We will join you, Genghis Khan. And to seal the agreement, we will give you the gift of flaming arrows!"
//! 46604 "Kill the traitor, Kushluk.\n\nPrevent the tent of Genghis Khan (Wonder) from being destroyed."
//! LOBBYBROWSER_DATMOD_TITLE_FORMAT "DatMod: \"%s\""
//! "#;
//! let f = Cursor::new(&text[..]);
//! let lang_file = LangFile::from_keyval(f).unwrap();
//! assert_eq!(
//!     lang_file.get(46523),
//!     Some("The Uighurs will join if you kill Ornlu the wolf and return to tell the tale."));
//! assert_eq!(
//!     lang_file.get_named("LOBBYBROWSER_DATMOD_TITLE_FORMAT"),
//!     Some(r#"DatMod: "%s""#));
//! ```
//!
//! ## Creating a file from scratch
//! ```rust
//! use genie_lang::LangFile;
//! use std::str;
//! let mut lang_file = LangFile::default();
//! lang_file.set(46604, "Kill the traitor, Kushluk.\n\n\
//!                       Prevent the tent of Genghis Khan (Wonder) from being destroyed.");
//! lang_file.set_named("LOBBYBROWSER_DATMOD_TITLE_FORMAT", r#"DatMod: "%s""#);
//! let mut out = vec![];
//! lang_file.write_to_ini(&mut out).unwrap();
//! assert_eq!(
//!     str::from_utf8(&out).unwrap(),
//!     r"46604=Kill the traitor, Kushluk.\n\nPrevent the tent of Genghis Khan (Wonder) from being destroyed.
//! ");
//! let mut out = vec![];
//! lang_file.write_to_keyval(&mut out).unwrap();
//! assert_eq!(
//!     str::from_utf8(&out).unwrap(),
//!     r#"46604 "Kill the traitor, Kushluk.\n\nPrevent the tent of Genghis Khan (Wonder) from being destroyed."
//!
//! LOBBYBROWSER_DATMOD_TITLE_FORMAT "DatMod: \"%s\""
//! "#);
//! ```

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::{Read, Write, BufRead, BufReader, Error as IoError};
use std::num::ParseIntError;
use std::str::FromStr;
use byteorder::{ReadBytesExt, LE};
use encoding_rs::{WINDOWS_1252, UTF_16LE};
use encoding_rs_io::DecodeReaderBytesBuilder;
use pelite::{
    pe32::{Pe, PeFile},
    resources::Name,
};

/// A key in a language file.
///
/// A key may be either a nonnegative integer or an arbitrary string.
///
/// The original game supports only nonnegative integers.
/// The HD Edition allows for integers as well as Strings to serve as keys in a
/// key value file.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum StringKey {

    /// An integer string key.
    Num(u32),

    /// A named string key.
    /// The string must not represent a `u32` value (such keys must be `Num`).
    Name(String),
}

impl fmt::Display for StringKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use StringKey::{Name, Num};
        match self {
            Num(n)  => write!(f, "{}", n),
            Name(s) => write!(f, "{}", s),
        }
    }
}

impl From<u32> for StringKey {
    fn from(n: u32) -> Self { StringKey::Num(n) }
}

impl From<&str> for StringKey {
    fn from(s: &str) -> Self {
        use StringKey::{Name, Num};
        if let Ok(n) = s.parse() { Num(n) }
        else { Name(String::from(s)) }
    }
}

impl From<String> for StringKey {
    fn from(s: String) -> Self { StringKey::from(&s[..]) }
}

/// Errors that may occur when loading a language file.
///
/// For DLL files, PeError and IoError can occur.
/// For INI and HD Edition files, ParseIntError and IoError can occur.
/// Both the INI and HD Edition parsers silently ignore invalid lines.
#[derive(Debug)]
pub enum LoadError {
    /// An error occurred while reading strings from the DLL.
    /// It probably does not contain any or is malformed.
    PeError(pelite::Error),
    /// An error occurred while reading data from the file.
    IoError(IoError),
    /// An error occurred while parsing a numeric string ID into an integer
    /// value.
    ParseIntError(ParseIntError),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LoadError::{IoError, ParseIntError, PeError};
        match self {
            IoError(e)       => e.fmt(f),
            ParseIntError(e) => e.fmt(f),
            PeError(e)       => e.fmt(f),
        }
    }
}

impl From<pelite::Error> for LoadError {
    fn from(error: pelite::Error) -> Self {
        LoadError::PeError(error)
    }
}

impl From<IoError> for LoadError {
    fn from(error: IoError) -> Self {
        LoadError::IoError(error)
    }
}

impl From<ParseIntError> for LoadError {
    fn from(error: ParseIntError) -> Self {
        LoadError::ParseIntError(error)
    }
}

impl Error for LoadError { }

/// An error when parsing a string to a language file.
///
/// The field contains the string that could not be parsed.
#[derive(Debug)]
pub struct ParseLangFileTypeError(String);

impl fmt::Display for ParseLangFileTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseLangFileTypeError { }

/// Aoe2 supports three types of language files
#[derive(Debug)]
pub enum LangFileType {
    Dll, Ini, KeyValue,
}

impl LangFileType {
    // TODO specify
    // TODO `r` might not need to be `mut`?
    pub fn read_from(&self, r: impl Read)
            -> Result<LanguageFile, LoadError> {
        use LangFileType::{Dll, Ini, KeyValue};
        let mut lang_file = LanguageFile::default();
        // TODO implement
        match self {
            Dll      => (),
            Ini      => (),
            KeyValue => lang_file.from_keyval(r)?,
        };
        Ok(lang_file)
    }
}

impl FromStr for LangFileType {
    type Err = ParseLangFileTypeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use LangFileType::{Dll, Ini, KeyValue};
        match &s.to_lowercase()[..] {
            "dll"       => Ok(Dll),
            "ini"       => Ok(Ini),
            "key-value" => Ok(KeyValue),
            _           => Err(ParseLangFileTypeError(String::from(s))),
        }
    }
}

/// A mapping of `StringKey` key to `String` values.
///
/// May be read from or written to one of the three file formats for Aoe2
/// language files.
// TODO rename to LangFile after refactoring
// TODO should a default use a capacity that fits all standard strings?
#[derive(Debug, Default)]
pub struct LanguageFile(HashMap<StringKey, String>);

impl LanguageFile {
    /// Reads a language file from an HD Edition-style key-value file.
    ///
    /// This eagerly loads all the strings into memory.
    // TODO fix specification
    pub fn from_keyval(&mut self, input: impl Read) -> Result<(), LoadError> {
        let input = BufReader::new(input);
        for line in input.lines() { self.load_keyval_line(&line?)?; }
        Ok(())
    }

    /// Parses an HD Edition string line.
    ///
    /// The key value pair stored in the line is added to the map, if parsed
    /// successfully.
    /// A `LoadError` is returned if an error occurs while parsing.
    ///
    /// This is incomplete, unquoting and un-escaping is not yet done.
    fn load_keyval_line(&mut self, line: &str) -> Result<(), LoadError> {
        let line = line.trim();
        if line.starts_with("//") || line.is_empty() { return Ok(()); }

        let mut iter = line.chars();
        let id: String = iter.by_ref()
            .take_while(|&c| !char::is_whitespace(c)).collect();
        let string_key = StringKey::from(id);

        // TODO unquoting and un-escaping
        let mut iter = iter.skip_while(|&c| char::is_whitespace(c));
        let value =
            if let Some('"') = iter.next() { unescape(iter, true) }
            else { return Ok(()); };
        self.0.insert(string_key, value);
        Ok(())
    }
}

impl fmt::Display for LanguageFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let strs: Vec<String> = self.0.iter()
            .map(|(k, v)| format!("{}: {}", k, v)).collect();
        write!(f, "{}", strs.join("\n"))
    }
}

/// A file containing language strings.
#[derive(Debug, Default)]
pub struct LangFile {
    strings: HashMap<u32, String>,
    named_strings: HashMap<String, String>,
}

impl LangFile {
    /// Reads a language file from a .DLL.
    ///
    /// This function eagerly loads all the strings into memory.
    pub fn from_dll(mut input: impl Read) -> Result<Self, LoadError> {
        let mut bytes = vec![];
        input.read_to_end(&mut bytes)?;

        let pe = PeFile::from_bytes(&bytes)?;

        LangFile::default().load_pe_file(pe)
    }

    /// Reads a language file from a .INI file, like the ones used by Voobly and
    /// the aoc-language-ini mod.
    ///
    /// This function eagerly loads all the strings into memory.
    /// At this time, the encoding of the language.ini file is assumed to be
    /// Windows codepage 1252.
    pub fn from_ini(input: impl Read) -> Result<Self, LoadError> {
        let mut lang_file = LangFile::default();

        let input = DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1252))
            .build(input);
        let input = BufReader::new(input);
        for line in input.lines() {
            let line = line?;
            lang_file.load_ini_line(&line)?;
        }

        Ok(lang_file)
    }

    /// Reads a language file from an HD Edition-style key-value file.
    ///
    /// This eagerly loads all the strings into memory.
    pub fn from_keyval(input: impl Read) -> Result<Self, LoadError> {
        let mut lang_file = LangFile::default();

        let input = BufReader::new(input);
        for line in input.lines() {
            let line = line?;
            lang_file.load_keyval_line(&line)?;
        }

        Ok(lang_file)
    }

    /// TODO specify
    fn load_pe_file(mut self, pe: PeFile) -> Result<Self, LoadError> {
        for root_dir_entry in pe.resources()?.root()?.entries() {
            if let Ok(Name::Id(6)) = root_dir_entry.name() {
                if let Some(directory) = root_dir_entry.entry()?.dir() {
                    self.load_pe_directory(directory)?;
                }
            }
        }

        Ok(self)
    }

    /// TODO specify
    fn load_pe_directory(&mut self, directory: pelite::resources::Directory)
            -> Result<(), LoadError> {
        for entry in directory.entries() {
            let base_index = if let Name::Id(n) = entry.name()? {
                (n - 1) * 16
            } else {
                continue;
            };
            if let Some(subdir) = entry.entry()?.dir() {
                for data_entry in subdir.entries() {
                    if let Some(data) = data_entry.entry()?.data() {
                        self.load_pe_data(base_index, data.bytes()?)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// TODO specify
    fn load_pe_data(&mut self, mut index: u32, data: &[u8])
            -> Result<(), LoadError> {
        use std::io::{Cursor, Seek, SeekFrom};
        let mut cursor = Cursor::new(data);
        while (cursor.position() as usize) < data.len() {
            let len = (cursor.read_u16::<LE>()? as usize) * 2;
            if len == 0 {
                index += 1;
                continue;
            }
            let start = cursor.position() as usize;
            let (string, _enc, failed) = UTF_16LE.decode(&data[start..start + len]);
            if !failed {
                self.strings.insert(index, string.to_string());
            }
            cursor.seek(SeekFrom::Current(len as i64))?;
            index += 1;
        }
        Ok(())
    }

    /// TODO specify
    fn load_ini_line(&mut self, line: &str) -> Result<(), LoadError> {
        if line.starts_with(';') {
            return Ok(());
        }
        let mut split = line.splitn(2, '=');
        let id = match split.next() {
            Some(id) => id,
            None => return Ok(()),
        };
        let value = match split.next() {
            Some(value) => value,
            None => return Ok(()),
        };

        let value = unescape(value.chars(), false);

        let id = id.parse()?;
        self.strings.insert(id, value);
        Ok(())
    }

    /// Parses an HD Edition string line.
    ///
    /// This is incomplete, unquoting and un-escaping is not yet done.
    fn load_keyval_line(&mut self, line: &str) -> Result<(), LoadError> {
        let line = line.trim();
        if line.starts_with("//") || line.is_empty() {
            return Ok(());
        }

        // TODO first whitespace skip redundant?
        // Is there an API method that does this instead?
        // what characters are allowed in a named string id?
        let mut iter = line.chars().skip_while(|&c| char::is_whitespace(c));
        let id = iter.by_ref()
            .take_while(|&c| !char::is_whitespace(c)).collect::<String>();
        let mut iter = iter.skip_while(|&c| char::is_whitespace(c));
        let value = if let Some('"') = iter.next() {
            unescape(iter, true)
        } else {
            return Ok(());
        };

        if id.chars().all(|ch| ch.is_digit(10)) {
            let id = id.parse()?;
            self.strings.insert(id, value);
        } else {
            self.named_strings.insert(id.to_string(), value);
        }
        Ok(())
    }

    /// Get a string by its numeric index.
    pub fn get(&self, index: u32) -> Option<&str> {
        self.strings.get(&index).map(|string| &**string)
    }

    /// Get a string by name (HD Edition only).
    pub fn get_named(&self, name: &str) -> Option<&str> {
        self.named_strings.get(name).map(|string| &**string)
    }

    /// Set a string at the given numeric index.
    pub fn set(&mut self, index: u32, value: impl Into<String>) {
        self.strings.insert(index, value.into());
    }

    /// Set a string by name.
    pub fn set_named(&mut self, name: &str, value: impl Into<String>) {
        self.named_strings.insert(name.to_string(), value.into());
    }

    /// Delete a string by its numeric index.
    ///
    /// Returns the string if it previously existed.
    pub fn del(&mut self, index: u32) -> Option<String> {
        self.strings.remove(&index)
    }

    /// Delete a string by name.
    ///
    /// Returns the string if it previously existed.
    pub fn del_named(&mut self, name: &str) -> Option<String> {
        self.named_strings.remove(name)
    }

    /// Get an iterator over all the numerically indexed strings.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &str)> {
        self.strings.iter()
            .map(|(id, string)| (*id, &**string))
    }

    /// Get an iterator over all the named strings.
    pub fn iter_named(&self) -> impl Iterator<Item = (&str, &str)> {
        self.named_strings.iter()
            .map(|(name, string)| (&**name, &**string))
    }

    // TODO specify
    pub fn write_to_ini<W: Write>(&self, output: &mut W) -> std::io::Result<()> {
        for (id, string) in self.iter() {
            output.write_all(format!("{}={}\n", id, escape(string, false)).as_bytes())?;
        }
        Ok(())
    }

    // TODO specify
    pub fn write_to_keyval<W: Write>(&self, output: &mut W) -> std::io::Result<()> {
        for (id, string) in self.iter() {
            output.write_all(format!("{} \"{}\"\n", id, escape(string, true)).as_bytes())?;
        }
        output.write_all(b"\n")?;
        for (name, string) in self.iter_named() {
            output.write_all(format!("{} \"{}\"\n", name, escape(string, true)).as_bytes())?;
        }
        Ok(())
    }
}

// TODO specify
fn unescape(escaped: impl Iterator<Item = char>, quoted: bool) -> String {
    let mut unescaped = String::new();
    let mut prev = 'x'; // Innocuous character
    for c in escaped {
        // NOTE this does not support escapes like "\\n", which should print out "\n"
        // literally, instead we get "\" followed by a newline.
        // Could be solved by making `prev` an Option
        match (prev, c) {
            ('\\', '\\') => unescaped.push('\\'),
            ('\\', 'n') => unescaped.push('\n'),
            ('\\', 'r') => unescaped.push('\r'),
            ('\\', 't') => unescaped.push('\t'),
            ('\\', '"') if quoted => unescaped.push('"'),
            (_, '"') if quoted => break,
            (_, '\\') => {}, // Might be escape, wait for one more
            ('\\', c) => {
                // Previous character was escape, but this is not part of a sequence
                unescaped.push('\\');
                unescaped.push(c);
            },
            (_, c) => unescaped.push(c),
        }
        prev = c;
    }
    unescaped
}

// TODO specify
fn escape(source: &str, quoted: bool) -> String {
    let mut escaped = String::new();
    for c in source.chars() {
        match c {
            '\\' => escaped.push_str(r"\\"),
            '\n' => escaped.push_str(r"\n"),
            '\r' => escaped.push_str(r"\r"),
            '\t' => escaped.push_str(r"\t"),
            '"' if quoted => escaped.push_str(r#"\""#),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests converting from an int to a string key.
    #[test]
    fn string_key_from_int() {
        if let StringKey::Num(n) = StringKey::from(0) { assert_eq!(0, n); }
        else { panic!(); }
    }

    /// Tests converting from a string representing an int to a string key.
    #[test]
    fn string_key_from_str_to_int() {
        let s = "57329";
        if let StringKey::Num(n) = StringKey::from(s) { assert_eq!(57329, n); }
        else { panic!(); }
    }

    /// Tests converting from a string not representing an int to a string key.
    #[test]
    fn string_key_from_str_to_str() {
        let s = "grassDaut";
        if let StringKey::Name(n) = StringKey::from(s) { assert_eq!(s, n); }
        else { panic!(); }
    }
}
