use dpcore::canvas::CanvasState;
use dpcore::paint::*;
use dpcore::protocol::message::{CommandMessage, Message};

#[test]
fn test_simple_conflict() {
    let mut canvas = CanvasState::new();
    let white = Color::rgb8(255, 255, 255);

    // Create a small canvas with one layer filled with solid white
    canvas.receive_message(&m("1 resize right=64 bottom=64"));
    canvas.receive_message(&m("1 newlayer id=0x0101 fill=#ffffff"));
    assert_eq!(lc(&canvas), Some(white));

    // Now, let's draw something
    canvas.receive_local_message(&m("1 undopoint"));
    canvas.receive_local_message(&m("1 fillrect layer=0x0101 x=0 y=0 w=10 h=10 color=#ff0000 mode=1"));

    assert_eq!(lc(&canvas), None); // not solid white anymore

    // Second user draws a white patch over our patch: retcon removes our changes
    canvas.receive_message(&m("2 fillrect layer=0x0101 x=5 y=5 w=10 h=10 color=#ffffff mode=1"));

    assert_eq!(lc(&canvas), Some(white), "expected retcon"); // is solid white again
}

fn m(msg: &str) -> CommandMessage {
    match Message::from_text(&msg.parse().unwrap()).unwrap() {
        Message::Command(m) => m,
        _ => panic!("Not a command message: {}", msg),
    }
}

fn lc(canvas: &CanvasState) -> Option<Color> {
    canvas.layerstack().get_layer(0x0101).unwrap().solid_color()
}
