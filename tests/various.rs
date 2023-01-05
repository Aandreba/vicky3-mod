#![feature(iterator_try_collect)]

use std::collections::HashMap;
use futures::{TryStreamExt};
use vicky3_mod::{Result, Game, religion::Religion, culture::Culture};

#[tokio::test]
async fn culture () -> Result<()> {
    unsafe { Game::initialize("D:/SteamLibrary/steamapps/common/Victoria 3/game").await };
    
    let religion = Religion::from_common(Game::common()).await?
        .try_collect::<HashMap<_, _>>()
        .await?;

    let culture = Culture::from_common(Game::common(), &religion).await?
        .try_collect::<HashMap<_, _>>()
        .await?;

    println!("{culture:#?}");

    Ok(())
}