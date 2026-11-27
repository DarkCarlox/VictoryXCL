//! Byte-checked load-time output size and render scale patches
//! (`XCL_OUTPUT`, `XCL_RENDER_SCALE`, `XCL_FFX_SCALE`, `XCL_UPSCALER`)
//! for `game_scarlett_master.exe` 1.0.2.1 (ported from `cool_fox`).

use super::frame_rate::Patched;

pub const MODES: [(u32, u32); 5] = [(1280, 720), (1664, 936), (1920, 1080), (2560, 1440), (3840, 2160)];

fn env_var_first(primary: &str, fallback: &str) -> Option<String> {
    std::env::var(primary).ok().or_else(|| std::env::var(fallback).ok())
}

pub fn asked_output() -> Option<(u32, u32)> {
    let spec = env_var_first("XCL_OUTPUT", "VICTORY_OUTPUT")?;
    let size = parse_output(&spec);
    if size.is_none() {
        crate::xlog_warn!("resolution: XCL_OUTPUT={spec:?}: expected 1280x720, 1664x936, 1920x1080, 2560x1440, or 3840x2160");
    }
    size
}

pub fn parse_output(spec: &str) -> Option<(u32, u32)> {
    let (w, h) = spec.trim().split_once(['x', 'X'])?;
    let size = (w.trim().parse().ok()?, h.trim().parse().ok()?);
    MODES.contains(&size).then_some(size)
}

pub fn asked_render_scale() -> Option<f32> {
    let spec = env_var_first("XCL_RENDER_SCALE", "VICTORY_RENDER_SCALE")?;
    let scale = parse_render_scale(&spec);
    if scale.is_none() {
        crate::xlog_warn!("resolution: XCL_RENDER_SCALE={spec:?}: expected 0.5 to 1.0");
    }
    scale
}

pub fn parse_render_scale(spec: &str) -> Option<f32> {
    spec.trim().parse::<f32>().ok().filter(|s| s.is_finite() && (0.5..=1.0).contains(s))
}

pub fn asked_ffx_scale() -> Option<f32> {
    let spec = env_var_first("XCL_FFX_SCALE", "VICTORY_FFX_SCALE")?;
    let scale = parse_render_scale(&spec);
    if scale.is_none() {
        crate::xlog_warn!("resolution: XCL_FFX_SCALE={spec:?}: expected 0.5 to 1.0");
    }
    scale
}

pub fn asked_upscaler() -> Option<u32> {
    let spec = env_var_first("XCL_UPSCALER", "VICTORY_UPSCALER")?;
    let kind = spec.trim().parse::<u32>().ok().filter(|k| matches!(k, 6 | 7));
    if kind.is_none() {
        crate::xlog_warn!("resolution: XCL_UPSCALER={spec:?}: expected 6 (title own) or 7 (FidelityFX)");
    }
    kind
}

pub struct Site {
    pub at: usize,
    pub what: &'static str,
    pub bytes: &'static [u8],
}

pub const SET_SCREEN_SIZE: usize = 0x2478130;
pub const SCREEN_HEIGHT: usize = 0xa9438d0;
pub const HEIGHT_CHANGED: usize = 0xd8c3a0;

pub const OUTPUT: Site = Site {
    at: 0x224352b,
    what: "settings' output size applied",
    bytes: &[
        0x8b, 0x0d, 0x9b, 0x03, 0x70, 0x08, 0x8b, 0x86, 0x04, 0x02, 0x00, 0x00, 0x39, 0xc1, 0x75, 0x0d, 0x8b, 0x96, 0x08, 0x02, 0x00,
        0x00, 0xe8, 0xea, 0x4b, 0x23, 0x00, 0xeb, 0x20, 0x8b, 0xbe, 0x08, 0x02, 0x00, 0x00, 0x8b, 0x2d, 0x7c, 0x03, 0x70, 0x08, 0x89,
        0xc1, 0x89, 0xfa, 0xe8, 0xd3, 0x4b, 0x23, 0x00, 0x39, 0xfd, 0x74, 0x07, 0xb1, 0x01, 0xe8, 0x38, 0x8e, 0xb4, 0xfe,
    ],
};

pub const POOL_CHECK: Site = Site {
    at: 0x247af25,
    what: "back buffer pool's left-over check",
    bytes: &[0x48, 0x3d, 0xff, 0xff, 0x1f, 0x00, 0x76, 0x01, 0xcc],
};

pub fn pool_check_code() -> Vec<u8> {
    let mut c = POOL_CHECK.bytes[..8].to_vec();
    c[6] = 0xeb;
    c
}

