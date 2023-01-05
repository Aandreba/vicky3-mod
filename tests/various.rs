#![feature(iterator_try_collect)]

use vicky3_mod::{Result, Game};

#[tokio::test]
async fn culture () -> Result<()> {
    unsafe { Game::initialize("D:/SteamLibrary/steamapps/common/Victoria 3/game").await };

    let countries = Game::countries().definition("SPA");
    println!("{countries:#?}");

    Ok(())
}