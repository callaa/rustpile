use dpcore::canvas::{CanvasObserver, CanvasState, ObservableCanvasState};
use dpcore::paint::AoE;
use dpcore::protocol::message::{CanvasResizeMessage, CommandMessage};

use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

#[test]
fn test_canvas_state_observation() {
    let mut canvas = ObservableCanvasState::new(CanvasState::new());

    let observer = Rc::new(RefCell::new(TestObserver {
        changed: AoE::Nothing,
    }));

    canvas.add_observer(observer.clone());

    assert_eq!(observer.borrow().changed, AoE::Nothing);
    assert_eq!(canvas.observer_count(), 1);

    canvas.receive_message(&CommandMessage::CanvasResize(
        1,
        CanvasResizeMessage {
            top: 0,
            right: 100,
            bottom: 100,
            left: 0,
        },
    ));

    assert_eq!(observer.borrow().changed, AoE::Resize(0, 0));

    drop(observer);
    assert_eq!(canvas.observer_count(), 1);

    // missing observer is not noticed until the next notification
    canvas.receive_message(&CommandMessage::CanvasResize(
        1,
        CanvasResizeMessage {
            top: 100,
            right: 0,
            bottom: 0,
            left: 200,
        },
    ));

    assert_eq!(canvas.observer_count(), 0);
}

struct TestObserver {
    changed: AoE,
}

impl CanvasObserver for TestObserver {
    fn changed(&mut self, aoe: &AoE) {
        let changed = mem::replace(&mut self.changed, AoE::Nothing);
        self.changed = changed.merge(aoe.clone());
    }
}
