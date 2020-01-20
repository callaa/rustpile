use super::brushes;
use super::history::History;
use crate::paint::tile::Tile;
use crate::paint::{
    editlayer, Blendmode, ClassicBrushCache, Color, LayerID, LayerStack, Rectangle, UserID,
};
use crate::protocol::message::*;

use std::convert::{TryFrom, TryInto};
use std::rc::Rc;
use tracing::warn;

pub struct CanvasState {
    layerstack: Rc<LayerStack>,
    history: History,
    brushcache: ClassicBrushCache,
}

impl CanvasState {
    pub fn new() -> CanvasState {
        CanvasState {
            layerstack: Rc::new(LayerStack::new(0, 0)),
            history: History::new(),
            brushcache: ClassicBrushCache::new(),
        }
    }

    pub fn layerstack(&self) -> &LayerStack {
        &self.layerstack
    }
    pub fn receive_message(&mut self, msg: &CommandMessage) {
        self.history.add(msg.clone());
        self.handle_message(msg);
    }

    fn handle_message(&mut self, msg: &CommandMessage) {
        use CommandMessage::*;
        match &msg {
            DrawDabsClassic(user, m) => self.handle_drawdabs_classic(*user, m),
            DrawDabsPixel(user, m) => self.handle_drawdabs_pixel(*user, m, false),
            DrawDabsPixelSquare(user, m) => self.handle_drawdabs_pixel(*user, m, true),

            UndoPoint(user) => self.handle_undopoint(*user),
            PenUp(user) => self.handle_penup(*user),
            Undo(user, m) => self.handle_undo(*user, m),

            CanvasResize(_, m) => self.handle_canvas_resize(m),
            LayerCreate(_, m) => self.handle_layer_create(m),
            LayerAttributes(_, m) => self.handle_layer_attributes(m),
            PutTile(user, m) => self.handle_puttile(*user, m),
            CanvasBackground(_, m) => self.handle_background(m),
            FillRect(user, m) => self.handle_fillrect(*user, m),

            _ => todo!("Unhandled message: {}", msg),
        }
    }

    fn handle_undopoint(&mut self, user_id: UserID) {
        self.make_savepoint_if_needed();
        // set "has participated" flag
    }

    fn handle_undo(&mut self, user_id: UserID, msg: &UndoMessage) {
        // Session operators are allowed to undo/redo other users' work
        let user = if msg.override_user > 0 {
            msg.override_user
        } else {
            user_id
        };

        let replay = if msg.redo {
            self.history.redo(user)
        } else {
            self.history.undo(user)
        };

        if let Some((savepoint, messages)) = replay {
            self.layerstack = savepoint;
            for msg in messages {
                self.handle_message(&msg);
            }
        }
    }

    /// Penup does nothing but end indirect strokes.
    /// This is done by merging this user's sublayers.
    fn handle_penup(&mut self, user_id: UserID) {
        let sublayer_id = user_id as LayerID;

        // Note: we could do a read-only pass first to check if
        // this is necesary at all, but we can just as well simply
        // not send unnecessary PenUps.

        // TODO map to AoE
        Rc::make_mut(&mut self.layerstack)
            .iter_layers_mut()
            .filter(|l| l.has_sublayer(sublayer_id)) // avoid unnecessary clones
            .for_each(|l| { editlayer::merge_sublayer(Rc::make_mut(l), sublayer_id); });
    }

    fn handle_canvas_resize(&mut self, msg: &CanvasResizeMessage) {
        if let Some(ls) = self
            .layerstack
            .resized(msg.top, msg.right, msg.bottom, msg.left)
        {
            self.layerstack = Rc::new(ls);
        } else {
            warn!("Invalid resize: {:?}", msg);
        }
    }

    fn handle_layer_create(&mut self, msg: &LayerCreateMessage) {
        // TODO placement, clone, name
        Rc::make_mut(&mut self.layerstack)
            .add_layer(msg.id as LayerID, &Color::from_argb32(msg.fill));
    }

    fn handle_layer_attributes(&mut self, msg: &LayerAttributesMessage) {
        if let Some(layer) = Rc::make_mut(&mut self.layerstack).get_layer_mut(msg.id as LayerID) {
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
        if let Some(layer) = Rc::make_mut(&mut self.layerstack).get_layer_mut(msg.layer as LayerID)
        {
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

    fn handle_background(&mut self, pixels: &[u8]) {
        let tile = if pixels.len() == 4 {
            let color = Color::from_argb32(u32::from_be_bytes(pixels[..].try_into().unwrap()));
            Tile::new(&color, 0)
        } else {
            todo!()
        };

        Rc::make_mut(&mut self.layerstack).background = tile;
    }

    fn handle_fillrect(&mut self, user: UserID, msg: &FillRectMessage) {
        if msg.w == 0 || msg.h == 0 {
            warn!("FillRect(user {}): zero size rectangle", user);
            return;
        }

        if let Some(layer) = Rc::make_mut(&mut self.layerstack).get_layer_mut(msg.layer as LayerID)
        {
            editlayer::fill_rect(
                layer,
                user,
                &Color::from_argb32(msg.color),
                Blendmode::try_from(msg.mode).unwrap_or(Blendmode::Normal),
                &Rectangle::new(msg.x as i32, msg.y as i32, msg.w as i32, msg.h as i32),
            );
        } else {
            warn!("DrawDabsClassic: Layer {:04x} not found!", msg.layer);
        }
    }

    fn handle_drawdabs_classic(&mut self, user: UserID, msg: &DrawDabsClassicMessage) {
        if let Some(layer) = Rc::make_mut(&mut self.layerstack).get_layer_mut(msg.layer as LayerID)
        {
            brushes::drawdabs_classic(layer, user, &msg, &mut self.brushcache);
        } else {
            warn!("DrawDabsClassic: Layer {:04x} not found!", msg.layer);
        }
    }

    fn handle_drawdabs_pixel(&mut self, user: UserID, msg: &DrawDabsPixelMessage, square: bool) {
        if let Some(layer) = Rc::make_mut(&mut self.layerstack).get_layer_mut(msg.layer as LayerID)
        {
            brushes::drawdabs_pixel(layer, user, &msg, square);
        } else {
            warn!("DrawDabsPixel: Layer {:04x} not found!", msg.layer);
        }
    }

    fn make_savepoint_if_needed(&mut self) {
        // Don't make savepoints while a local fork exists, since
        // there will be stuff on the canvas that is not yet in
        // the mainline session history
        // TODO

        self.history.add_savepoint(self.layerstack.clone());
    }
}
