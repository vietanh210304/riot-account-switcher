//! Danh sách avatar champion/agent ngoại tuyến (12 LoL + 12 Valorant).

pub struct AvatarDef {
    pub id: &'static str,
    pub name: &'static str,
    pub game: &'static str,
}

pub static AVATARS: [AvatarDef; 24] = [
    AvatarDef { id: "jinx", name: "Jinx", game: "LoL" },
    AvatarDef { id: "yasuo", name: "Yasuo", game: "LoL" },
    AvatarDef { id: "ahri", name: "Ahri", game: "LoL" },
    AvatarDef { id: "leesin", name: "Lee Sin", game: "LoL" },
    AvatarDef { id: "zed", name: "Zed", game: "LoL" },
    AvatarDef { id: "viego", name: "Viego", game: "LoL" },
    AvatarDef { id: "aatrox", name: "Aatrox", game: "LoL" },
    AvatarDef { id: "akali", name: "Akali", game: "LoL" },
    AvatarDef { id: "yone", name: "Yone", game: "LoL" },
    AvatarDef { id: "thresh", name: "Thresh", game: "LoL" },
    AvatarDef { id: "lux", name: "Lux", game: "LoL" },
    AvatarDef { id: "kaisa", name: "Kai'Sa", game: "LoL" },
    AvatarDef { id: "jett", name: "Jett", game: "VAL" },
    AvatarDef { id: "reyna", name: "Reyna", game: "VAL" },
    AvatarDef { id: "omen", name: "Omen", game: "VAL" },
    AvatarDef { id: "chamber", name: "Chamber", game: "VAL" },
    AvatarDef { id: "iso", name: "Iso", game: "VAL" },
    AvatarDef { id: "clove", name: "Clove", game: "VAL" },
    AvatarDef { id: "raze", name: "Raze", game: "VAL" },
    AvatarDef { id: "viper", name: "Viper", game: "VAL" },
    AvatarDef { id: "sage", name: "Sage", game: "VAL" },
    AvatarDef { id: "phoenix", name: "Phoenix", game: "VAL" },
    AvatarDef { id: "killjoy", name: "Killjoy", game: "VAL" },
    AvatarDef { id: "cypher", name: "Cypher", game: "VAL" },
];

pub fn avatar_path(id: &str) -> std::path::PathBuf {
    crate::assets_dir().join("avatars").join(format!("{id}.png"))
}

pub fn find(id: &str) -> Option<&'static AvatarDef> {
    AVATARS.iter().find(|a| a.id == id)
}
