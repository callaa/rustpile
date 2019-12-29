const PRIVATE: u8 = 0; // this blendmode should not be exposed in the UI
const LAYER: u8 = 0x01; // can be used for layers (pixel blend)
const BRUSH: u8 = 0x02; // can be used for brushes (mask blend)
const ALL: u8 = LAYER | BRUSH;
const DECR: u8 = 0x04; // can decrease opacity
const INCR: u8 = 0x08; // can increase opacity

const fn encode(id: u8, flags: u8) -> isize {
    return (id as isize) << 8 | (flags as isize);
}

#[derive(Copy, Clone, Debug)]
pub enum Blendmode {
    Erase = encode(0, PRIVATE | DECR),
    Normal = encode(1, ALL | INCR),
    Multiply = encode(2, ALL),
    Divide = encode(3, ALL),
    Burn = encode(4, ALL),
    Dodge = encode(5, ALL),
    Darken = encode(6, ALL),
    Lighten = encode(7, ALL),
    Subtract = encode(8, ALL),
    Add = encode(9, ALL),
    Recolor = encode(10, BRUSH),
    Behind = encode(11, BRUSH | INCR),
    ColorErase = encode(12, PRIVATE | DECR),
    Replace = encode(12, PRIVATE | DECR | INCR),
}

impl Blendmode {
    pub fn id(self) -> u8 {
        (((self as isize) & 0xff00) >> 8) as u8
    }

    pub fn can_decrease_opacity(self) -> bool {
        ((self as isize & 0xff) as u8) & DECR != 0
    }

    pub fn can_increase_opacity(self) -> bool {
        ((self as isize & 0xff) as u8) & INCR != 0
    }

    //pub fn from_id(id: u8) -> Option<Blendmode> {
    //}
}
