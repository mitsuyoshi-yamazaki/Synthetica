use std::collections::HashMap;
use std::fmt;

pub enum Side {
    Left,
    Right,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Side::Left => "left",
                Side::Right => "right",
            }
        )
    }
}

pub enum PlayerRole {
    Player(Side),
    Observer,
}

impl fmt::Display for PlayerRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PlayerRole::Player(side) => match side {
                    Side::Left => "left".to_string(),
                    Side::Right => "right".to_string(),
                },
                PlayerRole::Observer => "observer".to_string(),
            }
        )
    }
}

pub struct PaddleState {
    pub player: Option<u32>,
    pub position: u32,
}

pub struct ServerGameState {
    pub left_paddle: u32,
    pub right_paddle: u32,
    pub players: HashMap<u32, PlayerRole>,
}

pub struct GameConfig {
    pub width: u32,
    pub height: u32,
    pub pad_height: u32,
}

// pub struct ClientGameState {
//     pub left: u32,
//     pub right: u32,
//     pub role: &'a [PlayerRole],
// }

// impl ClientGameState {
//     pub fn to_json(&self) -> String {
//         format!(
//             "{{\"left\":{},\"right\":\"{}\",\"role\":{}}}",
//             self.left, self.right, self.role,
//         )
//     }
// }
