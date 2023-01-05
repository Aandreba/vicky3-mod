#![feature(iterator_try_collect)]

#[tokio::test]
async fn culture () -> Result<()> {
    unsafe { Game::initialize("D:/SteamLibrary/steamapps/common/Victoria 3/game").await };   



    Ok(())
}