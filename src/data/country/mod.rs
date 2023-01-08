use std::{collections::HashMap, path::Path};
use futures::{TryFutureExt, TryStreamExt};
use crate::{Str, Result};

flat_mod! { def, ty, rank, tier }

#[derive(Debug)]
pub struct GameCountry {
    pub ranks: HashMap<Str, CountryRank>,
    pub tys: HashMap<Str, CountryType>,
    pub definitions: HashMap<Str, CountryDefinition>
}

impl GameCountry {
    #[inline]
    pub async fn from_common (common: &Path) -> Result<Self> {
        let (ranks, tys, definitions) = futures::try_join! {
            CountryRank::from_common(common).and_then(TryStreamExt::try_collect::<HashMap<_, _>>),
            CountryType::from_common(common).and_then(TryStreamExt::try_collect::<HashMap<_, _>>),
            CountryDefinition::from_common(common).and_then(TryStreamExt::try_collect::<HashMap<_, _>>)
        }?;
        return Ok(Self { ranks, tys, definitions })
    }
}