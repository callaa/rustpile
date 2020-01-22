use super::brushes;
use super::compression;
use super::history::History;
use crate::paint::annotation::{AnnotationID, VAlign};
use crate::paint::layerstack::{LayerFill, LayerInsertion, LayerStack};
use crate::paint::{editlayer, Blendmode, ClassicBrushCache, Color, LayerID, Rectangle, UserID};
use crate::protocol::message::*;

use std::convert::TryFrom;
use std::rc::Rc;
use tracing::warn;

pub struct CanvasState {
    layerstack: Rc<LayerStack>,
    history: History,
    brushcache: ClassicBrushCache,
    local_user_id: UserID,
}

impl CanvasState {
    pub fn new() -> CanvasState {
        CanvasState {
            layerstack: Rc::new(LayerStack::new(0, 0)),
            history: History::new(),
            brushcache: ClassicBrushCache::new(),
            local_user_id: 0,
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
            UndoPoint(user) => self.handle_undopoint(*user),
            CanvasResize(_, m) => self.handle_canvas_resize(m),
            LayerCreate(_, m) => self.handle_layer_create(m),
            LayerAttributes(_, m) => self.handle_layer_attributes(m),
            LayerRetitle(_, m) => self.handle_layer_retitle(m),
            LayerOrder(_, order) => self.handle_layer_order(order),
            LayerDelete(_, m) => self.handle_layer_delete(m),
            LayerVisibility(u, m) => self.handle_layer_visibility(*u, m),
            PutImage(u, m) => self.handle_putimage(*u, m),
            FillRect(user, m) => self.handle_fillrect(*user, m),
            PenUp(user) => self.handle_penup(*user),
            AnnotationCreate(_, m) => self.handle_annotation_create(m),
            AnnotationReshape(_, m) => self.handle_annotation_reshape(m),
            AnnotationEdit(_, m) => self.handle_annotation_edit(m),
            AnnotationDelete(_, id) => self.handle_annotation_delete(*id),
            PutTile(user, m) => self.handle_puttile(*user, m),
            CanvasBackground(_, m) => self.handle_background(m),
            DrawDabsClassic(user, m) => self.handle_drawdabs_classic(*user, m),
            DrawDabsPixel(user, m) => self.handle_drawdabs_pixel(*user, m, false),
            DrawDabsPixelSquare(user, m) => self.handle_drawdabs_pixel(*user, m, true),
            Undo(user, m) => self.handle_undo(*user, m),
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
            .for_each(|l| {
                editlayer::merge_sublayer(Rc::make_mut(l), sublayer_id);
            });
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
        let pos = match (
            msg.flags & LayerCreateMessage::FLAGS_INSERT != 0,
            msg.source,
        ) {
            (true, 0) => LayerInsertion::Bottom,
            (true, source) => LayerInsertion::Above(source as LayerID),
            (false, _) => LayerInsertion::Top,
        };

        let fill = if msg.flags & LayerCreateMessage::FLAGS_COPY != 0 {
            LayerFill::Copy(msg.source as LayerID)
        } else {
            LayerFill::Solid(Color::from_argb32(msg.fill))
        };

        if let Some(layer) =
            Rc::make_mut(&mut self.layerstack).add_layer(msg.id as LayerID, fill, pos)
        {
            layer.title = msg.name.clone();
        } else {
            // todo add_layer could return Result instead with a better error message
            warn!("LayerCreate: layer {:04x} could not be created", msg.id);
        }
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

    fn handle_layer_retitle(&mut self, msg: &LayerRetitleMessage) {
        if let Some(layer) = Rc::make_mut(&mut self.layerstack).get_layer_mut(msg.id as LayerID) {
            layer.title = msg.title.clone()
        } else {
            warn!("LayerRetitle: Layer {:04x} not found!", msg.id);
        }
    }

    fn handle_layer_order(&mut self, new_order: &[u16]) {
        let order: Vec<LayerID> = new_order.iter().map(|i| *i as LayerID).collect();
        self.layerstack = Rc::new(self.layerstack.reordered(&order));
    }

    fn handle_layer_delete(&mut self, msg: &LayerDeleteMessage) {
        let stack = Rc::make_mut(&mut self.layerstack);
        let id = msg.id as LayerID;
        if msg.merge {
            if let Some(below) = stack.find_layer_below(id) {
                let above = stack.get_layer_rc(id).unwrap();
                editlayer::merge(stack.get_layer_mut(below).unwrap(), &above);
            } else {
                warn!("LayerDelete: Cannot merge {:04x}", id);
                return;
            }
        }
        stack.remove_layer(id);
    }

    fn handle_layer_visibility(&mut self, user: UserID, msg: &LayerVisibilityMessage) {
        if user == self.local_user_id {
            if let Some(layer) = Rc::make_mut(&mut self.layerstack).get_layer_mut(msg.id as LayerID)
            {
                layer.hidden = !msg.visible;
            }
        }
    }

    fn handle_annotation_create(&mut self, msg: &AnnotationCreateMessage) {
        Rc::make_mut(&mut self.layerstack).add_annotation(
            msg.id,
            Rectangle::new(msg.x, msg.y, msg.w.max(1) as i32, msg.h.max(1) as i32),
        );
    }

    fn handle_annotation_reshape(&mut self, msg: &AnnotationReshapeMessage) {
        if let Some(a) = Rc::make_mut(&mut self.layerstack).get_annotation_mut(msg.id) {
            a.rect = Rectangle::new(msg.x, msg.y, msg.w.max(1) as i32, msg.h.max(1) as i32);
        }
    }

    fn handle_annotation_edit(&mut self, msg: &AnnotationEditMessage) {
        if let Some(a) = Rc::make_mut(&mut self.layerstack).get_annotation_mut(msg.id) {
            a.background = Color::from_argb32(msg.bg);
            a.protect = (msg.flags & 0x01) != 0;
            a.valign = match msg.flags & 0x06 {
                0x02 => VAlign::Center,
                0x06 => VAlign::Bottom,
                _ => VAlign::Top,
            };
            // border not implemented yet
            a.text = msg.text.clone();
        }
    }

    fn handle_annotation_delete(&mut self, id: AnnotationID) {
        Rc::make_mut(&mut self.layerstack).remove_annotation(id);
    }

    fn handle_puttile(&mut self, user_id: UserID, msg: &PutTileMessage) {
        if let Some(layer) = Rc::make_mut(&mut self.layerstack).get_layer_mut(msg.layer as LayerID)
        {
            if let Some(tile) = compression::decompress_tile(&msg.image, user_id) {
                editlayer::put_tile(
                    layer,
                    msg.sublayer as LayerID,
                    msg.col.into(),
                    msg.row.into(),
                    msg.repeat.into(),
                    &tile,
                );
            }
        } else {
            warn!("PutTile: Layer {:04x} not found!", msg.layer);
        }
    }

    fn handle_putimage(&mut self, user_id: UserID, msg: &PutImageMessage) {
        if let Some(layer) = Rc::make_mut(&mut self.layerstack).get_layer_mut(msg.layer as LayerID)
        {
            if msg.w == 0 || msg.h == 0 {
                warn!("PutImage: zero size!");
                return;
            }
            if msg.w > 65535 || msg.h > 65535 {
                warn!("PutImage: oversize image! ({}, {})", msg.w, msg.h);
                return;
            }
            if let Some(imagedata) =
                compression::decompress_image(&msg.image, (msg.w * msg.h) as usize)
            {
                let mode = Blendmode::try_from(msg.mode).unwrap_or_default();
                editlayer::draw_image(
                    layer,
                    user_id,
                    &imagedata,
                    &Rectangle::new(msg.x as i32, msg.y as i32, msg.w as i32, msg.h as i32),
                    1.0,
                    mode,
                );

                if mode.can_decrease_opacity() {
                    layer.optimize();
                }
            }
        } else {
            warn!("PutImage: Layer {:04x} not found!", msg.layer);
        }
    }

    fn handle_background(&mut self, pixels: &[u8]) {
        if let Some(tile) = compression::decompress_tile(pixels, 0) {
            Rc::make_mut(&mut self.layerstack).background = tile;
        }
    }

    fn handle_fillrect(&mut self, user: UserID, msg: &FillRectMessage) {
        if msg.w == 0 || msg.h == 0 {
            warn!("FillRect(user {}): zero size rectangle", user);
            return;
        }

        if let Some(layer) = Rc::make_mut(&mut self.layerstack).get_layer_mut(msg.layer as LayerID)
        {
            let mode = Blendmode::try_from(msg.mode).unwrap_or_default();
            editlayer::fill_rect(
                layer,
                user,
                &Color::from_argb32(msg.color),
                mode,
                &Rectangle::new(msg.x as i32, msg.y as i32, msg.w as i32, msg.h as i32),
            );

            if mode.can_decrease_opacity() {
                layer.optimize();
            }
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
