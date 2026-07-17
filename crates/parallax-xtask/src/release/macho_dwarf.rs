//! Post-link Mach-O DWARF line-table embedding for release binaries.
//!
//! Apple `ld` leaves only OSO debug-map stabs in the final executable.
//! `dsymutil` links relocated DWARF into a companion; this module merges that
//! companion's `__DWARF` sections into the executable with a correct
//! `LC_SEGMENT_64` (Go-style: `vmsize = 0`, `vmaddr` shared with `__LINKEDIT`,
//! file payload before `__LINKEDIT`) so `verify_object` and `addr2line` see
//! real `__debug_line` without a shipped symbol companion.
//!
//! Requires link-time header padding (`-Wl,-headerpad,0x10000`). On macOS the
//! rewritten binary is re-signed ad-hoc with `codesign`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};
use object::{BinaryFormat, Object, ObjectSection};

const MH_MAGIC_64: u32 = 0xfeed_facf;
const LC_SEGMENT_64: u32 = 0x19;
const LC_SYMTAB: u32 = 0x2;
const LC_DYSYMTAB: u32 = 0xb;
const LC_DYLD_INFO_ONLY: u32 = 0x8000_0022;
const LC_FUNCTION_STARTS: u32 = 0x26;
const LC_DATA_IN_CODE: u32 = 0x29;
const LC_CODE_SIGNATURE: u32 = 0x1d;
const LC_DYLD_EXPORTS_TRIE: u32 = 0x8000_0033;
const LC_DYLD_CHAINED_FIXUPS: u32 = 0x8000_0034;
const S_ATTR_DEBUG: u32 = 0x0200_0000;
const PAGE: u64 = 0x4000;
const HEADERPAD_HINT: &str =
    "build Apple targets with -C link-arg=-Wl,-headerpad,0x10000 (see .cargo/config.toml)";

/// Ensure a release binary carries in-binary line tables the verifier accepts.
///
/// Mach-O binaries without `__debug_line` are rewritten in place from a dSYM
/// companion (created with `dsymutil` when missing). ELF is left unchanged.
pub(super) fn ensure_line_tables(binary: &Path) -> Result<()> {
    let bytes = std::fs::read(binary)
        .with_context(|| format!("read release binary {}", binary.display()))?;
    if has_line_table_section(&bytes)? {
        return Ok(());
    }
    let format = object::File::parse(bytes.as_slice())
        .context("parse release binary for line-table preparation")?
        .format();
    match format {
        BinaryFormat::MachO => embed_and_resign(binary, &bytes),
        BinaryFormat::Elf => bail!(
            "release binary is missing line tables (ELF must retain .debug_line from the link)"
        ),
        other => bail!("unsupported release binary format {other:?} for line-table preparation"),
    }
}

fn embed_and_resign(binary: &Path, bytes: &[u8]) -> Result<()> {
    println!(
        "==> embed Mach-O DWARF line tables into {}",
        binary.display()
    );
    let companion = ensure_dsym_companion(binary)?;
    let rewritten = embed_dwarf_from_companion(bytes, &companion)?;
    std::fs::write(binary, &rewritten)
        .with_context(|| format!("write DWARF-embedded binary {}", binary.display()))?;
    resign_macho(binary)?;
    ensure!(
        has_line_table_section(&std::fs::read(binary)?)?,
        "Mach-O rewrite did not produce a line-table section"
    );
    Ok(())
}

fn has_line_table_section(bytes: &[u8]) -> Result<bool> {
    let object = object::File::parse(bytes).context("parse binary for line-table sections")?;
    Ok(object.sections().any(|section| {
        section
            .name()
            .is_ok_and(super::verify::is_line_table_section_name)
    }))
}

fn ensure_dsym_companion(binary: &Path) -> Result<PathBuf> {
    if let Some(existing) = find_dsym_companion(binary)? {
        return Ok(existing);
    }
    ensure!(
        cfg!(target_os = "macos"),
        "Mach-O release binary lacks line tables and no dSYM companion is present; \
         dsymutil is only available when packaging on macOS"
    );
    println!("==> dsymutil {}", binary.display());
    let status = Command::new("dsymutil")
        .arg(binary)
        .status()
        .context("start dsymutil")?;
    ensure!(status.success(), "dsymutil failed with {status}");
    find_dsym_companion(binary)?.with_context(|| {
        format!(
            "dsymutil did not produce a DWARF companion next to {}",
            binary.display()
        )
    })
}

