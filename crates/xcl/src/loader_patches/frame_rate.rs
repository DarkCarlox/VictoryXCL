//! Byte-checked load-time frame-rate patches (`XCL_FRAME_INTERVAL_MS`, `XCL_TITLE_FPS`)
//! for `game_scarlett_master.exe` 1.0.2.1 (ported from `cool_fox`).

fn env_var_first(primary: &str, fallback: &str) -> Option<String> {
    std::env::var(primary).ok().or_else(|| std::env::var(fallback).ok())
}

pub fn asked_interval_ms() -> Option<f64> {
    let spec = env_var_first("XCL_FRAME_INTERVAL_MS", "VICTORY_FRAME_INTERVAL_MS")?;
    let ms = parse_interval_ms(&spec);
    if ms.is_none() {
        crate::xlog_warn!("frame rate: XCL_FRAME_INTERVAL_MS={spec:?}: expected ms from 0 (no wait) to 1000");
    }
    ms
}

pub fn parse_interval_ms(spec: &str) -> Option<f64> {
    spec.trim().parse::<f64>().ok().filter(|ms| ms.is_finite() && (0.0..=1000.0).contains(ms))
}

pub fn title_rate(interval_ms: f64, title_fps: Option<&str>) -> Option<u32> {
    match title_fps.map(str::trim) {
        Some("off") => None,
        Some(rate) => rate.parse::<u32>().ok().filter(|r| *r <= 1000),
        None if interval_ms <= 0.0 => Some(0),
        None => Some(((1000.0 / interval_ms).round() as u32).clamp(1, 1000)),
    }
}

pub fn title_frame_us(rate: u32) -> u32 {
    let clamped = if rate == 0 || rate >= 1000 { 1000 } else { rate };
    (1_000_000.0_f32 / clamped as f32) as u32
}

pub fn title_interval_us(rate: u32, display_120: bool) -> u32 {
    const TABLE: [[u32; 5]; 2] = [[16667, 16667, 33333, 50000, 66667], [8333, 8333, 16667, 25000, 33333]];
    let fps = (1_000_000.0_f32 / title_frame_us(rate) as f32) as u32 & 0xffff;
    let display = if display_120 { 120 } else { 60 };
    let index = if fps <= display { (display / fps.max(1)).min(4) } else { 1 };
    TABLE[usize::from(display_120)][index as usize]
}

pub struct Site {
    pub at: usize,
    pub kind: Kind,
    pub bytes: &'static [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Load,
    SimRate,
}

pub const SITES: [Site; 3] = [
    Site {
        at: 0x1e0fc62,
        kind: Kind::Load,
        bytes: &[
            0x8b, 0x87, 0x10, 0x02, 0x00, 0x00, 0xba, 0xe8, 0x03, 0x00, 0x00, 0xc5, 0xfa, 0x10, 0x0d, 0x9f, 0xa1, 0x34, 0x04, 0xb9, 0xe8,
            0x03, 0x00, 0x00, 0x3d, 0xe8, 0x03, 0x00, 0x00, 0x0f, 0x42, 0xd0, 0x85, 0xc0, 0x0f, 0x44, 0xd1, 0xc5, 0xea, 0x2a, 0xc2, 0xc5,
            0xf2, 0x5e, 0xc0, 0xc4, 0xe1, 0xfa, 0x2c, 0xc0, 0x89, 0x05, 0x12, 0x88, 0xae, 0x07,
        ],
    },
    Site {
        at: 0x224359a,
        kind: Kind::Load,
        bytes: &[
            0x8b, 0x86, 0x10, 0x02, 0x00, 0x00, 0xba, 0xe8, 0x03, 0x00, 0x00, 0xc5, 0xfa, 0x10, 0x0d, 0x67, 0x68, 0xf1, 0x03, 0xb9, 0xe8,
            0x03, 0x00, 0x00, 0x3d, 0xe8, 0x03, 0x00, 0x00, 0x0f, 0x42, 0xd0, 0x85, 0xc0, 0x0f, 0x44, 0xd1, 0xb1, 0xff, 0xc5, 0xea, 0x2a,
            0xc2, 0xc5, 0xf2, 0x5e, 0xc0, 0xc4, 0xe1, 0xfa, 0x2c, 0xc0, 0x89, 0x05, 0xd8, 0x4e, 0x6b, 0x07,
        ],
    },
    Site {
        at: 0x2241fdb,
        kind: Kind::SimRate,
        bytes: &[
            0x83, 0xb8, 0x10, 0x02, 0x00, 0x00, 0x00, 0x74, 0x12, 0xe8, 0xc7, 0xa1, 0xe0, 0x02, 0x8b, 0x80, 0x10, 0x02, 0x00, 0x00, 0xc4,
            0xe1, 0xfa, 0x2a, 0xc0,
        ],
    },
];

