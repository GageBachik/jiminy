use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/instructions");
    println!("cargo:rerun-if-changed=src/error.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_program.rs");

    // Parse instruction files and extract metadata
    let instructions = extract_instruction_metadata();

    // Parse error definitions from error.rs
    let errors = extract_error_metadata();

    // Generate the program enum and dispatch
    let generated_code = generate_program_code(&instructions, &errors);

    // Write to output file
    fs::write(&dest_path, &generated_code).unwrap();

    // Also write to src/generated.rs for shank IDL generation
    let src_generated_path = Path::new("src/generated.rs");
    fs::write(src_generated_path, &generated_code).unwrap();

    println!(
        "cargo:rustc-env=GENERATED_PROGRAM_PATH={}",
        dest_path.display()
    );
}

#[derive(Debug)]
struct InstructionMeta {
    name: String,
    discriminator: u8,
    accounts: Vec<AccountMeta>,
    fields: Vec<FieldMeta>,
}

#[derive(Debug)]
struct AccountMeta {
    name: String,
    index: usize,
    desc: String,
    attrs: Vec<String>,
}

#[derive(Debug)]
struct FieldMeta {
    name: String,
    field_type: String,
}

fn extract_instruction_metadata() -> Vec<InstructionMeta> {
    let mut instructions = Vec::new();

    // Find all instruction files
    let instruction_dir = Path::new("src/instructions");
    if instruction_dir.exists() {
        for entry in fs::read_dir(instruction_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rs")
                && path.file_name().and_then(|s| s.to_str()) != Some("mod.rs")
            {
                if let Some(instruction) = parse_instruction_file(&path) {
                    instructions.push(instruction);
                }
            }
        }
    }

    instructions.sort_by_key(|i| i.discriminator);
    instructions
}

fn parse_instruction_file(path: &Path) -> Option<InstructionMeta> {
    let content = fs::read_to_string(path).ok()?;

    // Look for either define_instruction_with_metadata! or define_instruction! macro
    let start = content
        .find("define_instruction_with_metadata!(")
        .or_else(|| content.find("define_instruction!("))?;
    let mut paren_count = 0;
    let mut in_macro = false;
    let mut macro_content = String::new();

    for (_i, ch) in content[start..].char_indices() {
        if ch == '(' {
            paren_count += 1;
            in_macro = true;
        } else if ch == ')' {
            paren_count -= 1;
        }

        if in_macro {
            macro_content.push(ch);
        }

        if paren_count == 0 && in_macro {
            break;
        }
    }

    parse_macro_content(&macro_content)
}

fn parse_macro_content(content: &str) -> Option<InstructionMeta> {
    let lines: Vec<&str> = content.lines().collect();

    let mut name = String::new();
    let mut discriminator = 0u8;
    let mut accounts = Vec::new();
    let mut fields = Vec::new();

    let mut in_accounts = false;
    let mut in_data = false;
    let mut account_index = 0;

    for line in lines {
        let line = line.trim();

        // Extract discriminant
        if line.starts_with("discriminant:") {
            if let Some(num) = line.split(':').nth(1) {
                discriminator = num.trim().trim_end_matches(',').parse().unwrap_or(0);
            }
            continue;
        }

        // Extract instruction name (first identifier after discriminant)
        if name.is_empty()
            && !line.is_empty()
            && !line.starts_with("define_instruction")
            && !line.starts_with("discriminant:")
            && line.ends_with(',')
        {
            name = line.trim_end_matches(',').to_string();
            continue;
        }

        // Track sections
        if line.starts_with("accounts:") {
            in_accounts = true;
            in_data = false;
            continue;
        } else if line.starts_with("data:") {
            in_accounts = false;
            in_data = true;
            continue;
        } else if line.starts_with("process:") {
            break;
        }

        // Parse account lines with new format
        if in_accounts && line.contains("desc:") {
            if let Some(account) = parse_new_account_line(line, account_index) {
                accounts.push(account);
                account_index += 1;
            }
        }

        // Parse data fields
        if in_data && line.contains(':') && !line.starts_with("data:") && !line.starts_with('}') {
            if let Some(field) = parse_field_line(line) {
                fields.push(field);
            }
        }
    }

    if !name.is_empty() {
        Some(InstructionMeta {
            name,
            discriminator,
            accounts,
            fields,
        })
    } else {
        None
    }
}

