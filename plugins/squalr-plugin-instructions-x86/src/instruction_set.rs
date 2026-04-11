use iced_x86::{Code, Decoder, DecoderOptions, Encoder, Formatter, Instruction, NasmFormatter, Register};
use squalr_engine_api::plugins::instruction_set::InstructionSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum X86InstructionMode {
    Bit32,
    Bit64,
}

impl X86InstructionMode {
    fn bitness(&self) -> u32 {
        match self {
            Self::Bit32 => 32,
            Self::Bit64 => 64,
        }
    }

    fn nop_code(&self) -> Code {
        match self {
            Self::Bit32 => Code::Nopd,
            Self::Bit64 => Code::Nopq,
        }
    }

    fn ret_code(&self) -> Code {
        match self {
            Self::Bit32 => Code::Retnd,
            Self::Bit64 => Code::Retnq,
        }
    }

    fn push_code(&self) -> Code {
        match self {
            Self::Bit32 => Code::Push_r32,
            Self::Bit64 => Code::Push_r64,
        }
    }

    fn pop_code(&self) -> Code {
        match self {
            Self::Bit32 => Code::Pop_r32,
            Self::Bit64 => Code::Pop_r64,
        }
    }

    fn mov_immediate_code(&self) -> Code {
        match self {
            Self::Bit32 => Code::Mov_r32_imm32,
            Self::Bit64 => Code::Mov_r64_imm64,
        }
    }

    fn inc_code(&self) -> Code {
        match self {
            Self::Bit32 => Code::Inc_r32,
            Self::Bit64 => Code::Inc_rm64,
        }
    }

