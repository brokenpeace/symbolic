use goblin::mach;
use uuid::Uuid;

use symbolic_common::Result;

/// A segment inside a Mach object file containing multiple sections
type MachSegment<'mach, 'data> = &'mach mach::segment::Segment<'data>;

/// A section inside a Mach object file
pub struct MachSection<'data> {
    // The header struct
    pub header: mach::segment::Section,
    // The raw data
    pub data: &'data [u8],
}

/// Locates and reads a segment in a Mach object file
pub fn find_mach_segment<'mach, 'data>(
    mach: &'mach mach::MachO<'data>,
    name: &str,
) -> Option<MachSegment<'mach, 'data>> {
    for segment in &mach.segments {
        if segment.name().map(|seg| seg == name).unwrap_or(false) {
            return Some(segment);
        }
    }

    None
}

/// Checks whether a Mach object file contains a segment
///
/// This is useful to determine whether the object contains certain information
/// without iterating over all section headers and loading their data.
pub fn has_mach_segment(mach: &mach::MachO, name: &str) -> bool {
    find_mach_segment(mach, name).is_some()
}

/// Locates and reads a section in a Mach object file
///
/// Depending on its name, the segment will be loaded from either the `"__TEXT"`
/// or the `"__DWARF"` segment.
pub fn find_mach_section<'data>(
    mach: &mach::MachO<'data>,
    name: &str,
) -> Option<MachSection<'data>> {
    let segment_name = match name {
        "__eh_frame" => "__TEXT",
        _ => "__DWARF",
    };

    let segment = match find_mach_segment(mach, segment_name) {
        Some(segment) => segment,
        None => return None,
    };

    for section in segment {
        if let Ok((header, data)) = section {
            if header.name().map(|sec| sec == name).unwrap_or(false) {
                return Some(MachSection { header, data });
            }
        }
    }

    None
}

/// Checks whether a Mach object file contains a section
///
/// Depending on its name, the section will searched in the `"__TEXT"` or the
/// `"__DWARF"` segment. This is useful to determine whether the object contains
/// certain information without iterating over all section headers and loading
/// their data.
pub fn has_mach_section(mach: &mach::MachO, name: &str) -> bool {
    find_mach_section(mach, name).is_some()
}

/// Resolves the UUID from Mach object load commands
pub fn get_mach_uuid(macho: &mach::MachO) -> Option<Uuid> {
    for cmd in &macho.load_commands {
        if let mach::load_command::CommandVariant::Uuid(ref uuid_cmd) = cmd.command {
            return Uuid::from_bytes(&uuid_cmd.uuid).ok();
        }
    }

    None
}

/// Loads the virtual memory address of this object's __TEXT (code) segment
pub fn get_mach_vmaddr(macho: &mach::MachO) -> Result<u64> {
    for seg in &macho.segments {
        if seg.name()? == "__TEXT" {
            return Ok(seg.vmaddr);
        }
    }

    Ok(0)
}
