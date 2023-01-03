use std::{collections::HashMap, path::Path};
use crate::{Result, try_collect};

flat_mod! { def, ty, rank }

#[derive(Debug)]
pub struct CountryGame {
    ranks: HashMap<String, CountryRank>,
    #[borrows(ranks)]
    #[covariant]
    ty: HashMap<String, CountryType<'this>>
}

impl CountryGame {
    #[inline]
    pub(crate) fn init (common: &Path) -> Result<Self> {
        let builder = CountryGameBuilder {
            ranks: try_collect(CountryRank::from_common(common)?)?,
            ty: |ranks| try_collect(CountryRank::from_common(common)?)?
        };

        return Ok(builder.build())
    }

    #[inline]
    pub fn ranks (&self) -> &HashMap<String, CountryRank> {
        return &self.ranks
    }

    #[inline]
    pub fn rank (&self, name: &str) -> Option<(&str, &CountryRank)> {
        return self.ranks.get_key_value(name)
            .map(|(x, y)| (x.as_str(), y))
    }
}