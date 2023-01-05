use std::{collections::HashMap, path::Path, pin::Pin};
use futures::TryStreamExt;
use sis::self_referencing;
use crate::{Result, Str, utils::GetStr};
use super::culture::{Culture};

flat_mod! { def, ty, rank, tier }

#[self_referencing]
#[derive(Debug)]
pub struct CountryGame {
    ranks: HashMap<Str, CountryRank>,
    #[borrows(ranks)]
    pub ty: HashMap<Str, CountryType<'this>>,
    #[borrows(ty)]
    pub definitions: HashMap<Str, Definition<'this>>
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
    pub(crate) async unsafe fn initialize_with_common (
        self: Pin<&'this mut Self>,
        common: &Path,
        cultures: &'this HashMap<Str, Culture<'this>>
    ) -> Result<()> {
        return self._try_initialize_async(
            |ranks| async move {
                CountryType::from_common(common, ranks)
                .await?
                .try_collect::<HashMap<_, _>>()
                .await
            },
            
            |tys| async move {
                Definition::from_common(common, tys, cultures)
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
    pub fn definition<'a> (&'a self, name: &str) -> Option<NamedDefinition<'a>> {
        return self.definitions().get_str_value(name)
    }
    
    #[inline]
    pub fn rank<'a> (&'a self, name: &str) -> Option<NamedCountryRank<'a>> {
        return self.ranks.get_str_value(name)
    }
}