pub trait Command {
    const NEEDS_LOCK: bool;
}

pub trait Request: Command {
    type Response;
}