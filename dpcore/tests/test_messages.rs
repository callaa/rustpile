use dpcore::protocol::message::*;

#[test]
fn test_message_serialization() {
    let test_data = vec![
        (
            b"\x00\x0b\x20\x01\x03\x05helloworld".to_vec(),
            Message {
                user_id: 1,
                body: Body::Join(JoinMessage {
                    flags: 3,
                    name: "hello".to_string(),
                    avatar: b"world".to_vec(),
                }),
            },
        ),
        (
            b"\x00\x00\x21\x80".to_vec(),
            Message {
                user_id: 128,
                body: Body::Leave,
            },
        ),
        (
            b"\x00\x03\x22\xff\x10\x20\x30".to_vec(),
            Message {
                user_id: 255,
                body: Body::SessionOwner(b"\x10\x20\x30".to_vec()),
            },
        ),
        (
            b"\x00\x06\x23\x0a\x05hello".to_vec(),
            Message {
                user_id: 10,
                body: Body::Chat(ChatMessage {
                    flags: 0x05,
                    message: "hello".to_string(),
                }),
            },
        ),
        (
            b"\x00\x15\x94\x01\x00\x01\0\0\0\x01\0\0\0\x02\0\0\0\0\x01\x04\x05\x01\x00\xff\x80"
                .to_vec(),
            Message {
                user_id: 1,
                body: Body::DrawDabsClassic(DrawDabsClassicMessage {
                    layer: 1,
                    x: 1,
                    y: 2,
                    color: 0,
                    mode: 1,
                    dabs: vec![ClassicDab {
                        x: 4,
                        y: 5,
                        size: 256,
                        opacity: 255,
                        hardness: 128,
                    }],
                }),
            },
        ),
    ];

    for (sample, expected) in test_data {
        let msg = Message::deserialize(&sample).unwrap();
        assert_eq!(msg, expected, "deserialization");
        assert_eq!(msg.serialize(), sample, "re-serialization");

        let text = msg.to_string();
        assert_eq!(
            Message::from_text(&text.parse().unwrap()).unwrap(),
            expected,
            "text encode + parsing"
        );
    }
}
