//! Cardinal points, only integer angles (N, NE, E, ..) are supported

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[repr(u16)]
pub enum Cardinal {
    /// Northern Cardinal
    North = 0,
    /// North Eastern Cardinal
    NorthEast = 45,
    /// Eastern Cardinal
    East = 90,
    /// South Eastern Cardinal
    SouthEast = 135,
    /// Southern Cardinal
    South = 180,
    /// South Western Cardinal
    SouthWest = 225,
    /// Western Cardinal
    West = 270,
    /// North Western Cardinal
    NorthWest = 315,
}

impl Default for Cardinal {
    /// Builds default Northern Cardinal
    fn default() -> Self {
        Self::North
    }
}

impl std::fmt::Display for Cardinal {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Cardinal::North => write!(f, "N"),
            Cardinal::NorthEast => write!(f, "NE"),
            Cardinal::East => write!(f, "E"),
            Cardinal::SouthEast => write!(f, "SE"),
            Cardinal::South => write!(f, "S"),
            Cardinal::SouthWest => write!(f, "SW"),
            Cardinal::West => write!(f, "W"),
            Cardinal::NorthWest => write!(f, "NW"),
        }
    }
}

impl Cardinal {
    /// Returns True if Self matches a latitude cardinal
    pub fn is_latitude (&self) -> bool { 
        match self {
            Cardinal::North | Cardinal::South => true,
            _ => false,
        }
    }
    /// Returns True if Self matches a longitude cardinal
    pub fn is_longitude (&self) -> bool {
        match self {
            Cardinal::East | Cardinal::West => true,
            _ => false,
        }
    }
    /// Returns True if Cardinal and `rhs` represents
    /// same kind of coordinates
    pub fn same_kind (&self, rhs: Self) -> bool {
        (self.is_latitude() && rhs.is_latitude())
        || (self.is_longitude() && rhs.is_longitude())
    }
    /// Returns True if Self is a Northern cardinal 
    pub fn is_northern (&self) -> bool {
        match self {
            Cardinal::North | Cardinal::NorthEast | Cardinal::NorthWest => true,
            _ => false,
        }
    }
    /// Returns True if Self is a Southern cardinal 
    pub fn is_southern (&self) -> bool {
        match self {
            Cardinal::South | Cardinal::SouthEast | Cardinal::SouthWest => true,
            _ => false,
        }
    }
    /// Returns True if Self is an Eastern cardinal 
    pub fn is_eastern (&self) -> bool {
        match self {
            Cardinal::East | Cardinal::NorthEast | Cardinal::SouthEast => true,
            _ => false,
        }
    }
    /// Returns True if Self is a Western cardinal 
    pub fn is_western (&self) -> bool {
        match self {
            Cardinal::West | Cardinal::NorthWest | Cardinal::SouthWest => true,
            _ => false,
        }
    }
}