pub const OWN_SCALE: Site = Site {
    at: 0x28f1e78,
    what: "title upscaler render scale",
    bytes: &[0xc7, 0x81, 0x90, 0x00, 0x00, 0x00, 0xab, 0xaa, 0x2a, 0x3f],
};

pub const FFX_SCALE: Site = Site {
    at: 0x28ef680,
    what: "FidelityFX render scale",
    bytes: &[
        0x41, 0x56, 0x56, 0x57, 0x53, 0x48, 0x83, 0xec, 0x28, 0x44, 0x89, 0xcb, 0x4c, 0x89, 0xc6, 0x48, 0x89, 0xd7, 0x49, 0x89,
        0xce, 0xe8, 0xa6, 0x5a, 0x78, 0xfd, 0x0f, 0xb6, 0xc0, 0x41, 0x89, 0x86, 0x90, 0x00, 0x00, 0x00, 0x84, 0xc0, 0x75, 0x0a,
        0xc5, 0xfa, 0x10, 0x05, 0x28, 0x78, 0x6d, 0x03, 0xeb, 0x08, 0xc5, 0xfa, 0x10, 0x05, 0xa2, 0xe2, 0x86, 0x03, 0xc5, 0xf2,
        0x2a, 0x54, 0x24, 0x70, 0xc5, 0xf2, 0x2a, 0xcb, 0xc5, 0xfa, 0x59, 0xc9, 0xc5, 0xfa, 0x2c, 0xc1, 0x89, 0x07, 0xc5, 0xfa,
        0x59, 0xc2, 0xc5, 0xfa, 0x2c, 0xc0, 0x89, 0x06, 0x48, 0x83, 0xc4, 0x28, 0x5b, 0x5f, 0x5e, 0x41, 0x5e, 0xc3, 0xcc, 0xcc,
        0xcc, 0xcc, 0xcc, 0xcc,
    ],
};

pub const PICKER: Site = Site {
    at: 0x7e93f0,
    what: "upscaler picker",
    bytes: &[
        0x48, 0x83, 0xec, 0x28, 0x80, 0x3d, 0x6d, 0xf0, 0x3a, 0x07, 0x00, 0x75, 0x09, 0x83, 0x3d, 0x68, 0xf0, 0x3a, 0x07, 0x00,
        0x74, 0x28, 0xe8, 0x35, 0xbd, 0x88, 0xff, 0x8a, 0x0d, 0x5b, 0xf0, 0x3a, 0x07, 0x80, 0xe1, 0x40, 0x80, 0xf9, 0x01, 0xb9,
        0x07, 0x00, 0x00, 0x00, 0x83, 0xd9, 0x00, 0x84, 0xc0, 0xb8, 0x06, 0x00, 0x00, 0x00, 0x0f, 0x45, 0xc1, 0x48, 0x83, 0xc4,
        0x28, 0xc3,
    ],
};

fn rel(to: usize, next: usize) -> [u8; 4] {
    ((to as i64 - next as i64) as i32).to_le_bytes()
}

pub fn output_code(width: u32, height: u32) -> Vec<u8> {
    let at = OUTPUT.at;
    let mut c = Vec::with_capacity(OUTPUT.bytes.len());
    c.push(0xb8);
    c.extend(width.to_le_bytes());
    c.extend([0x89, 0x86, 0x04, 0x02, 0x00, 0x00]);
    c.push(0xbf);
    c.extend(height.to_le_bytes());
    c.extend([0x89, 0xbe, 0x08, 0x02, 0x00, 0x00]);
    c.extend([0x8b, 0x2d]);
    c.extend(rel(SCREEN_HEIGHT, at + c.len() + 4));
    c.extend([0x89, 0xc1, 0x89, 0xfa]);
    c.push(0xe8);
    c.extend(rel(SET_SCREEN_SIZE, at + c.len() + 4));
    let end = OUTPUT.bytes.len();
    c.extend([0x39, 0xfd, 0x74, (end - (c.len() + 4)) as u8]);
    c.extend([0xb1, 0x01, 0xe8]);
    c.extend(rel(HEIGHT_CHANGED, at + c.len() + 4));
    c.extend([0xeb, (end - (c.len() + 2)) as u8]);
    c.resize(end, 0xcc);
    c
}

pub fn own_scale_code(scale: f32) -> Vec<u8> {
    let mut c = OWN_SCALE.bytes[..6].to_vec();
    c.extend(scale.to_le_bytes());
    c
}

