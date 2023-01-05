use std::{collections::HashMap, path::Path, pin::Pin};
use futures::TryStreamExt;
use sis::self_referencing;
use crate::{Result, Str};

flat_mod! { def, ty, rank, tier }

#[self_referencing]
#[derive(Debug)]
pub struct CountryGame {
    ranks: HashMap<Str, CountryRank>,
    #[borrows(ranks)]
    pub ty: HashMap<Str, CountryType<'this>>
}

impl<'this> CountryGame<'this> {
    #[inline]
    pub async unsafe fn new_uninit (common: &Path) -> Result<CountryGame<'this>> {
        let ranks = CountryRank::from_common(common).await?
            .try_collect::<HashMap<_, _>>()
            .await?;
        return Ok(Self::_new_uninit(ranks))
    }

    #[inline]
    pub(crate) async unsafe fn initialize_with_common (self: Pin<&'this mut Self>, common: &Path) -> Result<()> {
        return self._try_initialize_async(
            |ranks| async move {
                CountryType::from_common(common, ranks)
                .await?
                .try_collect::<HashMap<_, _>>()
                .await
            }
        ).await
    }

    #[inline]
    pub fn ranks (&self) -> &HashMap<Str, CountryRank> {
        return &self.ranks
    }

    #[inline]
    pub fn rank (&self, name: &str) -> Option<NamedCountryRank<'_>> {
        return self.ranks.get_key_value(name)
    }
}