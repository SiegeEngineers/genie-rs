use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{read_str, DecodeStringError, ReadStringError};
use std::convert::TryFrom;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u32)]
pub enum AIErrorCode {
    ///
    ConstantAlreadyDefined = 0,
    ///
    FileOpenFailed = 1,
    ///
    FileReadFailed = 2,
    ///
    InvalidIdentifier = 3,
    ///
    InvalidKeyword = 4,
    ///
    InvalidPreprocessorDirective = 5,
    ///
    ListFull = 6,
    ///
    MissingArrow = 7,
    ///
    MissingClosingParenthesis = 8,
    ///
    MissingClosingQuote = 9,
    ///
    MissingEndIf = 10,
    ///
    MissingFileName = 11,
    ///
    MissingIdentifier = 12,
    ///
    MissingKeyword = 13,
    ///
    MissingLHS = 14,
    ///
    MissingOpeningParenthesis = 15,
    ///
    MissingPreprocessorSymbol = 16,
    ///
    MissingRHS = 17,
    ///
    NoRules = 18,
    ///
    PreprocessorNestingTooDeep = 19,
    ///
    RuleTooLong = 20,
    ///
    StringTableFull = 21,
    ///
    UndocumentedError = 22,
    ///
    UnexpectedElse = 23,
    ///
    UnexpectedEndIf = 24,
    ///
    UnexpectedError = 25,
    ///
    UnexpectedEOF = 26,
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
    pub fn read_from(mut input: impl Read) -> Result<Self> {
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
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let len = input.read_i32::<LE>()? as usize;
        let filename = read_str(&mut input, len)?.expect("missing ai file name");
        let len = input.read_i32::<LE>()? as usize;
        let content = read_str(&mut input, len)?.expect("empty ai file?");

        Ok(Self { filename, content })
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIInfo {
    error: Option<AIErrorInfo>,
    files: Vec<AIFile>,
}

impl AIInfo {
    pub fn read_from(mut input: impl Read) -> Result<Option<Self>> {
        let has_ai_files = input.read_u32::<LE>()? != 0;
        let has_error = input.read_u32::<LE>()? != 0;

        if !has_error && !has_ai_files {
            return Ok(None);
        }

        let error = if has_error {
            Some(AIErrorInfo::read_from(&mut input)?)
        } else {
            None
        };

        let num_ai_files = input.read_u32::<LE>()?;
        let mut files = vec![];
        for _ in 0..num_ai_files {
            files.push(AIFile::read_from(&mut input)?);
        }

        Ok(Some(Self { error, files }))
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;
        Ok(())
    }
}