pub fn ffx_scale_code(scale: f32) -> Vec<u8> {
    const LOADS: [usize; 2] = [0x28, 0x32];
    const SLOT: usize = 0x64;
    let mut c = FFX_SCALE.bytes.to_vec();
    c[SLOT..SLOT + 4].copy_from_slice(&scale.to_le_bytes());
    for l in LOADS {
        c[l + 4..l + 8].copy_from_slice(&((SLOT - (l + 8)) as i32).to_le_bytes());
    }
    c
}

pub fn picker_code(kind: u32) -> Vec<u8> {
    let mut c = vec![0xb8];
    c.extend(kind.to_le_bytes());
    c.push(0xc3);
    c
}

pub unsafe fn matches(base: usize, size: usize, site: &Site) -> Patched {
    if site.at.checked_add(site.bytes.len()).is_none_or(|end| end > size) {
        return Patched::Failed("outside the image");
    }
    let there = std::slice::from_raw_parts((base + site.at) as *const u8, site.bytes.len());
    if there == site.bytes { Patched::Done } else { Patched::Differs }
}

pub unsafe fn write(base: usize, size: usize, site: &Site, code: &[u8]) -> Patched {
    use windows_sys::Win32::System::Diagnostics::Debug::FlushInstructionCache;
    use windows_sys::Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    if code.len() > site.bytes.len() {
        return Patched::Failed("replacement longer than the bytes compared");
    }
    let found = matches(base, size, site);
    if found != Patched::Done {
        return found;
    }
    let at = base + site.at;
    let mut old = 0;
    if VirtualProtect(at as *const _, code.len(), PAGE_EXECUTE_READWRITE, &mut old) == 0 {
        return Patched::Failed("Windows would not make it writable");
    }
    std::ptr::copy_nonoverlapping(code.as_ptr(), at as *mut u8, code.len());
    VirtualProtect(at as *const _, code.len(), old, &mut old);
    FlushInstructionCache(GetCurrentProcess(), at as *const _, code.len());
    Patched::Done
}

