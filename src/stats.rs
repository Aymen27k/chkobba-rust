use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Write, Read};

#[derive(Serialize, Deserialize, Debug)]
pub struct GameStats {
    pub total_games: u32,
    pub player_wins: u32,
    pub cpu_wins: u32,
}

impl GameStats {
    // Initialize empty stats if no file exists
    fn new() -> Self {
        GameStats {
            total_games: 0,
            player_wins: 0,
            cpu_wins: 0,
        }
    }
    pub fn win_percentage(&self) -> f32 {
        let decisive_games = self.player_wins + self.cpu_wins;
        if decisive_games == 0 {
            0.0
        } else {
            (self.player_wins as f32 / decisive_games as f32) * 100.0
        }
    }
}

// Internal helper to save data (pub so main can call it)
pub fn save_stats(stats: &GameStats) -> std::io::Result<()> {
    let json_text = serde_json::to_string_pretty(stats)?;
    let mut file = File::create("chkobba_stats.json")?;
    file.write_all(json_text.as_bytes())?;
    Ok(())
}

// Internal helper to load data (pub so main can call it)
pub fn load_stats() -> GameStats {
    if let Ok(mut file) = File::open("chkobba_stats.json") {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(stats) = serde_json::from_str(&contents) {
                return stats;
            }
        }
    }
    GameStats::new()
}