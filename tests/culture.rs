#![feature(iterator_try_collect)]

use std::collections::HashMap;
use vicky3_mod::{Result, Game, culture::RawCulture};

#[test]
fn read () -> Result<()> {
    unsafe { Game::initialize("D:/SteamLibrary/steamapps/common/Victoria 3/game") };
    let raw = RawCulture::from_common(Game::common())?
        .try_collect::<HashMap<_, _>>()?;

    println!("{raw:#?}");
    Ok(())
}