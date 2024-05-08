//! Some extensions to the [`teloxide`].

use std::fmt::Debug;
use std::sync::Arc;
use teloxide::{
    dispatching::{
        dialogue::{GetChatId, Storage},
        DpHandlerDescription,
    },
    prelude::*,
};

/// Enters a dialogue context efficiently.
pub fn efficient_dialogue_enter<Upd, S, D, Output>(
) -> Handler<'static, DependencyMap, Output, DpHandlerDescription>
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: Debug + Send,
    D: Default + Send + Sync + 'static,
    Upd: GetChatId + Clone + Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    dptree::entry()
        .chain(dptree::filter_map(|storage: Arc<S>, upd: Upd| {
            let chat_id = upd.chat_id()?;
            Some(Dialogue::new(storage, chat_id))
        }))
        .chain(dptree::filter_map_async(
            |dialogue: Dialogue<D, S>| async move {
                match dialogue.get().await {
                    Ok(state) => Some(state.unwrap_or_default()),
                    Err(err) => {
                        tracing::error!("dialogue.get() failed: {:?}", err);
                        None
                    }
                }
            },
        ))
}
