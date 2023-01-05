#![feature(iterator_try_collect)]

use std::collections::HashMap;
use futures::{TryStreamExt};
use vicky3_mod::{Result, Game, culture::RawCulture};

#[tokio::test]
async fn read () -> Result<()> {
    unsafe { Game::initialize("D:/SteamLibrary/steamapps/common/Victoria 3/game").await };
    let raw = RawCulture::from_common(Game::common()).await?
        .try_collect::<HashMap<_, _>>()
        .await?;

    println!("{raw:#?}");
    Ok(())
}