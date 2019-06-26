use conver::message::Message;

mod common;

#[test]
fn test_chat() {
    let _shared = common::TEST_LOCK.lock().unwrap();

    let first_user = common::generate_user();
    let second_user = common::generate_user();

    let mut first_client = common::create_client(&first_user);
    let mut second_client = common::create_client(&second_user);

    let chat = common::generate_chat(&first_user, &second_user);
    first_client
        .send_message(Message::Chat(chat.clone()))
        .unwrap();
    let sent = second_client.read_chat().unwrap();
    assert_eq!(chat, sent);

    let chat = common::generate_chat(&second_user, &first_user);
    second_client
        .send_message(Message::Chat(chat.clone()))
        .unwrap();
    let sent = first_client.read_chat().unwrap();
    assert_eq!(chat, sent);
}

#[test]
fn test_chat_pending() {
    let _shared = common::TEST_LOCK.lock().unwrap();

    let first_user = common::generate_user();
    let second_user = common::generate_user();

    let mut first_client = common::create_client(&first_user);
    let chat = common::generate_chat(&first_user, &second_user);
    first_client
        .send_message(Message::Chat(chat.clone()))
        .unwrap();

    // Second connects after being sent a message
    let mut second_client = common::create_client(&second_user);
    let sent = second_client.read_chat().unwrap();
    assert_eq!(chat, sent);
}

#[test]
fn test_group() {
    let _shared = common::TEST_LOCK.lock().unwrap();

    let group = common::generate_group();

    let first_user = common::generate_user();
    let second_user = common::generate_user();
    let third_user = common::generate_user();

    let mut first_client = common::create_client(&first_user);
    let mut second_client = common::create_client(&second_user);
    let mut third_client = common::create_client(&third_user);

    first_client
        .send_message(Message::Join(common::create_join(&first_user, &group)))
        .unwrap();
    second_client
        .send_message(Message::Join(common::create_join(&second_user, &group)))
        .unwrap();
    third_client
        .send_message(Message::Join(common::create_join(&third_user, &group)))
        .unwrap();

    let chat = common::generate_group_chat(&first_user, &group);
    first_client
        .send_message(Message::Chat(chat.clone()))
        .unwrap();

    let sent = second_client.read_chat().unwrap();
    assert_eq!(chat, sent);
    let sent = third_client.read_chat().unwrap();
    assert_eq!(chat, sent);
}

#[test]
fn test_group_pending() {
    let _shared = common::TEST_LOCK.lock().unwrap();

    let group = common::generate_group();

    let first_user = common::generate_user();
    let second_user = common::generate_user();

    let mut first_client = common::create_client(&first_user);
    let mut second_client = common::create_client(&second_user);

    first_client
        .send_message(Message::Join(common::create_join(&first_user, &group)))
        .unwrap();
    second_client
        .send_message(Message::Join(common::create_join(&second_user, &group)))
        .unwrap();

    // Third joins then disconnects
    let third_user = common::generate_user();
    {
        let mut third_client = common::create_client(&third_user);

        third_client
            .send_message(Message::Join(common::create_join(&third_user, &group)))
            .unwrap();
    }

    let chat = common::generate_group_chat(&first_user, &group);
    first_client
        .send_message(Message::Chat(chat.clone()))
        .unwrap();

    // Third reconnects
    let mut third_client = common::create_client(&third_user);
    let sent = third_client.read_chat().unwrap();
    assert_eq!(chat, sent);
}