fn parse_new_account_line(line: &str, index: usize) -> Option<AccountMeta> {
    // Parse lines like: authority: signer => writable, desc: "Authority of the vault",
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 3 {
        return None;
    }

    let name = parts[0].trim().to_string();
    let account_def = parts[1].trim();
    let desc_part = parts[2].trim().trim_end_matches(',').trim_matches('"');

    // Parse account type and validation from account_def
    let mut attrs = Vec::new();
    if account_def.contains("signer") {
        attrs.push("signer".to_string());
    }
    if account_def.contains("writable") || account_def.contains("=> writable") {
        attrs.push("writable".to_string());
    }
    // Uninitialized accounts are always writable since they're being created
    if account_def.contains("uninitialized") {
        attrs.push("writable".to_string());
    }

    Some(AccountMeta {
        name,
        index,
        desc: desc_part.to_string(),
        attrs,
    })
}

fn parse_field_line(line: &str) -> Option<FieldMeta> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts[0].trim().to_string();
    let field_type = parts[1].trim().trim_end_matches(',').to_string();

    Some(FieldMeta { name, field_type })
}

#[derive(Debug)]
struct ErrorMeta {
    name: String,
    variants: Vec<ErrorVariant>,
}

#[derive(Debug)]
struct ErrorVariant {
    name: String,
    code: u32,
}

fn extract_error_metadata() -> Vec<ErrorMeta> {
    let error_path = Path::new("src/error.rs");
    if !error_path.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(error_path).unwrap_or_default();

    // Look for define_errors! macro calls
    let mut errors = Vec::new();

    if let Some(start) = content.find("define_errors!") {
        if let Some(error_meta) = parse_error_macro(&content[start..]) {
            errors.push(error_meta);
        }
    }

    errors
}

fn parse_error_macro(content: &str) -> Option<ErrorMeta> {
    // Find the macro content between braces
    let start = content.find('{')?;
    let mut brace_count = 0;
    let mut in_macro = false;
    let mut macro_content = String::new();

    for ch in content[start..].chars() {
        if ch == '{' {
            brace_count += 1;
            in_macro = true;
        } else if ch == '}' {
            brace_count -= 1;
        }

        if in_macro {
            macro_content.push(ch);
        }

        if brace_count == 0 && in_macro {
            break;
        }
    }

    // Parse the macro content
    let lines: Vec<&str> = macro_content.lines().collect();
    let mut error_name = String::new();
    let mut variants = Vec::new();

    for line in lines {
        let line = line.trim();

        // First non-empty line after { should be the error type name
        if error_name.is_empty() && !line.is_empty() && !line.starts_with('{') {
            error_name = line.trim_end_matches(',').to_string();
            continue;
        }

        // Parse error variants: "ErrorName = code,"
        if line.contains('=') && !line.starts_with('{') && !line.starts_with('}') {
            if let Some((name, code)) = line.split_once('=') {
                let name = name.trim().to_string();
                if let Ok(code) = code.trim().trim_end_matches(',').parse::<u32>() {
                    variants.push(ErrorVariant { name, code });
                }
            }
        }
    }

    if !error_name.is_empty() && !variants.is_empty() {
        Some(ErrorMeta {
            name: error_name,
            variants,
        })
    } else {
        None
    }
}

