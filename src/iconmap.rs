// linear to nerdfont
pub fn ico_to_nf(name: &str) -> String {
    let nf = match name {
        "Umbrella" => "  ",
        "FaceStarEyes" => "  ",
        _ => "idk"
    };
    nf.to_string()
}

pub fn p_to_nf(priority: f64) -> String {
    let nf = match priority {
        1.0 => "",
        2.0 => "󰕾",
        3.0 => "󰖀",
        4.0 => "󰕿",
        _ => "󰸈",
    };
    nf.to_string()
}