fn find_dsym_companion(binary: &Path) -> Result<Option<PathBuf>> {
    let dsym_dir = dsym_bundle_path(binary).join("Contents/Resources/DWARF");
    if !dsym_dir.is_dir() {
        return Ok(None);
    }
    let mut entries = std::fs::read_dir(&dsym_dir)
        .with_context(|| format!("read dSYM DWARF directory {}", dsym_dir.display()))?
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    let files = entries
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    ensure!(
        files.len() <= 1,
        "dSYM DWARF directory {} has multiple companions",
        dsym_dir.display()
    );
    Ok(files.into_iter().next())
}

fn dsym_bundle_path(binary: &Path) -> PathBuf {
    let mut name = binary.file_name().unwrap_or_default().to_os_string();
    name.push(".dSYM");
    binary.with_file_name(name)
}

fn resign_macho(binary: &Path) -> Result<()> {
    if !cfg!(target_os = "macos") {
        bail!("Mach-O DWARF embed requires codesign on macOS after rewrite");
    }
    println!("==> codesign ad-hoc {}", binary.display());
    let status = Command::new("codesign")
        .args(["-s", "-", "-f"])
        .arg(binary)
        .status()
        .context("start codesign")?;
    ensure!(status.success(), "codesign failed with {status}");
    Ok(())
}

fn embed_dwarf_from_companion(executable: &[u8], companion: &Path) -> Result<Vec<u8>> {
    let companion_bytes = std::fs::read(companion)
        .with_context(|| format!("read dSYM companion {}", companion.display()))?;
    embed_dwarf(executable, &companion_bytes)
}

fn embed_dwarf(executable: &[u8], companion: &[u8]) -> Result<Vec<u8>> {
    let header = parse_header(executable)?;
    ensure!(
        header.magic == MH_MAGIC_64,
        "only thin MH_MAGIC_64 Mach-O executables are supported"
    );
    ensure!(
        !header.commands.iter().any(is_dwarf_segment),
        "executable already has a __DWARF segment but no line-table section name the verifier accepts"
    );
    let plan = plan_embed(&header, executable, companion)?;
    let (ncmds, cmds_blob) = assemble_load_commands(&header, &plan)?;
    write_embedded_image(executable, &header, &plan, ncmds, &cmds_blob)
}

struct EmbedPlan {
    le_fileoff: u64,
    le_vmaddr: u64,
    le_bytes: Vec<u8>,
    dwarf_payload: Vec<u8>,
    section_entries: Vec<SectionEntry>,
    dwarf_fileoff: u64,
    new_le_fileoff: u64,
    pad: u64,
    delta: u64,
}

fn plan_embed(header: &Header, executable: &[u8], companion: &[u8]) -> Result<EmbedPlan> {
    let codesig_off = code_signature_offset(header).unwrap_or(executable.len() as u64);
    let linkedit = header
        .commands
        .iter()
        .find(|cmd| cmd.cmd == LC_SEGMENT_64 && segment_name(&cmd.raw) == b"__LINKEDIT")
        .context("Mach-O executable is missing __LINKEDIT")?;
    let (le_vmaddr, _le_vmsize, le_fileoff, _le_filesize) = read_segment_geometry(&linkedit.raw);
    ensure!(
        codesig_off >= le_fileoff,
        "code signature offset is before __LINKEDIT"
    );
    let codesig_end = usize_off(codesig_off, "code signature offset")?;
    ensure!(
        codesig_end <= executable.len(),
        "code signature offset is past EOF"
    );

    let dwarf_sections = dwarf_sections_from_companion(companion)?;
    ensure!(
        !dwarf_sections.is_empty(),
        "dSYM companion has no __DWARF sections"
    );
    ensure!(
        dwarf_sections
            .iter()
            .any(|section| section.name == "__debug_line" || section.name == "__zdebug_line"),
        "dSYM companion is missing __debug_line"
    );

    let (dwarf_payload, section_entries) = pack_dwarf_payload(&dwarf_sections);
    let dwarf_len = dwarf_payload.len() as u64;
    let dwarf_fileoff = le_fileoff;
    let mut new_le_fileoff = dwarf_fileoff + dwarf_len;
    let pad = (PAGE - (new_le_fileoff % PAGE)) % PAGE;
    new_le_fileoff += pad;
    let delta = new_le_fileoff - le_fileoff;
    let le_start = usize_off(le_fileoff, "__LINKEDIT fileoff")?;
    let le_bytes = executable[le_start..codesig_end].to_vec();

    Ok(EmbedPlan {
        le_fileoff,
        le_vmaddr,
        le_bytes,
        dwarf_payload,
        section_entries,
        dwarf_fileoff,
        new_le_fileoff,
        pad,
        delta,
    })
}

