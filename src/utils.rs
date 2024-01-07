use async_std::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::marker::Unpin;

// Aliases for custom errors and results
pub type ChatResult<T> = Result<T, ChatError>;
pub type ChatError = Box<dyn Error + Send + Sync + 'static>;

// Async function that can send a serialized json file
pub async fn send_json<O, P>(leaving: &mut O, packet: &P) -> ChatResult<()>
where
    O: async_std::io::Write + Unpin,
    P: Serialize,
{
    let mut json = serde_json::to_string(&packet)?;
    json.push('\n');

    leaving.write_all(json.as_bytes()).await?;
    Ok(())
}

// Function that can receives a serialized json file, maps it to lines each of type ChatResult.
pub fn receive<I, T>(incoming: I) -> impl Stream<Item = ChatResult<T>>
where
    I: async_std::io::BufRead + Unpin,
    T: DeserializeOwned,
{
    incoming.lines().map(|line| -> ChatResult<T> {
        let li = line?;
        let msg = serde_json::from_str::<T>(&li)?;
        Ok(msg)
    })
}
