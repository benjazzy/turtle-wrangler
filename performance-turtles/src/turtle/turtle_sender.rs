mod turtle_sender_inner;
mod turtle_sender_container;

enum LockItem {
    Command(),
    Request(),
    Lock(),
}
enum SenderState {
    Normal(),
    Locked(Vec<LockItem>),
}

struct TurtleSenderContainer {
    state: SenderState,
}
