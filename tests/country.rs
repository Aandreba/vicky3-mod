#![feature(iterator_try_collect)]

use vicky3_mod::{Game, Result};

#[test]
fn read () -> Result<()> {
    unsafe { Game::initialize("D:/SteamLibrary/steamapps/common/Victoria 3/game")? };
    
    Game::country().rank(name);
    Ok(())
}