fn pack_dwarf_payload(sections: &[DwarfSection]) -> (Vec<u8>, Vec<SectionEntry>) {
    let mut dwarf_payload = Vec::new();
    let mut section_entries = Vec::with_capacity(sections.len());
    for section in sections {
        while !dwarf_payload.len().is_multiple_of(8) {
            dwarf_payload.push(0);
        }
        section_entries.push(SectionEntry {
            name: section.name.clone(),
            size: section.data.len() as u64,
            relative_off: dwarf_payload.len() as u64,
            flags: section.flags | S_ATTR_DEBUG,
        });
        dwarf_payload.extend_from_slice(&section.data);
    }
    while !dwarf_payload.len().is_multiple_of(16) {
        dwarf_payload.push(0);
    }
    (dwarf_payload, section_entries)
}

fn assemble_load_commands(header: &Header, plan: &EmbedPlan) -> Result<(u32, Vec<u8>)> {
    let dwarf_cmd = build_dwarf_segment_command(
        plan.le_vmaddr,
        plan.dwarf_fileoff,
        plan.dwarf_payload.len() as u64,
        &plan.section_entries,
    )?;
    let mut new_commands = Vec::with_capacity(header.commands.len() + 1);
    let mut inserted = false;
    for cmd in &header.commands {
        if cmd.cmd == LC_CODE_SIGNATURE {
            continue;
        }
        if cmd.cmd == LC_SEGMENT_64 && segment_name(&cmd.raw) == b"__LINKEDIT" {
            new_commands.push(dwarf_cmd.clone());
            new_commands.push(rewritten_linkedit(&cmd.raw, plan)?);
            inserted = true;
            continue;
        }
        if is_dwarf_segment(cmd) {
            continue;
        }
        new_commands.push(relocate_command(cmd, plan.delta)?);
    }
    ensure!(inserted, "failed to insert __DWARF before __LINKEDIT");

    let ncmds = u32::try_from(new_commands.len()).context("load command count")?;
    let cmds_blob = new_commands.iter().flatten().copied().collect::<Vec<_>>();
    let new_sizeofcmds = u32::try_from(cmds_blob.len()).context("load command size")?;
    let first_section_off = first_nondebug_section_fileoff(&new_commands)?
        .context("Mach-O has no file-backed sections")?;
    ensure!(
        u64::from(32 + new_sizeofcmds) <= first_section_off,
        "Mach-O has insufficient load-command padding to embed DWARF \
         (need {}, first section at {first_section_off}); {HEADERPAD_HINT}",
        32 + new_sizeofcmds
    );
    Ok((ncmds, cmds_blob))
}

fn rewritten_linkedit(raw: &[u8], plan: &EmbedPlan) -> Result<Vec<u8>> {
    let mut out = raw.to_vec();
    let new_fs = plan.le_bytes.len() as u64;
    write_u64(&mut out, 32, new_fs)?;
    write_u64(&mut out, 40, plan.new_le_fileoff)?;
    write_u64(&mut out, 48, new_fs)?;
    Ok(out)
}

fn relocate_command(cmd: &LoadCommand, delta: u64) -> Result<Vec<u8>> {
    let mut raw = cmd.raw.clone();
    match cmd.cmd {
        LC_DYLD_INFO_ONLY => {
            for field in [8usize, 16, 24, 32, 40] {
                bump_u32_if_nonzero(&mut raw, field, delta)?;
            }
        }
        LC_SYMTAB => {
            bump_u32_if_nonzero(&mut raw, 8, delta)?;
            bump_u32_if_nonzero(&mut raw, 16, delta)?;
        }
        LC_DYSYMTAB => {
            for field in [32usize, 40, 48, 56, 64, 72] {
                bump_u32_if_nonzero(&mut raw, field, delta)?;
            }
        }
        LC_FUNCTION_STARTS | LC_DATA_IN_CODE | LC_DYLD_EXPORTS_TRIE | LC_DYLD_CHAINED_FIXUPS => {
            bump_u32_if_nonzero(&mut raw, 8, delta)?;
        }
        _ => {}
    }
    Ok(raw)
}