pub const FRAME_US: usize = 0x98f84ac;

pub fn replacement(kind: Kind, rate: u32) -> Vec<u8> {
    match kind {
        Kind::Load => {
            let r = rate.to_le_bytes();
            vec![0xb8, r[0], r[1], r[2], r[3], 0x90]
        }
        Kind::SimRate => {
            let r = (if rate == 0 { 1000 } else { rate }).to_le_bytes();
            vec![0xb8, r[0], r[1], r[2], r[3], 0xeb, 0x0d]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Patched {
    Done,
    Differs,
    Failed(&'static str),
}

pub unsafe fn apply(base: usize, size: usize, rate: u32, sites: &[Site]) -> Vec<Patched> {
    use windows_sys::Win32::System::Diagnostics::Debug::FlushInstructionCache;
    use windows_sys::Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    sites
        .iter()
        .map(|site| {
            let code = replacement(site.kind, rate);
            let len = code.len();
            let done = (|| {
                if site.bytes.len() < len || site.at.checked_add(site.bytes.len()).is_none_or(|end| end > size) {
                    return Patched::Failed("outside the image");
                }
                let at = base + site.at;
                let there = std::slice::from_raw_parts(at as *const u8, site.bytes.len());
                if there != site.bytes {
                    return Patched::Differs;
                }
                let mut old = 0;
                if VirtualProtect(at as *const _, len, PAGE_EXECUTE_READWRITE, &mut old) == 0 {
                    return Patched::Failed("Windows would not make it writable");
                }
                std::ptr::copy_nonoverlapping(code.as_ptr(), at as *mut u8, len);
                VirtualProtect(at as *const _, len, old, &mut old);
                FlushInstructionCache(GetCurrentProcess(), at as *const _, len);
                Patched::Done
            })();
            match done {
                Patched::Done => crate::xlog_info!("frame rate: site {:#x} ({:?}) patched to {rate}", site.at, site.kind),
                Patched::Differs => crate::xlog_warn!("frame rate: site {:#x} left untouched (bytes differ)", site.at),
                Patched::Failed(why) => crate::xlog_warn!("frame rate: site {:#x} not patched ({why})", site.at),
            }
            done
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intervals_and_rates() {
        assert_eq!(parse_interval_ms("16.667"), Some(16.667));
        assert_eq!(parse_interval_ms(" 0 "), Some(0.0));
        assert_eq!(parse_interval_ms("-1"), None);
        assert_eq!(parse_interval_ms("fast"), None);
        assert_eq!(parse_interval_ms("NaN"), None);
        assert_eq!(parse_interval_ms("1001"), None);
        assert_eq!(title_rate(16.667, None), Some(60));
        assert_eq!(title_rate(8.333, None), Some(120));
        assert_eq!(title_rate(33.333, None), Some(30));
        assert_eq!(title_rate(0.0, None), Some(0));
        assert_eq!(title_rate(0.5, None), Some(1000));
        assert_eq!(title_rate(16.667, Some("off")), None);
        assert_eq!(title_rate(0.0, Some("90")), Some(90));
        assert_eq!(title_rate(0.0, Some("lots")), None);
    }

    #[test]
    fn the_titles_intervals() {
        assert_eq!(title_frame_us(30), 33333);
        assert_eq!(title_frame_us(60), 16666);
        assert_eq!(title_frame_us(0), 1000);
        assert_eq!(title_frame_us(5000), 1000);
        assert_eq!(title_interval_us(30, false), 33333);
        assert_eq!(title_interval_us(30, true), 33333);
        assert_eq!(title_interval_us(40, true), 25000);
        assert_eq!(title_interval_us(60, false), 16667);
        assert_eq!(title_interval_us(120, true), 8333);
        assert_eq!(title_interval_us(120, false), 16667);
        assert_eq!(title_interval_us(0, false), 16667);
        assert_eq!(title_interval_us(0, true), 8333);
        assert_eq!(title_interval_us(20, false), 50000);
    }

    #[test]
    fn the_sites_are_reads_of_the_setting() {
        for site in SITES.iter().filter(|s| s.kind == Kind::Load) {
            assert_eq!(site.bytes[0], 0x8b);
            assert!(matches!(site.bytes[1], 0x86 | 0x87));
            assert_eq!(&site.bytes[2..6], &[0x10, 0x02, 0x00, 0x00]);
            assert_eq!(&site.bytes[6..11], &[0xba, 0xe8, 0x03, 0x00, 0x00]);
            assert_eq!(&site.bytes[site.bytes.len() - 6..site.bytes.len() - 4], &[0x89, 0x05]);
            let disp = i32::from_le_bytes(site.bytes[site.bytes.len() - 4..].try_into().unwrap());
            let end = site.at + site.bytes.len();
            assert_eq!(end as i64 + i64::from(disp), FRAME_US as i64);
        }
        let sim = SITES.iter().find(|s| s.kind == Kind::SimRate).unwrap();
        assert_eq!(&sim.bytes[..7], &[0x83, 0xb8, 0x10, 0x02, 0x00, 0x00, 0x00]);
        assert_eq!(&sim.bytes[14..20], &[0x8b, 0x80, 0x10, 0x02, 0x00, 0x00]);
        assert_eq!(&sim.bytes[20..], &[0xc4, 0xe1, 0xfa, 0x2a, 0xc0]);
        let code = replacement(Kind::SimRate, 60);
        assert_eq!(code.len() + usize::from(code[6]), 20, "the jump lands on the vcvtsi2ss");
        assert_eq!(replacement(Kind::Load, 60), [0xb8, 60, 0, 0, 0, 0x90]);
        assert_eq!(replacement(Kind::SimRate, 0)[1..5], 1000u32.to_le_bytes());
    }

    #[test]
    fn sites_take_the_rate_and_others_are_left_alone() {
        use windows_sys::Win32::System::Memory::{VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
        unsafe {
            let place = VirtualAlloc(std::ptr::null(), 0x10000, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) as *mut u8;
            assert!(!place.is_null());
            let bytes = SITES[0].bytes;
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), place.add(0x100), bytes.len());
            let mut other = SITES[1].bytes.to_vec();
            other[2] = 0x14;
            std::ptr::copy_nonoverlapping(other.as_ptr(), place.add(0x200), other.len());
            let settings = [30u32; 0x100];
            let sim = SITES[2].bytes;
            let mut f = vec![0x48, 0xb8];
            f.extend((settings.as_ptr() as u64).to_le_bytes());
            let window_at = f.len();
            f.extend(sim);
            f.push(0xc3);
            std::ptr::copy_nonoverlapping(f.as_ptr(), place.add(0x300), f.len());
            let mine = [
                Site { at: 0x100, kind: Kind::Load, bytes },
                Site { at: 0x200, kind: Kind::Load, bytes: SITES[1].bytes },
                Site { at: 0x300 + window_at, kind: Kind::SimRate, bytes: sim },
                Site { at: 0xfff0, kind: Kind::Load, bytes },
            ];
            let done = apply(place as usize, 0x10000, 60, &mine);
            assert_eq!(done[..3], [Patched::Done, Patched::Differs, Patched::Done]);
            assert!(matches!(done[3], Patched::Failed(_)));
            let after = std::slice::from_raw_parts(place.add(0x100), bytes.len());
            assert_eq!(after[..6], replacement(Kind::Load, 60)[..]);
            assert_eq!(after[6..], bytes[6..]);
            assert_eq!(std::slice::from_raw_parts(place.add(0x200), other.len()), &other[..]);
            let rate: extern "system" fn() -> f32 = std::mem::transmute(place.add(0x300));
            assert_eq!(rate(), 60.0);
            VirtualFree(place.cast(), 0, MEM_RELEASE);
        }
    }
}
