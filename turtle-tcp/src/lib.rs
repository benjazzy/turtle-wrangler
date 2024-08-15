use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::error;

const MARK_SIZE: usize = 4;
const MARK_BYTE: u8 = 0xa;
const MARK: [u8; MARK_SIZE] = [MARK_BYTE; MARK_SIZE];

pub fn parse_buffer<T>(data: &mut Vec<u8>) -> Vec<T>
where
    T: DeserializeOwned,
{
    let mut messages = vec![];
    while let Some(message) = parse_message(data) {
        match serde_json::from_slice(&message) {
            Ok(m) => messages.push(m),
            Err(e) => {
                println!("{e}");
                error!("Problem deserializing message {e}");
            }
        }
    }

    messages
}

fn parse_message(data: &mut Vec<u8>) -> Option<Vec<u8>> {
    let pos = data.windows(MARK_SIZE).position(|d| d == MARK)?;
    let message = data.drain(0..pos).collect();
    if data.len() >= MARK_SIZE {
        data.drain(0..MARK_SIZE);
    }

    Some(message)
}

pub fn message_to_bytes<T>(message: &T) -> Result<Vec<u8>, serde_json::Error>
where
    T: Serialize,
{
    let mut data = serde_json::to_vec(message)?;
    data.extend_from_slice(&MARK);

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_parse_buffer() {
        let message1 = "test_message1";
        let message2 = "test_message2";
        let mut data = vec![];
        data.extend_from_slice(serde_json::to_string(message1).unwrap().as_bytes());
        data.extend_from_slice(&MARK[..]);
        data.extend_from_slice(serde_json::to_string(message2).unwrap().as_bytes());
        data.extend_from_slice(&MARK[..]);

        let expected_out = vec![message1.to_string(), message2.to_string()];

        assert_eq!(parse_buffer::<String>(&mut data), expected_out);
        assert!(data.is_empty());
    }

    #[test]
    fn check_parse_single() {
        let message = b"test_message";
        let mut message_vec = vec![];
        message_vec.extend_from_slice(message);
        let mut data = vec![];
        data.extend_from_slice(message);
        data.extend_from_slice(&MARK[..]);

        assert_eq!(parse_message(&mut data), Some(message_vec));
        assert!(data.is_empty());
    }

    #[test]
    fn check_empty_no_mark() {
        let mut data = vec![];
        assert_eq!(parse_message(&mut data), None);
        assert!(data.is_empty());
    }

    #[test]
    fn check_some_no_mark() {
        let message = b"test_message";
        let mut data = vec![];
        data.extend_from_slice(message);
        let data_copy = data.clone();

        assert_eq!(parse_message(&mut data), None);
        assert_eq!(data, data_copy);
    }
}
