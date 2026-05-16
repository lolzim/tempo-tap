pub fn detect_lang() -> &'static str {
    // Priority by POSIX standard
    let lang = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();

    // "fr_CH.UTF-8" -> take only two first chars
    if lang.starts_with("fr") {
        "fr"
    } else if lang.starts_with("de") {
        "de"
    } else {
        "en"
    }
}

pub fn t(lang: &'static str, key: &'static str) -> &'static str {
    match (lang, key) {
        ("fr", "tap_hint") => "Appuyez sur n'importe quelle touche...",
        ("de", "tap_hint") => "Beliebige Taste drücken...",
        (_,    "tap_hint") => "Press any key...",

        ("fr", "tap_hint_too") => "Appuyez sur n'importe quelle touche pour taper le tempo.",
        ("de", "tap_hint_too") => "Beliebige Taste drücken zum Tempo sehen",
        (_,    "tap_hint_too") => "Press any key to get tempo",

        ("fr", "quit_hint") => "Ctrl+C ou 'q' pour quitter.",
        ("de", "quit_hint") => "Ctrl+C oder 'q' zum beenden.",
        (_,    "quit_hint") => "Ctrl+C or 'q' to quit.",

        ("fr", "note_noire") => "Noire",
        ("de", "note_noire") => "Viertelnote",
        (_,    "note_noire") => "Quarter",

        ("fr", "note_croche") => "Croche",
        ("de", "note_croche") => "Achtelnote",
        (_,    "note_croche") => "Eighth",

        ("fr", "note_double") => "Double croche",
        ("de", "note_double") => "Sechzehntelnote",
        (_,    "note_double") => "Sixteenth",

        ("fr", "appuis") => "appuis",
        ("de", "appuis") => "Tastendrücke",
        (_,    "appuis") => "keystrockes",
        _ => key, // fallback
    }
}
