use super::brushes;
use crate::paint::tile::Tile;
use crate::paint::{editlayer, Blendmode, ClassicBrushCache, Color, LayerID, LayerStack, UserID};
use crate::protocol::message::*;

use std::convert::{TryFrom, TryInto};
use tracing::warn;

pub struct CanvasState {
    layerstack: LayerStack,
    brushcache: ClassicBrushCache,
}

impl CanvasState {
    pub fn new() -> CanvasState {
        CanvasState {
            layerstack: LayerStack::new(0, 0),
            brushcache: ClassicBrushCache::new(),
        }
    }

    pub fn layerstack(&self) -> &LayerStack {
        &self.layerstack
    }

    pub fn receive_message(&mut self, msg: &Message) {
        use Body::*;
        match &msg.body {
            DrawDabsClassic(m) => self.handle_drawdabs_classic(msg.user_id, m),
            UndoPoint => self.handle_undopoint(msg.user_id),
            PenUp => self.handle_penup(msg.user_id),

            CanvasResize(m) => self.handle_canvas_resize(m),
            LayerCreate(m) => self.handle_layer_create(m),
            LayerAttributes(m) => self.handle_layer_attributes(m),
            PutTile(m) => self.handle_puttile(msg.user_id, m),
            CanvasBackground(m) => self.handle_background(m),

            Join(_)
            | SessionOwner(_)
            | Chat(_)
            | TrustedUsers(_)
            | PrivateChat(_)
            | Interval(_)
            | LaserTrail(_)
            | MovePointer(_)
            | Marker(_)
            | UserACL(_)
            | LayerACL(_)
            | FeatureAccessLevels(_)
            | Filtered(_) => (), // These messages have no effect on rendering
            _ => todo!("Unhandled message: {}", msg),
        }
    }

    fn handle_undopoint(&mut self, user_id: UserID) {
        // TODO
    }

    fn handle_penup(&mut self, user_id: UserID) {
        // TODO
    }

    fn handle_canvas_resize(&mut self, msg: &CanvasResizeMessage) {
        if let Some(ls) = self
            .layerstack
            .resized(msg.top, msg.right, msg.bottom, msg.left)
        {
            self.layerstack = ls;
        } else {
            warn!("Invalid resize: {:?}", msg);
        }
    }

    fn handle_layer_create(&mut self, msg: &LayerCreateMessage) {
        // TODO placement, clone, name
        self.layerstack
            .add_layer(msg.id as LayerID, &Color::from_argb32(msg.fill));
    }

    fn handle_layer_attributes(&mut self, msg: &LayerAttributesMessage) {
        if let Some(layer) = self.layerstack.get_layer_mut(msg.id as LayerID) {
            editlayer::change_attributes(
                layer,
                msg.sublayer as LayerID,
                msg.opacity as f32 / 255.0,
                Blendmode::try_from(msg.blend).unwrap_or(Blendmode::Normal),
                (msg.flags & LayerAttributesMessage::FLAGS_CENSOR) != 0,
                (msg.flags & LayerAttributesMessage::FLAGS_FIXED) != 0,
            );
        } else {
            warn!("LayerAttributes: Layer {:04x} not found!", msg.id);
        }
    }

    fn handle_puttile(&mut self, user_id: UserID, msg: &PutTileMessage) {
        if let Some(layer) = self.layerstack.get_layer_mut(msg.layer as LayerID) {
            let tile = if msg.image.len() == 4 {
                let color =
                    Color::from_argb32(u32::from_be_bytes(msg.image[..].try_into().unwrap()));
                Tile::new(&color, user_id)
            } else {
                todo!() // DEFLATE
            };

            editlayer::put_tile(
                layer,
                msg.sublayer as LayerID,
                msg.col.into(),
                msg.row.into(),
                msg.repeat.into(),
                &tile,
            );
        } else {
            warn!("PutTile: Layer {:04x} not found!", msg.layer);
        }
    }

    fn handle_background(&mut self, pixels: &Vec<u8>) {
        let tile = if pixels.len() == 4 {
            let color = Color::from_argb32(u32::from_be_bytes(pixels[..].try_into().unwrap()));
            Tile::new(&color, 0)
        } else {
            todo!()
        };

        self.layerstack.background = tile;
    }

    fn handle_drawdabs_classic(&mut self, user: UserID, msg: &DrawDabsClassicMessage) {
        if let Some(layer) = self.layerstack.get_layer_mut(msg.layer as LayerID) {
            brushes::drawdabs_classic(layer, user, &msg, &mut self.brushcache);
        } else {
            warn!("DrawDabsClassic: Layer {:04x} not found!", msg.layer);
        }
    }
}