    fn dec_code(&self) -> Code {
        match self {
            Self::Bit32 => Code::Dec_r32,
            Self::Bit64 => Code::Dec_rm64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum X86Register {
    Ax,
    Cx,
    Dx,
    Bx,
    Sp,
    Bp,
    Si,
    Di,
}

impl X86Register {
    fn parse(
        register_name: &str,
        mode: X86InstructionMode,
    ) -> Result<Self, String> {
        match (mode, register_name.trim().to_ascii_lowercase().as_str()) {
            (X86InstructionMode::Bit32, "eax") | (X86InstructionMode::Bit64, "rax") => Ok(Self::Ax),
            (X86InstructionMode::Bit32, "ecx") | (X86InstructionMode::Bit64, "rcx") => Ok(Self::Cx),
            (X86InstructionMode::Bit32, "edx") | (X86InstructionMode::Bit64, "rdx") => Ok(Self::Dx),
            (X86InstructionMode::Bit32, "ebx") | (X86InstructionMode::Bit64, "rbx") => Ok(Self::Bx),
            (X86InstructionMode::Bit32, "esp") | (X86InstructionMode::Bit64, "rsp") => Ok(Self::Sp),
            (X86InstructionMode::Bit32, "ebp") | (X86InstructionMode::Bit64, "rbp") => Ok(Self::Bp),
            (X86InstructionMode::Bit32, "esi") | (X86InstructionMode::Bit64, "rsi") => Ok(Self::Si),
            (X86InstructionMode::Bit32, "edi") | (X86InstructionMode::Bit64, "rdi") => Ok(Self::Di),
            _ => Err(format!(
                "Unsupported {} register '{}'.",
                match mode {
                    X86InstructionMode::Bit32 => "x86",
                    X86InstructionMode::Bit64 => "x64",
                },
                register_name.trim()
            )),
        }
    }

    fn to_iced_register(
        self,
        mode: X86InstructionMode,
    ) -> Register {
        match (mode, self) {
            (X86InstructionMode::Bit32, Self::Ax) => Register::EAX,
            (X86InstructionMode::Bit32, Self::Cx) => Register::ECX,
            (X86InstructionMode::Bit32, Self::Dx) => Register::EDX,
            (X86InstructionMode::Bit32, Self::Bx) => Register::EBX,
            (X86InstructionMode::Bit32, Self::Sp) => Register::ESP,
            (X86InstructionMode::Bit32, Self::Bp) => Register::EBP,
            (X86InstructionMode::Bit32, Self::Si) => Register::ESI,
            (X86InstructionMode::Bit32, Self::Di) => Register::EDI,
            (X86InstructionMode::Bit64, Self::Ax) => Register::RAX,
            (X86InstructionMode::Bit64, Self::Cx) => Register::RCX,
            (X86InstructionMode::Bit64, Self::Dx) => Register::RDX,
            (X86InstructionMode::Bit64, Self::Bx) => Register::RBX,
            (X86InstructionMode::Bit64, Self::Sp) => Register::RSP,
            (X86InstructionMode::Bit64, Self::Bp) => Register::RBP,
            (X86InstructionMode::Bit64, Self::Si) => Register::RSI,
            (X86InstructionMode::Bit64, Self::Di) => Register::RDI,
        }
    }
}

#[derive(Debug)]
struct X86InstructionSetBase {
    instruction_set_id: &'static str,
    display_name: &'static str,
    mode: X86InstructionMode,
}

impl X86InstructionSetBase {
    fn new(
        instruction_set_id: &'static str,
        display_name: &'static str,
        mode: X86InstructionMode,
    ) -> Self {
        Self {
            instruction_set_id,
            display_name,
            mode,
        }
    }

    fn assemble_instruction_sequence(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String> {
        let mut encoder = Encoder::new(self.mode.bitness());
        let mut current_ip = 0u64;

        for instruction_text in split_instruction_sequence(assembly_source) {
            let instruction = self.parse_instruction(instruction_text)?;
            let instruction_length = encoder
                .encode(&instruction, current_ip)
                .map_err(|error| format!("Failed to encode {} instruction '{}': {}.", self.display_name, instruction_text.trim(), error))?;

            current_ip = current_ip.saturating_add(instruction_length as u64);
        }

        let encoded_bytes = encoder.take_buffer();

        if encoded_bytes.is_empty() {
            return Err(format!("{} assembly source must not be empty.", self.display_name));
        }

        Ok(encoded_bytes)
    }

    fn parse_instruction(
        &self,
        instruction_text: &str,
    ) -> Result<Instruction, String> {
        let (mnemonic, operands_text) = split_mnemonic_and_operands(instruction_text);

        match mnemonic.as_str() {
            "nop" => {
                ensure_no_operands(mnemonic.as_str(), operands_text)?;

                Ok(Instruction::with(self.mode.nop_code()))
            }
            "ret" => {
                ensure_no_operands(mnemonic.as_str(), operands_text)?;

                Ok(Instruction::with(self.mode.ret_code()))
            }
            "push" => {
                let register = self.parse_register_operand(mnemonic.as_str(), operands_text)?;

                Instruction::with1(self.mode.push_code(), register)
                    .map_err(|error| format!("Failed to assemble {} '{}': {}.", self.display_name, instruction_text.trim(), error))
            }
            "pop" => {
                let register = self.parse_register_operand(mnemonic.as_str(), operands_text)?;

                Instruction::with1(self.mode.pop_code(), register)
                    .map_err(|error| format!("Failed to assemble {} '{}': {}.", self.display_name, instruction_text.trim(), error))
            }
            "inc" => {
                let register = self.parse_register_operand(mnemonic.as_str(), operands_text)?;

                Instruction::with1(self.mode.inc_code(), register)
                    .map_err(|error| format!("Failed to assemble {} '{}': {}.", self.display_name, instruction_text.trim(), error))
            }
            "dec" => {
                let register = self.parse_register_operand(mnemonic.as_str(), operands_text)?;

                Instruction::with1(self.mode.dec_code(), register)
                    .map_err(|error| format!("Failed to assemble {} '{}': {}.", self.display_name, instruction_text.trim(), error))
            }
            "mov" => {
                let (destination_operand, source_operand) = split_two_operands(operands_text)?;
                let destination_register = X86Register::parse(destination_operand, self.mode)?.to_iced_register(self.mode);

                match self.mode {
                    X86InstructionMode::Bit32 => {
                        let immediate_value = parse_signed_immediate(source_operand, 32)?;

                        Instruction::with2(self.mode.mov_immediate_code(), destination_register, immediate_value as i32)
                            .map_err(|error| format!("Failed to assemble {} '{}': {}.", self.display_name, instruction_text.trim(), error))
                    }
                    X86InstructionMode::Bit64 => {
                        let immediate_value = parse_signed_immediate(source_operand, 64)?;

                        Instruction::with2(self.mode.mov_immediate_code(), destination_register, immediate_value as i64)
                            .map_err(|error| format!("Failed to assemble {} '{}': {}.", self.display_name, instruction_text.trim(), error))
                    }
                }
            }
            _ => Err(format!("Unsupported {} mnemonic '{}'.", self.display_name, mnemonic)),
        }
    }

    fn parse_register_operand(
        &self,
        mnemonic: &str,
        operands_text: &str,
    ) -> Result<Register, String> {
        X86Register::parse(expect_single_operand(mnemonic, operands_text)?, self.mode).map(|register| register.to_iced_register(self.mode))
    }

    fn disassemble_instruction_sequence(
        &self,
        instruction_bytes: &[u8],
    ) -> Result<String, String> {
        if instruction_bytes.is_empty() {
            return Ok(String::new());
        }

        let mut instruction_texts = Vec::new();
        let mut formatter = NasmFormatter::new();
        let mut byte_offset = 0usize;

        while byte_offset < instruction_bytes.len() {
            let mut decoder = Decoder::with_ip(self.mode.bitness(), &instruction_bytes[byte_offset..], byte_offset as u64, DecoderOptions::NONE);
            let instruction = decoder.decode();
            let instruction_length = instruction.len();

            if instruction.is_invalid() || instruction_length == 0 {
                instruction_texts.push(format!("db 0x{:02X}", instruction_bytes[byte_offset]));
                byte_offset += 1;
                continue;
            }

            let mut instruction_text = String::new();
            formatter.format(&instruction, &mut instruction_text);
            instruction_texts.push(normalize_instruction_text(instruction_text));
            byte_offset += instruction_length;
        }

        Ok(instruction_texts.join("; "))
    }
}

#[derive(Debug)]
pub struct X86InstructionSet {
    inner: X86InstructionSetBase,
}

impl X86InstructionSet {
    pub fn new() -> Self {
        Self {
            inner: X86InstructionSetBase::new("x86", "x86", X86InstructionMode::Bit32),
        }
    }
}

impl Default for X86InstructionSet {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionSet for X86InstructionSet {
    fn get_instruction_set_id(&self) -> &str {
        self.inner.instruction_set_id
    }

    fn get_display_name(&self) -> &str {
        self.inner.display_name
    }

    fn assemble(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String> {
        self.inner.assemble_instruction_sequence(assembly_source)
    }

    fn disassemble(
        &self,
        instruction_bytes: &[u8],
    ) -> Result<String, String> {
        self.inner.disassemble_instruction_sequence(instruction_bytes)
    }
}

#[derive(Debug)]
pub struct X64InstructionSet {
    inner: X86InstructionSetBase,
}

impl X64InstructionSet {
    pub fn new() -> Self {
        Self {
            inner: X86InstructionSetBase::new("x64", "x64", X86InstructionMode::Bit64),
        }
    }
}

impl Default for X64InstructionSet {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionSet for X64InstructionSet {
    fn get_instruction_set_id(&self) -> &str {
        self.inner.instruction_set_id
    }

    fn get_display_name(&self) -> &str {
        self.inner.display_name
    }

    fn assemble(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String> {
        self.inner.assemble_instruction_sequence(assembly_source)
    }

    fn disassemble(
        &self,
        instruction_bytes: &[u8],
    ) -> Result<String, String> {
        self.inner.disassemble_instruction_sequence(instruction_bytes)
    }
}

fn split_instruction_sequence(assembly_source: &str) -> Vec<&str> {
    assembly_source
        .split(|character| character == ';' || character == '\n' || character == '\r')
        .map(str::trim)
        .filter(|instruction_text| !instruction_text.is_empty())
        .collect()
}

fn split_mnemonic_and_operands(instruction_text: &str) -> (String, &str) {
    let trimmed_instruction_text = instruction_text.trim();

    if let Some(mnemonic_end_index) = trimmed_instruction_text.find(char::is_whitespace) {
        (
            trimmed_instruction_text[..mnemonic_end_index]
                .trim()
                .to_ascii_lowercase(),
            trimmed_instruction_text[mnemonic_end_index..].trim(),
        )
    } else {
        (trimmed_instruction_text.to_ascii_lowercase(), "")
    }
}

fn normalize_instruction_text(instruction_text: String) -> String {
    let mut normalized_instruction_text = String::with_capacity(instruction_text.len() + 4);
    let mut previous_character = '\0';

    for current_character in instruction_text.trim().chars() {
        normalized_instruction_text.push(current_character);

        if current_character == ',' && previous_character != ' ' {
            normalized_instruction_text.push(' ');
        }

        previous_character = current_character;
    }

    normalized_instruction_text
}

fn ensure_no_operands(
    mnemonic: &str,
    operands_text: &str,
) -> Result<(), String> {
    if operands_text.trim().is_empty() {
        Ok(())
    } else {
        Err(format!("Mnemonic '{}' does not take operands.", mnemonic))
    }
}

fn expect_single_operand<'a>(
    mnemonic: &str,
    operands_text: &'a str,
) -> Result<&'a str, String> {
    let operand = operands_text.trim();

    if operand.is_empty() {
        Err(format!("Mnemonic '{}' requires one operand.", mnemonic))
    } else if operand.contains(',') {
        Err(format!("Mnemonic '{}' only supports one operand.", mnemonic))
    } else {
        Ok(operand)
    }
}

fn split_two_operands(operands_text: &str) -> Result<(&str, &str), String> {
    let Some((left_operand, right_operand)) = operands_text.split_once(',') else {
        return Err(String::from("Expected two operands separated by a comma."));
    };
    let left_operand = left_operand.trim();
    let right_operand = right_operand.trim();

    if left_operand.is_empty() || right_operand.is_empty() {
        Err(String::from("Expected both operands to be present."))
    } else {
        Ok((left_operand, right_operand))
    }
}

fn parse_signed_immediate(
    immediate_text: &str,
    bit_width: u32,
) -> Result<i128, String> {
    let trimmed_immediate_text = immediate_text.trim();

    if trimmed_immediate_text.is_empty() {
        return Err(String::from("Immediate operand must not be empty."));
    }

    let parsed_value = if let Some(hexadecimal_digits) = trimmed_immediate_text
        .strip_prefix("0x")
        .or_else(|| trimmed_immediate_text.strip_prefix("0X"))
    {
        i128::from_str_radix(hexadecimal_digits, 16).map_err(|error| format!("Invalid hexadecimal immediate '{}': {}.", immediate_text, error))?
    } else {
        trimmed_immediate_text
            .parse::<i128>()
            .map_err(|error| format!("Invalid immediate '{}': {}.", immediate_text, error))?
    };
    let minimum_value = -(1_i128 << (bit_width - 1));
    let maximum_value = (1_i128 << bit_width) - 1;

    if parsed_value < minimum_value || parsed_value > maximum_value {
        return Err(format!("Immediate '{}' does not fit in {} bits.", immediate_text, bit_width));
    }

    Ok(parsed_value)
}
