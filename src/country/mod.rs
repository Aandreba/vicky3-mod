use std::{collections::HashMap, path::Path, pin::Pin};
use sis::self_referencing;
use crate::{Result, try_collect, Str};

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
    pub unsafe fn new_uninit (common: &Path) -> Result<Self> {
        let ranks = try_collect(CountryRank::from_common(common)?)?;
        return Ok(Self::_new_uninit(ranks))
    }

    #[inline]
    pub(crate) unsafe fn initialize_with_common (self: Pin<&'this mut Self>, common: &Path) -> Result<()> {
        return self._try_initialize(
            |ranks| try_collect(CountryType::from_common(common, ranks)?)
        )
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