fn write_embedded_image(
    executable: &[u8],
    header: &Header,
    plan: &EmbedPlan,
    ncmds: u32,
    cmds_blob: &[u8],
) -> Result<Vec<u8>> {
    let new_sizeofcmds = u32::try_from(cmds_blob.len()).context("load command size")?;
    let prefix_end = usize_off(plan.le_fileoff, "__LINKEDIT fileoff")?;
    let mut out = executable[..prefix_end].to_vec();
    write_mach_header(
        &mut out,
        &MachHeaderFields {
            cputype: header.cputype,
            cpusubtype: header.cpusubtype,
            filetype: header.filetype,
            ncmds,
            sizeofcmds: new_sizeofcmds,
            flags: header.flags,
            reserved: header.reserved,
        },
    )?;
    let old_cmd_end = 32 + header.sizeofcmds as usize;
    ensure!(old_cmd_end <= out.len(), "load commands overrun prefix");
    out[32..old_cmd_end].fill(0);
    ensure!(
        32 + cmds_blob.len() <= out.len(),
        "expanded load commands overrun prefix"
    );
    out[32..32 + cmds_blob.len()].copy_from_slice(cmds_blob);
    out.extend_from_slice(&plan.dwarf_payload);
    out.extend(std::iter::repeat_n(0u8, usize_off(plan.pad, "DWARF pad")?));
    ensure!(
        out.len() as u64 == plan.new_le_fileoff,
        "internal layout error before __LINKEDIT"
    );
    out.extend_from_slice(&plan.le_bytes);
    Ok(out)
}

fn is_dwarf_segment(cmd: &LoadCommand) -> bool {
    cmd.cmd == LC_SEGMENT_64 && segment_name(&cmd.raw) == b"__DWARF"
}

fn code_signature_offset(header: &Header) -> Option<u64> {
    header
        .commands
        .iter()
        .find(|cmd| cmd.cmd == LC_CODE_SIGNATURE)
        .map(|cmd| u64::from(read_u32(&cmd.raw, 8)))
}

fn usize_off(value: u64, label: &str) -> Result<usize> {
    usize::try_from(value).with_context(|| format!("{label} does not fit usize"))
}

struct Header {
    magic: u32,
    cputype: u32,
    cpusubtype: u32,
    filetype: u32,
    sizeofcmds: u32,
    flags: u32,
    reserved: u32,
    commands: Vec<LoadCommand>,
}

struct LoadCommand {
    cmd: u32,
    raw: Vec<u8>,
}

struct DwarfSection {
    name: String,
    data: Vec<u8>,
    flags: u32,
}

struct SectionEntry {
    name: String,
    size: u64,
    relative_off: u64,
    flags: u32,
}

struct MachHeaderFields {
    cputype: u32,
    cpusubtype: u32,
    filetype: u32,
    ncmds: u32,
    sizeofcmds: u32,
    flags: u32,
    reserved: u32,
}

fn parse_header(data: &[u8]) -> Result<Header> {
    ensure!(data.len() >= 32, "Mach-O header truncated");
    let magic = read_u32(data, 0);
    ensure!(
        magic == MH_MAGIC_64,
        "not a 64-bit Mach-O (magic {magic:#x})"
    );
    let cputype = read_u32(data, 4);
    let cpusubtype = read_u32(data, 8);
    let filetype = read_u32(data, 12);
    let ncmds = read_u32(data, 16);
    let sizeofcmds = read_u32(data, 20);
    let flags = read_u32(data, 24);
    let reserved = read_u32(data, 28);
    ensure!(
        data.len() >= 32 + sizeofcmds as usize,
        "Mach-O load commands truncated"
    );
    let mut commands = Vec::with_capacity(ncmds as usize);
    let mut off = 32usize;
    for _ in 0..ncmds {
        ensure!(off + 8 <= data.len(), "load command header truncated");
        let cmd = read_u32(data, off);
        let cmdsize = read_u32(data, off + 4) as usize;
        ensure!(cmdsize >= 8, "load command size too small");
        ensure!(off + cmdsize <= data.len(), "load command body truncated");
        commands.push(LoadCommand {
            cmd,
            raw: data[off..off + cmdsize].to_vec(),
        });
        off += cmdsize;
    }
    Ok(Header {
        magic,
        cputype,
        cpusubtype,
        filetype,
        sizeofcmds,
        flags,
        reserved,
        commands,
    })
}

