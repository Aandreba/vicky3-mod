use std::{collections::{BTreeMap}, path::Path};
use futures::{TryFutureExt, TryStreamExt};
use crate::{Result};

flat_mod! { def, ty, rank, tier }

#[derive(Debug)]
pub struct GameCountry {
    pub ranks: BTreeMap<String, CountryRank>,
    pub tys: BTreeMap<String, CountryType>,
    pub definitions: BTreeMap<String, CountryDefinition>
}

impl GameCountry {
    #[inline]
    pub async fn from_common (common: &Path) -> Result<Self> {
        let (ranks, tys, definitions) = futures::try_join! {
            CountryRank::from_common(common).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>),
            CountryType::from_common(common).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>),
            CountryDefinition::from_common(common).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>)
        }?;
        return Ok(Self { ranks, tys, definitions })
    }
}