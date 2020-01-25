use super::CanvasState;
use crate::paint::{AoE, LayerStack};
use crate::protocol::message::CommandMessage;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub trait CanvasObserver {
    fn changed(&mut self, area: &AoE);
}

pub struct ObservableCanvasState {
    canvas: CanvasState,
    observers: Vec<Weak<RefCell<dyn CanvasObserver>>>,
}

/// A wrapper around CanvasState that allows observers to listen for changes.
impl ObservableCanvasState {
    pub fn new(canvas: CanvasState) -> ObservableCanvasState {
        ObservableCanvasState {
            canvas,
            observers: Vec::new(),
        }
    }

    /// Unwrap the inner CanvasState object
    pub fn into_inner(self) -> CanvasState {
        self.canvas
    }

    /// Add a new observer.
    /// This struct will hold a weak reference to it.
    pub fn add_observer(&mut self, o: Rc<RefCell<dyn CanvasObserver>>) {
        self.observers.push(Rc::downgrade(&o));
    }

    pub fn observer_count(&self) -> usize {
        return self.observers.len();
    }

    /// Get a read only reference to layerstack under observation
    pub fn layerstack(&self) -> &LayerStack {
        &self.canvas.layerstack()
    }

    /// Handle a command.
    /// Subscribers will be notified of any possible visual changes.
    pub fn receive_message(&mut self, msg: &CommandMessage) {
        let aoe = self.canvas.receive_message(msg);
        if aoe != AoE::Nothing {
            self.notify(aoe);
        }
    }

    /// Handle a local command.
    /// Subscribers will be notified of any possible visual changes.
    pub fn receive_local_message(&mut self, msg: &CommandMessage) {
        let aoe = self.canvas.receive_local_message(msg);
        if aoe != AoE::Nothing {
            self.notify(aoe);
        }
    }

    fn notify(&mut self, aoe: AoE) {
        let mut cleanup = false;

        for o in self.observers.iter() {
            if let Some(o_rc) = o.upgrade() {
                let mut observer = o_rc.borrow_mut();
                observer.changed(&aoe);
            } else {
                cleanup = true;
            }
        }

        if cleanup {
            self.observers.retain(|o| o.upgrade().is_some());
        }
    }
}
