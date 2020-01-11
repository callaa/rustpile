use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
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
}
