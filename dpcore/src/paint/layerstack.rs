use std::rc::Rc;

use super::annotation::Annotation;
use super::tile::{Tile, TileData, TILE_SIZE};
use super::{Color, Layer};

#[derive(Clone)]
pub struct LayerStack {
    layers: Rc<Vec<Rc<Layer>>>,
    annotations: Rc<Vec<Rc<Annotation>>>,
    pub background: Tile,
    width: u32,
    height: u32,
}

impl LayerStack {
    pub fn new(width: u32, height: u32) -> LayerStack {
        LayerStack {
            layers: Rc::new(Vec::<Rc<Layer>>::new()),
            annotations: Rc::new(Vec::<Rc<Annotation>>::new()),
            background: Tile::Blank,
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Add a new layer and return a mutable reference to it
    ///
    /// If a layer with the given ID exists already, None will be returned.
    pub fn add_layer(&mut self, id: i32, fill: &Color) -> Option<&mut Layer> {
        if self.find_layer_index(id).is_some() {
            return None;
        }

        let layers = Rc::make_mut(&mut self.layers);
        layers.push(Rc::new(Layer::new(id, self.width, self.height, fill)));
        Some(Rc::make_mut(layers.last_mut().unwrap()))
    }

    /// Find a layer with the given ID
    pub fn get_layer(&self, id: i32) -> Option<&Layer> {
        for l in self.layers.iter() {
            if l.id == id {
                return Some(l);
            }
        }
        None
    }

    /// Find a layer with the given ID
    pub fn get_layer_mut(&mut self, id: i32) -> Option<&mut Layer> {
        if let Some(idx) = self.find_layer_index(id) {
            Some(Rc::make_mut(&mut Rc::make_mut(&mut self.layers)[idx]))
        } else {
            None
        }
    }

    /// Remove a layer with the given ID
    pub fn remove_layer(&mut self, id: i32) {
        if let Some(idx) = self.find_layer_index(id) {
            Rc::make_mut(&mut self.layers).remove(idx);
        }
    }

    fn find_layer_index(&self, id: i32) -> Option<usize> {
        for (i, l) in self.layers.iter().enumerate() {
            if l.id == id {
                return Some(i);
            }
        }
        None
    }

    /// Flatten layer stack content
    pub fn flatten_tile(&self, i: u32, j: u32) -> TileData {
        let mut destination = self.background.clone_data();

        if (i * TILE_SIZE) < self.width && (j * TILE_SIZE) < self.height {
            for layer in self.layers.iter() {
                layer.flatten_tile(&mut destination, i, j);
            }
        }

        destination
    }

    // Convert to a flat image
    pub fn to_image(&self) -> (Vec<u32>, u32, u32) {
        let xtiles = Tile::div_up(self.width) as usize;
        let ytiles = Tile::div_up(self.height) as usize;

        let tw = TILE_SIZE as usize;
        let width = self.width as usize;
        let height = self.height as usize;

        let mut image = vec![0u32; width * height];

        for j in 0..ytiles {
            let h = tw.min(height - (j * tw));
            for i in 0..xtiles {
                let td = self.flatten_tile(i as u32, j as u32);
                let w = tw.min(width - (i * tw));
                for y in 0..h {
                    let dest_offset = (j * tw + y) * width + i * tw;
                    let src_offset = y * tw;

                    image[dest_offset..dest_offset + w]
                        .copy_from_slice(&td.pixels[src_offset..src_offset + w]);
                }
            }
        }

        (image, self.width, self.height)
    }

    /// Return a resized copy of this stack
    pub fn resized(&self, top: i32, right: i32, bottom: i32, left: i32) -> Option<LayerStack> {
        let new_width = left + self.width as i32 + right;
        let new_height = top + self.height as i32 + bottom;
        if new_width <= 0 || new_height <= 0 {
            return None;
        }

        Some(LayerStack {
            layers: Rc::new(
                self.layers
                    .iter()
                    .map(|l| Rc::new(l.resized(top, right, bottom, left)))
                    .collect(),
            ),
            annotations: Rc::new(
                self.annotations
                    .iter()
                    .map(|a| {
                        Rc::new(Annotation {
                            rect: a.rect.offset(left, top),
                            text: a.text.clone(),
                            ..**a
                        })
                    })
                    .collect(),
            ),
            background: self.background.clone(),
            width: new_width as u32,
            height: new_height as u32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_addition() {
        let mut stack = LayerStack::new(256, 256);
        stack.add_layer(1, &Color::TRANSPARENT);

        // Adding a layer with an existing ID does nothing
        assert!(stack.add_layer(1, &Color::TRANSPARENT).is_none());

        let layer = stack.get_layer(1).unwrap();

        assert!(stack.get_layer(0).is_none());
        assert_eq!(layer.width(), 256);
        assert_eq!(layer.height(), 256);
    }

    #[test]
    fn test_layer_removal() {
        let mut stack = LayerStack::new(256, 256);
        stack.add_layer(1, &Color::TRANSPARENT);
        stack.add_layer(2, &Color::TRANSPARENT);
        stack.add_layer(3, &Color::TRANSPARENT);

        assert_eq!(stack.layers.len(), 3);
        stack.remove_layer(2);
        assert_eq!(stack.layers.len(), 2);
        assert!(stack.get_layer(1).is_some());
        assert!(stack.get_layer(2).is_none());
        assert!(stack.get_layer(3).is_some());
    }

    #[test]
    fn test_flattening() {
        let mut stack = LayerStack::new(128, 64);
        stack.background = Tile::new_solid(&Color::rgb8(255, 255, 255), 0);
        stack.add_layer(1, &Color::TRANSPARENT);
        stack.add_layer(2, &Color::TRANSPARENT);

        let layer = stack.get_layer_mut(1).unwrap();
        *layer.tile_mut(0, 0) = Tile::new_solid(&Color::rgb8(255, 0, 0), 0);
        layer.opacity = 0.5;

        let t1 = stack.flatten_tile(0, 0);
        assert_eq!(t1.pixels[0], Color::rgb8(255, 128, 128).as_pixel());

        let t2 = stack.flatten_tile(1, 0);
        assert_eq!(t2.pixels[0], Color::rgb8(255, 255, 255).as_pixel());
    }
}
