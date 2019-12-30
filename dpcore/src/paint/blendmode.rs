#[derive(Copy, Clone, Debug)]
pub enum Blendmode {
    Erase = 0,
    Normal,
    Multiply,
    Divide,
    Burn,
    Dodge,
    Darken,
    Lighten,
    Subtract,
    Add,
    Recolor,
    Behind,
    ColorErase,
    Replace = 255,
}

impl Blendmode {
    pub fn id(self) -> u8 {
        (((self as isize) & 0xff00) >> 8) as u8
    }

    pub fn can_decrease_opacity(self) -> bool {
        match self {
            Blendmode::Erase | Blendmode::ColorErase | Blendmode::Replace => true,
            _ => false,
        }
    }

    pub fn can_increase_opacity(self) -> bool {
        match self {
            Blendmode::Normal | Blendmode::Recolor | Blendmode::Behind | Blendmode::Replace => true,
            _ => false,
        }
    }

    //pub fn from_id(id: u8) -> Option<Blendmode> {
    //}
}
