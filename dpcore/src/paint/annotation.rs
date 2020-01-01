use super::{Color, Rectangle};

/// A floating text box over the image
///
/// These are not strictly part of the canvas pixel data,
/// but they belong to the layerstack.
///
/// The core paint engine has no opinion on how annotations
/// are rasterized. To merge an annotation, it must be converted
/// to a bitmap on the client side, using fonts available there,
/// then merged using the PutImage command.
pub struct Annotation {
    pub id: u16,
    pub text: String,
    pub rect: Rectangle,
    pub background: Color,
    pub protect: bool,
    pub valign: u8,
}
