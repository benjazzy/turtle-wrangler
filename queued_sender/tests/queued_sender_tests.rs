use queued_sender::{QueuedSender, SenderState};

#[tokio::test]
async fn check_send_and_ready() {
    let message = "test_message";
    let mut out: Vec<&'static str> = vec![];

    let mut sender = QueuedSender::new(&mut out);
    assert_eq!(*sender.get_state(), SenderState::Ready);

    sender.send(message).await.expect("Problem sending message");
    assert_eq!(*sender.get_state(), SenderState::Sending);

    sender
        .ready()
        .await
        .expect("Problem setting sender to ready");
    assert_eq!(*sender.get_state(), SenderState::Ready);

    assert_eq!(out, vec![message]);
}