fn dwarf_sections_from_companion(companion: &[u8]) -> Result<Vec<DwarfSection>> {
    let header = parse_header(companion)?;
    let mut sections = Vec::new();
    for cmd in &header.commands {
        if !is_dwarf_segment(cmd) {
            continue;
        }
        let nsects = read_u32(&cmd.raw, 64) as usize;
        let mut so = 72usize;
        for _ in 0..nsects {
            ensure!(so + 80 <= cmd.raw.len(), "section header truncated");
            let name = read_cname(&cmd.raw[so..so + 16]);
            let size = read_u64(&cmd.raw, so + 40);
            let fileoff = read_u32(&cmd.raw, so + 48) as usize;
            let flags = read_u32(&cmd.raw, so + 64);
            let end = fileoff
                .checked_add(usize_off(size, "section size")?)
                .context("section range overflow")?;
            ensure!(
                end <= companion.len(),
                "section {name} exceeds companion EOF"
            );
            sections.push(DwarfSection {
                name,
                data: companion[fileoff..end].to_vec(),
                flags,
            });
            so += 80;
        }
    }
    Ok(sections)
}

fn build_dwarf_segment_command(
    vmaddr: u64,
    fileoff: u64,
    filesize: u64,
    sections: &[SectionEntry],
) -> Result<Vec<u8>> {
    let nsects = u32::try_from(sections.len()).context("section count")?;
    let cmdsize = 72 + 80 * sections.len();
    let mut raw = vec![0u8; cmdsize];
    write_u32(&mut raw, 0, LC_SEGMENT_64)?;
    write_u32(&mut raw, 4, u32::try_from(cmdsize).context("cmdsize")?)?;
    write_cname(&mut raw[8..24], b"__DWARF");
    write_u64(&mut raw, 24, vmaddr)?;
    write_u64(&mut raw, 32, 0)?;
    write_u64(&mut raw, 40, fileoff)?;
    write_u64(&mut raw, 48, filesize)?;
    write_u32(&mut raw, 56, 0)?;
    write_u32(&mut raw, 60, 0)?;
    write_u32(&mut raw, 64, nsects)?;
    write_u32(&mut raw, 68, 0)?;
    let mut so = 72usize;
    for section in sections {
        write_cname(&mut raw[so..so + 16], section.name.as_bytes());
        write_cname(&mut raw[so + 16..so + 32], b"__DWARF");
        write_u64(&mut raw, so + 32, 0)?;
        write_u64(&mut raw, so + 40, section.size)?;
        write_u32(
            &mut raw,
            so + 48,
            u32::try_from(fileoff + section.relative_off).context("section fileoff")?,
        )?;
        write_u32(&mut raw, so + 52, 0)?;
        write_u32(&mut raw, so + 56, 0)?;
        write_u32(&mut raw, so + 60, 0)?;
        write_u32(&mut raw, so + 64, section.flags)?;
        write_u32(&mut raw, so + 68, 0)?;
        write_u32(&mut raw, so + 72, 0)?;
        write_u32(&mut raw, so + 76, 0)?;
        so += 80;
    }
    Ok(raw)
}

fn first_nondebug_section_fileoff(commands: &[Vec<u8>]) -> Result<Option<u64>> {
    let mut min = None;
    for raw in commands {
        if read_u32(raw, 0) != LC_SEGMENT_64 || segment_name(raw) == b"__DWARF" {
            continue;
        }
        let nsects = read_u32(raw, 64) as usize;
        let mut so = 72usize;
        for _ in 0..nsects {
            ensure!(so + 80 <= raw.len(), "section header truncated");
            let size = read_u64(raw, so + 40);
            let fileoff = u64::from(read_u32(raw, so + 48));
            let flags = read_u32(raw, so + 64);
            if size > 0 && fileoff > 0 && flags & S_ATTR_DEBUG == 0 {
                min = Some(min.map_or(fileoff, |value: u64| value.min(fileoff)));
            }
            so += 80;
        }
    }
    Ok(min)
}

fn write_mach_header(out: &mut [u8], fields: &MachHeaderFields) -> Result<()> {
    ensure!(out.len() >= 32, "output too small for Mach-O header");
    write_u32(out, 0, MH_MAGIC_64)?;
    write_u32(out, 4, fields.cputype)?;
    write_u32(out, 8, fields.cpusubtype)?;
    write_u32(out, 12, fields.filetype)?;
    write_u32(out, 16, fields.ncmds)?;
    write_u32(out, 20, fields.sizeofcmds)?;
    write_u32(out, 24, fields.flags)?;
    write_u32(out, 28, fields.reserved)?;
    Ok(())
}

