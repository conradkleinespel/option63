pub fn param_value_needs_quoting(input: &[u8]) -> bool {
    // Input can contain only safe-chars of qsafe-chars, but if we find stuff that's not qsafe,
    // that is ":" (0x3A) or ";" (0x3B), we'll need to quote the value
    input.iter().any(|c| matches!(c, 0x3A | 0x3B))
}
