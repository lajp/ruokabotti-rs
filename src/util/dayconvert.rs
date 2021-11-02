pub fn num_to_paiva(num: usize) -> Option<String> {
    match num {
        0 => Some("Maanantai".to_string()),
        1 => Some("Tiistai".to_string()),
        2 => Some("Keskiviikko".to_string()),
        3 => Some("Torstai".to_string()),
        4 => Some("Perjantai".to_string()),
        5 => Some("Lauantai".to_string()),
        6 => Some("Sunnuntai".to_string()),
        _ => None,
    }
}
