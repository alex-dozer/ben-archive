/// How hard Lucius should go on this artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuciusLevel {
    /// Quick / cheap sanitization or type check.
    Light,
    /// Heavier static analysis (YARA, strings, section parsing, etc.).
    StaticDeep,
    /// Full dynamic detonation in a sandbox.
    DynamicSandbox,
    /// Maximal everything-on-fire mode.
    HybridFull,
}

/// "What should Lucius do with this field?"
#[derive(Debug, Clone)]
pub struct LuciusFieldSpec {
    /// Name of the field on the struct.
    pub field: &'static str,

    /// Optional logical rule / tag (e.g. "MALDOC", "SUSPICIOUS_ATTACHMENT").
    pub rule: Option<&'static str>,

    /// Analysis intensity profile.
    pub level: LuciusLevel,

    /// What kind of thing we expect: "bin", "text", "url", "archive", etc.
    pub expected_type: &'static str,

    /// Route / queue / channel name on the Lucius side.
    ///
    /// Example: "attachments", "urls", "pcap", "office_docs".
    pub route: &'static str,

    /// Optional human hint.
    pub note: Option<&'static str>,
}

/// Trait implemented by structs that have Lucius annotations.
///
/// The derive macro will fill this with a &'static slice describing
/// which fields should create Lucius work.
pub trait LuciusSpec {
    const LUCIUS_SPECS: &'static [LuciusFieldSpec];

    /// Convenience helper if you ever want a Cow’d view later.
    fn lucius_specs() -> &'static [LuciusFieldSpec] {
        Self::LUCIUS_SPECS
    }
}