fn segment_name(raw: &[u8]) -> &[u8] {
    let end = raw
        .get(8..24)
        .map(|bytes| bytes.iter().position(|&b| b == 0).unwrap_or(16))
        .unwrap_or(0);
    raw.get(8..8 + end).unwrap_or(&[])
}

fn read_segment_geometry(raw: &[u8]) -> (u64, u64, u64, u64) {
    (
        read_u64(raw, 24),
        read_u64(raw, 32),
        read_u64(raw, 40),
        read_u64(raw, 48),
    )
}

fn read_cname(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

fn write_cname(dest: &mut [u8], name: &[u8]) {
    dest.fill(0);
    let n = name.len().min(dest.len());
    dest[..n].copy_from_slice(&name[..n]);
}

fn bump_u32_if_nonzero(raw: &mut [u8], off: usize, delta: u64) -> Result<()> {
    let old = u64::from(read_u32(raw, off));
    if old == 0 {
        return Ok(());
    }
    let new = old
        .checked_add(delta)
        .context("load-command offset overflow")?;
    write_u32(raw, off, u32::try_from(new).context("load-command offset")?)
}

fn read_u32(data: &[u8], off: usize) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&data[off..off + 4]);
    u32::from_le_bytes(bytes)
}

fn read_u64(data: &[u8], off: usize) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[off..off + 8]);
    u64::from_le_bytes(bytes)
}

fn write_u32(data: &mut [u8], off: usize, value: u32) -> Result<()> {
    ensure!(off + 4 <= data.len(), "u32 write past end");
    data[off..off + 4].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn write_u64(data: &mut [u8], off: usize, value: u64) -> Result<()> {
    ensure!(off + 8 <= data.len(), "u64 write past end");
    data[off..off + 8].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn rejects_non_macho_bytes() {
        let short = embed_dwarf(b"not-macho", b"also-not").unwrap_err();
        assert!(
            short.to_string().contains("Mach-O header truncated"),
            "{short}"
        );
        let mut fake = vec![0u8; 32];
        fake[0..4].copy_from_slice(&0xdead_beef_u32.to_le_bytes());
        let wrong_magic = embed_dwarf(&fake, &fake).unwrap_err();
        assert!(
            wrong_magic.to_string().contains("not a 64-bit Mach-O"),
            "{wrong_magic}"
        );
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn embeds_dsym_line_tables_into_apple_release_binary() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let project = temp.path().join("spike");
        std::fs::create_dir_all(project.join("src"))?;
        std::fs::create_dir_all(project.join(".cargo"))?;
        std::fs::write(
            project.join("Cargo.toml"),
            r#"[package]
name = "spike"
version = "0.1.0"
edition = "2021"

[profile.release]
debug = "line-tables-only"
strip = "none"
split-debuginfo = "packed"
"#,
        )?;
        std::fs::write(
            project.join(".cargo/config.toml"),
            r#"[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-Wl,-headerpad,0x10000"]
"#,
        )?;
        std::fs::write(
            project.join("src/main.rs"),
            r#"fn main() {
    println!("hello {}", line!());
}
#[used]
#[no_mangle]
static PARALLAX_RELEASE_IDENTITY: &[u8] = b"parallax-release-identity:0.1.0-spike+deadbeef";
"#,
        )?;
        let status = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(&project)
            .status()
            .context("cargo build spike")?;
        ensure!(status.success(), "cargo build failed: {status}");
        let binary = project.join("target/release/spike");
        let before = std::fs::read(&binary)?;
        ensure!(
            !has_line_table_section(&before)?,
            "precondition: linked Apple binary should lack __debug_line"
        );
        ensure_line_tables(&binary)?;
        let after = std::fs::read(&binary)?;
        ensure!(
            has_line_table_section(&after)?,
            "embedded binary must expose __debug_line"
        );
        super::super::verify::verify_object(
            &after,
            "aarch64-apple-darwin",
            "0.1.0-spike+deadbeef",
        )?;
        let run = Command::new(&binary)
            .output()
            .context("run embedded spike")?;
        ensure!(
            run.status.success(),
            "embedded spike failed to execute: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        Ok(())
    }
}
