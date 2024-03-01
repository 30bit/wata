#[cfg(any(feature = "read", feature = "write"))]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReadConfig {
    pub num_frames: u32,
    pub is_srgb: Option<bool>,
}

#[cfg(feature = "write")]
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, serde::Serialize, serde::Deserialize,
)]
pub struct WriteConfig {
    pub frame_height: u32,
    pub is_srgb: Option<bool>,
}
