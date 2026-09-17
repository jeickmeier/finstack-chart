//! Unicode identities of the reference plotmath names, independent of font lookup.
pub(super) fn symbol(name: &str) -> Option<&'static str> {
    Some(match name {
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ε",
        "zeta" => "ζ",
        "eta" => "η",
        "theta" => "θ",
        "iota" => "ι",
        "kappa" => "κ",
        "lambda" => "λ",
        "mu" => "μ",
        "nu" => "ν",
        "xi" => "ξ",
        "omicron" => "ο",
        "pi" => "π",
        "rho" => "ρ",
        "sigma" => "σ",
        "tau" => "τ",
        "upsilon" => "υ",
        "phi" => "φ",
        "chi" => "χ",
        "psi" => "ψ",
        "omega" => "ω",
        "Alpha" => "Α",
        "Beta" => "Β",
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Epsilon" => "Ε",
        "Zeta" => "Ζ",
        "Eta" => "Η",
        "Theta" => "Θ",
        "Iota" => "Ι",
        "Kappa" => "Κ",
        "Lambda" => "Λ",
        "Mu" => "Μ",
        "Nu" => "Ν",
        "Xi" => "Ξ",
        "Omicron" => "Ο",
        "Pi" => "Π",
        "Rho" => "Ρ",
        "Sigma" => "Σ",
        "Tau" => "Τ",
        "Upsilon" => "Υ",
        "Phi" => "Φ",
        "Chi" => "Χ",
        "Psi" => "Ψ",
        "Omega" => "Ω",
        "theta1" => "ϑ",
        "phi1" => "ϕ",
        "sigma1" => "ς",
        "omega1" => "ϖ",
        "Upsilon1" => "ϒ",
        "aleph" => "ℵ",
        "infinity" => "∞",
        "partialdiff" => "∂",
        "nabla" => "∇",
        "degree" => "°",
        "minute" => "′",
        "second" => "″",
        "cdots" => "⋯",
        "ldots" | "..." => "…",
        "lceil" => "⌈",
        "rceil" => "⌉",
        "lfloor" => "⌊",
        "rfloor" => "⌋",
        "langle" => "⟨",
        "rangle" => "⟩",
        _ => return None,
    })
}
pub(super) fn operator(name: &str) -> &str {
    match name {
        "+" => "+",
        "-" => "−",
        "/" => "/",
        "!" => "!",
        "==" => "=",
        "!=" => "≠",
        "<=" => "≤",
        ">=" => "≥",
        "%+-%" => "±",
        "%/%" => "÷",
        "%*%" => "×",
        "%.%" => "⋅",
        "%~~%" => "≈",
        "%=~%" => "≅",
        "%==%" => "≡",
        "%prop%" => "∝",
        "%~%" => "∼",
        "%subset%" => "⊂",
        "%subseteq%" => "⊆",
        "%notsubset%" => "⊄",
        "%supset%" => "⊃",
        "%supseteq%" => "⊇",
        "%in%" => "∈",
        "%notin%" => "∉",
        "%<->%" => "↔",
        "%->%" => "→",
        "%<-%" => "←",
        "%up%" => "↑",
        "%down%" => "↓",
        "%<=>%" => "⇔",
        "%=>%" => "⇒",
        "%<=%" => "⇐",
        "%dblup%" => "⇑",
        "%dbldown%" => "⇓",
        s => s,
    }
}
/// Adobe Symbol alphabet requested by symbol(), mapped to portable Unicode.
pub(super) fn symbol_text(text: &str) -> String {
    let lower = "αβχδεφγηιϕκλμνοπθρστυϖωξψζ".chars().collect::<Vec<_>>();
    let upper = "ΑΒΧΔΕΦΓΗΙϑΚΛΜΝΟΠΘΡΣΤΥςΩΞΨΖ".chars().collect::<Vec<_>>();
    text.chars()
        .map(|c| {
            if c.is_ascii_lowercase() {
                lower[c as usize - 'a' as usize]
            } else if c.is_ascii_uppercase() {
                upper[c as usize - 'A' as usize]
            } else {
                c
            }
        })
        .collect()
}
