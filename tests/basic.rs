use std::{thread, time};

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
    first_client.send_message(Message::Chat(chat.clone()));
    let sent = second_client.read_chat().unwrap();
    assert_eq!(chat, sent);

    let chat = common::generate_chat(&second_user, &first_user);
    second_client.send_message(Message::Chat(chat.clone()));
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
    first_client.send_message(Message::Chat(chat.clone()));

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

    // TODO: find out why it's necessary to sleep briefly between sending messages
    first_client.send_message(Message::Join(common::create_join(&first_user, &group)));
    thread::sleep(time::Duration::from_millis(1));
    second_client.send_message(Message::Join(common::create_join(&second_user, &group)));
    thread::sleep(time::Duration::from_millis(1));
    third_client.send_message(Message::Join(common::create_join(&third_user, &group)));
    thread::sleep(time::Duration::from_millis(1));

    let chat = common::generate_group_chat(&first_user, &group);
    first_client.send_message(Message::Chat(chat.clone()));

    let sent = second_client.read_chat().unwrap();
    assert_eq!(chat, sent);
    let sent = third_client.read_chat().unwrap();
    assert_eq!(chat, sent);
}

#[test]
fn test_group_pending() {
    // TODO: find out why this case fails intermittently

    let _shared = common::TEST_LOCK.lock().unwrap();

    let group = common::generate_group();

    let first_user = common::generate_user();
    let second_user = common::generate_user();

    let mut first_client = common::create_client(&first_user);
    let mut second_client = common::create_client(&second_user);

    // Due to an unknown reason, it's necessary to sleep briefly between sending messages
    first_client.send_message(Message::Join(common::create_join(&first_user, &group)));
    thread::sleep(time::Duration::from_millis(1));
    second_client.send_message(Message::Join(common::create_join(&second_user, &group)));
    thread::sleep(time::Duration::from_millis(1));

    // Third joins the disconnects
    let third_user = common::generate_user();
    {
        let mut third_client = common::create_client(&third_user);

        third_client.send_message(Message::Join(common::create_join(&third_user, &group)));
        thread::sleep(time::Duration::from_millis(1));
    }

    let chat = common::generate_group_chat(&first_user, &group);
    first_client.send_message(Message::Chat(chat.clone()));

    // Third reconnects
    let mut third_client = common::create_client(&third_user);
    let sent = third_client.read_chat().unwrap();
    assert_eq!(chat, sent);
}
