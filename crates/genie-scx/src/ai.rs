use crate::util::*;
use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::convert::TryFrom;
use std::io::{Read, Write};
use std::mem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AIErrorCode {
    ConstantAlreadyDefined = 0,
    FileOpenFailed = 1,
    FileReadFailed = 2,
    InvalidIdentifier = 3,
    InvalidKeyword = 4,
    InvalidPreprocessorDirective = 5,
    ListFull = 6,
    MissingArrow = 7,
    MissingClosingParenthesis = 8,
    MissingClosingQuote = 9,
    MissingEndIf = 10,
    MissingFileName = 11,
    MissingIdentifier = 12,
    MissingKeyword = 13,
    MissingLHS = 14,
    MissingOpeningParenthesis = 15,
    MissingPreprocessorSymbol = 16,
    MissingRHS = 17,
    NoRules = 18,
    PreprocessorNestingTooDeep = 19,
    RuleTooLong = 20,
    StringTableFull = 21,
    UndocumentedError = 22,
    UnexpectedElse = 23,
    UnexpectedEndIf = 24,
    UnexpectedError = 25,
    UnexpectedEOF = 26,
    // Update the check in the TryFrom impl if you add anything here
}

/// Found an AI error code that isn't defined.
#[derive(Debug)]
pub struct ParseAIErrorCodeError(u32);

impl std::fmt::Display for ParseAIErrorCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown AI error code {}", self.0)
    }
}

impl std::error::Error for ParseAIErrorCodeError {}

impl TryFrom<u32> for AIErrorCode {
    type Error = ParseAIErrorCodeError;
    fn try_from(n: u32) -> std::result::Result<Self, Self::Error> {
        if n < 27 {
            // I really don't want to write a 27 branch match statement
            // or depend on num_derive _just_ for this, because it needs a proc macro
            // Just keep the above check in sync with the possible values of the AIErrorCode enum
            Ok({
                #[allow(unsafe_code)]
                unsafe {
                    mem::transmute(n)
                }
            })
        } else {
            Err(ParseAIErrorCodeError(n))
        }
    }
}

impl AIErrorCode {
    // TODO remove allow(unused) when AIErrorInfo::write is implemented.
    #[allow(unused)]
    fn to_u32(self) -> u32 {
        // I really don't want to write a 27 branch match statement
        #[allow(unsafe_code)]
        unsafe {
            mem::transmute(self)
        }
    }
}

#[derive(Debug, Clone)]
pub struct AIErrorInfo {
    filename: String,
    line_number: i32,
    description: String,
    error_code: AIErrorCode,
}

fn parse_bytes(bytes: &[u8]) -> std::result::Result<String, ReadStringError> {
    let mut bytes = bytes.to_vec();
    if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
        bytes.truncate(end);
    }
    if bytes.is_empty() {
        Ok("<empty>".to_string())
    } else {
        String::from_utf8(bytes).map_err(|_| ReadStringError::DecodeStringError(DecodeStringError))
    }
}

impl AIErrorInfo {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut filename_bytes = [0; 257];
        input.read_exact(&mut filename_bytes)?;
        let line_number = input.read_i32::<LE>()?;
        let mut description_bytes = [0; 128];
        input.read_exact(&mut description_bytes)?;
        let error_code = AIErrorCode::try_from(input.read_u32::<LE>()?)?;

        let filename = parse_bytes(&filename_bytes)?;
        let description = parse_bytes(&description_bytes)?;

        Ok(AIErrorInfo {
            filename,
            line_number,
            description,
            error_code,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AIFile {
    filename: String,
    content: String,
}

impl AIFile {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let len = input.read_i32::<LE>()? as usize;
        let filename = read_str(input, len)?.expect("missing ai file name");
        let len = input.read_i32::<LE>()? as usize;
        let content = read_str(input, len)?.expect("empty ai file?");

        Ok(Self { filename, content })
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIInfo {
    error: Option<AIErrorInfo>,
    files: Vec<AIFile>,
}

impl AIInfo {
    pub fn from<R: Read>(input: &mut R) -> Result<Option<Self>> {
        let has_ai_files = input.read_u32::<LE>()? != 0;
        let has_error = input.read_u32::<LE>()? != 0;

        if !has_error && !has_ai_files {
            return Ok(None);
        }

        let error = if has_error {
            Some(AIErrorInfo::from(input)?)
        } else {
            None
        };

        let num_ai_files = input.read_u32::<LE>()?;
        let mut files = vec![];
        for _ in 0..num_ai_files {
            files.push(AIFile::from(input)?);
        }

        Ok(Some(Self { error, files }))
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;
        Ok(())
    }
}