fn generate_program_code(instructions: &[InstructionMeta], errors: &[ErrorMeta]) -> String {
    let mut code = String::new();

    code.push_str("use shank::ShankInstruction;\n");
    if !errors.is_empty() {
        code.push_str("use shank::ShankType;\n");
        code.push_str("use pinocchio::program_error::ProgramError;\n");
    }
    code.push('\n');

    // Since shank looks for declare_id! patterns in all source files,
    // and we already have pinocchio_pubkey::declare_id! in lib.rs,
    // we don't need to add another one here. Shank should parse the existing one.
    // If it can't, we can override with the -p flag when running shank idl.\n");

    // Generate error enums first
    for error in errors {
        code.push_str(&format!("// Generated error enum: {}\n", error.name));
        code.push_str("#[derive(Clone, PartialEq, ShankType)]\n");
        code.push_str(&format!("pub enum {} {{\n", error.name));

        for variant in &error.variants {
            code.push_str(&format!("    {} = {},\n", variant.name, variant.code));
        }

        code.push_str("}\n\n");

        // Generate From implementation
        code.push_str(&format!("impl From<{}> for ProgramError {{\n", error.name));
        code.push_str(&format!("    fn from(e: {}) -> Self {{\n", error.name));
        code.push_str("        Self::Custom(e as u32)\n");
        code.push_str("    }\n");
        code.push_str("}\n\n");
    }

    // Only generate if we have instructions
    if instructions.is_empty() {
        code.push_str("// No instructions found - using fallback\n");
        code.push_str("pub enum ProgramInstructions {}\n\n");
        code.push_str("pub fn process_instruction(_program_id: &pinocchio::pubkey::Pubkey, _accounts: &[pinocchio::account_info::AccountInfo], _instruction_data: &[u8]) -> pinocchio::ProgramResult {\n");
        code.push_str("    Err(pinocchio::program_error::ProgramError::InvalidInstructionData)\n");
        code.push_str("}\n");
        return code;
    }

    // Generate shank enum
    code.push_str("#[repr(u8)]\n");
    code.push_str("#[derive(Clone, Debug, PartialEq, ShankInstruction)]\n");
    code.push_str("pub enum ProgramInstructions {\n");

    for instruction in instructions {
        // Add account attributes
        for account in &instruction.accounts {
            code.push_str(&format!("    #[account({}", account.index));
            for attr in &account.attrs {
                code.push_str(&format!(", {attr}"));
            }
            code.push_str(&format!(
                ", name = \"{}\", desc = \"{}\")]\n",
                account.name, account.desc
            ));
        }

        // Add variant
        code.push_str(&format!("    {} {{\n", instruction.name));
        for field in &instruction.fields {
            code.push_str(&format!("        {}: {},\n", field.name, field.field_type));
        }
        code.push_str("    },\n\n");
    }
    code.push_str("}\n\n");

    // Generate ShankAccount definitions for state structs
    code.push_str("// ShankAccount definitions for state structs\n");
    code.push_str("// These are generated for IDL compatibility\n");
    code.push_str("#[repr(C)]\n");
    code.push_str("#[derive(Clone, shank::ShankAccount)]\n");
    code.push_str("pub struct Vote {\n");
    code.push_str("    pub token: [u8; 32],\n");
    code.push_str("    pub true_votes: [u8; 8],\n");
    code.push_str("    pub false_votes: [u8; 8],\n");
    code.push_str("    pub end_timestamp: [u8; 8],\n");
    code.push_str("    pub vault_bump: u8,\n");
    code.push_str("}\n\n");

    code.push_str("#[repr(C)]\n");
    code.push_str("#[derive(Clone, shank::ShankAccount)]\n");
    code.push_str("pub struct Position {\n");
    code.push_str("    pub amount: [u8; 8],\n");
    code.push_str("    pub side: u8,\n");
    code.push_str("    pub bump: u8,\n");
    code.push_str("}\n\n");

    // Generate dispatch function
    code.push_str("pub fn process_instruction(\n");
    code.push_str("    program_id: &pinocchio::pubkey::Pubkey,\n");
    code.push_str("    accounts: &[pinocchio::account_info::AccountInfo],\n");
    code.push_str("    instruction_data: &[u8],\n");
    code.push_str(") -> pinocchio::ProgramResult {\n");
    code.push_str("    if program_id != &crate::ID {\n");
    code.push_str(
        "        return Err(pinocchio::program_error::ProgramError::IncorrectProgramId);\n",
    );
    code.push_str("    }\n\n");
    code.push_str("    match instruction_data.first() {\n");

    for instruction in instructions {
        code.push_str(&format!(
            "        Some({}) => {{\n",
            instruction.discriminator
        ));
        code.push_str(&format!("            crate::instructions::{}Instruction::try_from((accounts, &instruction_data[1..]))?.process()\n", instruction.name));
        code.push_str("        }\n");
    }

    // Use the first error type if available, otherwise use a generic error
    if let Some(error) = errors.first() {
        code.push_str(&format!(
            "        _ => Err({}::InvalidDiscriminator.into()),\n",
            error.name
        ));
    } else {
        code.push_str(
            "        _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),\n",
        );
    }
    code.push_str("    }\n");
    code.push_str("}\n");

    code
}