pub unsafe fn apply_asked(base: usize, size: usize) {
    let mut asked: Vec<(&Site, Vec<u8>, String)> = Vec::new();
    if let Some((w, h)) = asked_output() {
        asked.push((&OUTPUT, output_code(w, h), format!("{w}x{h}")));
        if (w, h) != (3840, 2160) {
            asked.push((&POOL_CHECK, pool_check_code(), format!("{w}x{h}")));
        }
    }
    if let Some(s) = asked_render_scale() {
        asked.push((&OWN_SCALE, own_scale_code(s), format!("{s}")));
    }
    if let Some(s) = asked_ffx_scale() {
        asked.push((&FFX_SCALE, ffx_scale_code(s), format!("{s}")));
    }
    if let Some(kind) = asked_upscaler() {
        asked.push((&PICKER, picker_code(kind), format!("returns {kind}")));
    }
    for (site, code, value) in asked {
        let done = write(base, size, site, &code);
        match done {
            Patched::Done => crate::xlog_info!("resolution: {} ({value}) patched at title+{:#x}", site.what, site.at),
            Patched::Differs => crate::xlog_warn!("resolution: {} at title+{:#x} left untouched (bytes differ)", site.what, site.at),
            Patched::Failed(why) => crate::xlog_warn!("resolution: {} at title+{:#x} not patched ({why})", site.what, site.at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switches_parse() {
        assert_eq!(parse_output("1920x1080"), Some((1920, 1080)));
        assert_eq!(parse_output(" 2560X1440 "), Some((2560, 1440)));
        assert_eq!(parse_output("1600x900"), None);
        assert_eq!(parse_output("1080p"), None);
        assert_eq!(parse_render_scale("0.67"), Some(0.67));
        assert_eq!(parse_render_scale("0.5"), Some(0.5));
        assert_eq!(parse_render_scale("1"), Some(1.0));
        assert_eq!(parse_render_scale("0.4"), None);
        assert_eq!(parse_render_scale("1.5"), None);
        assert_eq!(parse_render_scale("NaN"), None);
    }

    #[test]
    fn the_output_block_is_the_titles() {
        let b = OUTPUT.bytes;
        assert_eq!(b.len(), 61);
        let target = |op_end: usize| (OUTPUT.at + op_end) as i64 + i64::from(i32::from_le_bytes(b[op_end - 4..op_end].try_into().unwrap()));
        assert_eq!(target(27), SET_SCREEN_SIZE as i64);
        assert_eq!(target(50), SET_SCREEN_SIZE as i64);
        assert_eq!(target(61), HEIGHT_CHANGED as i64);
        assert_eq!(target(41), SCREEN_HEIGHT as i64);
        assert_eq!(target(6), (SCREEN_HEIGHT - 4) as i64);
        assert_eq!((b[14], 16 + b[15] as usize), (0x75, 29));
        assert_eq!((b[27], 29 + b[28] as usize), (0xeb, 61));
        assert_eq!((b[52], 54 + b[53] as usize), (0x74, 61));
    }

    #[test]
    fn the_output_code_keeps_the_titles_else_path() {
        let c = output_code(1920, 1080);
        assert_eq!(c.len(), OUTPUT.bytes.len());
        assert_eq!(c[0], 0xb8);
        assert_eq!(c[1..5], 1920u32.to_le_bytes());
        assert_eq!(c[5..11], [0x89, 0x86, 0x04, 0x02, 0x00, 0x00]);
        assert_eq!(c[12..16], 1080u32.to_le_bytes());
        assert_eq!(c[16..22], [0x89, 0xbe, 0x08, 0x02, 0x00, 0x00]);
        let target = |op_end: usize| (OUTPUT.at + op_end) as i64 + i64::from(i32::from_le_bytes(c[op_end - 4..op_end].try_into().unwrap()));
        assert_eq!(c[22..24], [0x8b, 0x2d]);
        assert_eq!(target(28), SCREEN_HEIGHT as i64);
        assert_eq!(c[32], 0xe8);
        assert_eq!(target(37), SET_SCREEN_SIZE as i64);
        assert_eq!(c[37..40], [0x39, 0xfd, 0x74]);
        assert_eq!(41 + c[40] as usize, 61);
        assert_eq!(c[43], 0xe8);
        assert_eq!(target(48), HEIGHT_CHANGED as i64);
        assert_eq!(c[48], 0xeb);
        assert_eq!(50 + c[49] as usize, 61);
        assert!(c[50..].iter().all(|&x| x == 0xcc));
    }

    #[test]
    fn the_scale_codes() {
        let own = own_scale_code(0.5);
        assert_eq!(own[..6], OWN_SCALE.bytes[..6]);
        assert_eq!(own[6..], 0.5f32.to_le_bytes());
        assert_eq!(OWN_SCALE.bytes[6..], (2.0f32 / 3.0).to_le_bytes());
        let ffx = ffx_scale_code(0.75);
        assert_eq!(ffx.len(), FFX_SCALE.bytes.len());
        assert_eq!(ffx[0x64..0x68], 0.75f32.to_le_bytes());
        for at in [0x28, 0x32] {
            assert_eq!(ffx[at..at + 4], [0xc5, 0xfa, 0x10, 0x05]);
            let disp = i32::from_le_bytes(ffx[at + 4..at + 8].try_into().unwrap());
            assert_eq!(at as i32 + 8 + disp, 0x64);
        }
        let changed: Vec<usize> = (0..ffx.len()).filter(|&i| ffx[i] != FFX_SCALE.bytes[i]).collect();
        assert!(changed.iter().all(|&i| (0x2c..0x30).contains(&i) || (0x36..0x3a).contains(&i) || (0x64..0x68).contains(&i)));
        assert_eq!(picker_code(7), [0xb8, 7, 0, 0, 0, 0xc3]);
        assert_eq!(pool_check_code(), [0x48, 0x3d, 0xff, 0xff, 0x1f, 0x00, 0xeb, 0x01]);
    }

    #[test]
    fn a_site_is_written_only_over_its_bytes() {
        use windows_sys::Win32::System::Memory::{VirtualAlloc, VirtualFree, VirtualProtect, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READ};
        unsafe {
            let place = VirtualAlloc(std::ptr::null(), 0x1000, MEM_COMMIT | MEM_RESERVE, 0x40) as *mut u8;
            assert!(!place.is_null());
            let site = Site { at: 0x100, what: "test", bytes: OWN_SCALE.bytes };
            std::ptr::copy_nonoverlapping(site.bytes.as_ptr(), place.add(0x100), site.bytes.len());
            let mut old = 0;
            VirtualProtect(place.cast(), 0x1000, PAGE_EXECUTE_READ, &mut old);
            assert_eq!(write(place as usize, 0x1000, &site, &own_scale_code(0.5)), Patched::Done);
            assert_eq!(std::slice::from_raw_parts(place.add(0x106), 4), 0.5f32.to_le_bytes());
            assert_eq!(write(place as usize, 0x1000, &site, &own_scale_code(0.75)), Patched::Differs);
            assert_eq!(std::slice::from_raw_parts(place.add(0x106), 4), 0.5f32.to_le_bytes());
            let far = Site { at: 0xffa, what: "test", bytes: OWN_SCALE.bytes };
            assert!(matches!(write(place as usize, 0x1000, &far, &own_scale_code(0.5)), Patched::Failed(_)));
            VirtualFree(place.cast(), 0, MEM_RELEASE);
        }
    }
}
