use std::sync::OnceLock;

use anyhow::Result;

static VERSION: OnceLock<u16> = OnceLock::new();

#[inline(always)]
pub fn tef_version() -> Result<u16> {
    let val = VERSION.get_or_init(|| {
        let version = env!("TEF_VERSION")
            .split(".")
            .map(|v| v.parse::<u8>())
            .collect::<Result<Vec<u8>, _>>()
            .expect("Failed to parse TEF_VERSION");

        (version[0] as u16) << 8 | (version[1] as u16)
    });

    Ok(*val)
}
