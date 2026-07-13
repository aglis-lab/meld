use std::{ops::Range, vec};

use anyhow::{Result, anyhow};

use crate::utils;

const HEADER_SIZE: usize = 42;

#[derive(Debug, Clone)]
pub struct TextLoc {
    start: u32,
    end: u32,
}

pub struct Header {
    version: u16,
    instruction_length: u32,
    content_length: u32,
    checksum: String,
}

pub struct Program {
    header: Header,
    instructions: Box<[u8]>,
    content: Box<[u8]>,
}

impl TextLoc {
    #[inline(always)]
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    #[inline(always)]
    pub fn as_range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}

impl Program {
    pub fn new(data: &vec::Vec<u8>) -> Result<Self> {
        let header = Header::new(data);
        let expected_version = utils::tef_version()?;
        if expected_version != header.version() {
            return Err(anyhow::anyhow!(
                "Version mismatch {} != {}",
                expected_version,
                header.version()
            ));
        }

        let ins_start = HEADER_SIZE;
        let ins_end = HEADER_SIZE + header.instruction_length() as usize;
        let content_start = ins_end;
        let content_end = content_start + header.content_length() as usize;
        if data.len() < ins_end || data.len() < content_end {
            return Err(anyhow::anyhow!("Data is too short"));
        }

        let checksum = sha256::digest(&data[ins_start..content_end]);
        if checksum != header.checksum() {
            return Err(anyhow::anyhow!("Checksum mismatch",));
        }

        let instructions = data[ins_start..ins_end].to_vec().into_boxed_slice();
        let content = data[content_start..content_end].to_vec().into_boxed_slice();

        Ok(Self {
            header,
            instructions,
            content,
        })
    }

    #[inline(always)]
    pub fn header(&self) -> &Header {
        &self.header
    }

    #[inline(always)]
    pub fn get_content(&self, range: TextLoc) -> Result<&[u8]> {
        if range.end as usize > self.content.len() {
            return Err(anyhow::anyhow!("Content range out of bounds"));
        }

        Ok(&self.content[range.as_range()])
    }

    #[inline(always)]
    pub fn get_op(&self, pc: usize) -> Result<u8> {
        let opcode = self
            .instructions
            .get(pc)
            .ok_or_else(|| anyhow!("Failed to get opcode because of out of bounds, pc: {}", pc))?;
        Ok(*opcode)
    }

    #[inline(always)]
    pub fn get_op_range(&self, pc: usize) -> Result<TextLoc> {
        let bytes = self.read_op::<8>(pc)?;
        let start = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let end = u32::from_le_bytes(bytes[4..8].try_into().unwrap());

        Ok(TextLoc::new(start, end))
    }

    #[inline(always)]
    pub fn get_op_u32(&self, pc: usize) -> Result<u32> {
        let bytes = self.read_op::<4>(pc)?;
        Ok(u32::from_le_bytes(*bytes))
    }

    #[inline(always)]
    pub fn get_op_u8(&self, pc: usize) -> Result<u8> {
        let opcode = self
            .instructions
            .get(pc)
            .ok_or_else(|| anyhow!("Failed to get opcode because of out of bounds, pc: {}", pc))?;
        Ok(*opcode)
    }

    #[inline(always)]
    fn read_op<const N: usize>(&self, pc: usize) -> Result<&[u8; N]> {
        Ok(self
            .instructions
            .get(pc..pc + N)
            .ok_or_else(|| {
                anyhow!(
                    "Instruction out of bounds, pc: {}, N: {}, instruction: {}",
                    pc,
                    N,
                    self.instructions.len()
                )
            })?
            .try_into()
            .unwrap())
    }
}

impl Header {
    #[inline(always)]
    pub fn new(data: &vec::Vec<u8>) -> Self {
        Self {
            version: u16::from_le_bytes([data[0], data[1]]),
            instruction_length: u32::from_le_bytes(data[2..6].try_into().unwrap()),
            content_length: u32::from_le_bytes(data[6..10].try_into().unwrap()),
            checksum: hex::encode(&data[10..42]),
        }
    }

    #[inline(always)]
    pub fn version(&self) -> u16 {
        self.version
    }

    #[inline(always)]
    pub fn instruction_length(&self) -> u32 {
        self.instruction_length
    }

    #[inline(always)]
    pub fn content_length(&self) -> u32 {
        self.content_length
    }

    #[inline(always)]
    pub fn checksum(&self) -> &str {
        &self.checksum
    }
}
