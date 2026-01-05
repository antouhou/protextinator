use crate::style::{FontFamily, Weight};
use crate::utils::ArcCowStr;
use ahash::HashMap;
use cosmic_text::{fontdb, FontSystem};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FontFamilyQuery {
    pub family_query_string: ArcCowStr,
    pub weight: Weight,
}

impl FontFamilyQuery {
    pub fn split_families(&self) -> impl Iterator<Item = FontFamily> + use<'_> {
        self.family_query_string
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let unquoted = s
                    .strip_prefix(['\'', '"'])
                    .and_then(|s| s.strip_suffix(['\'', '"']))
                    .unwrap_or(s);
                FontFamily::parse(unquoted)
            })
    }
}

pub struct FontFamilyCache {
    /// This cache maps font family query strings to the first resolved font family name
    /// that exists on the current platform.
    /// For example, a query `"Helvetica, 'Segoe UI'"` will map to `"Helvetica"` on a Mac and
    /// `"Segoe UI"` on Windows.
    font_family_query_to_resolved_family_cache: HashMap<FontFamilyQuery, FontFamily>,
}

impl FontFamilyCache {
    pub fn new() -> Self {
        Self {
            font_family_query_to_resolved_family_cache: HashMap::default(),
        }
    }

    pub(crate) fn resolve_font_family_query(
        &mut self,
        query: FontFamilyQuery,
        font_system: &mut FontSystem,
    ) -> FontFamily {
        if let Some(cached) = self.font_family_query_to_resolved_family_cache.get(&query) {
            return cached.clone();
        }

        for family in query.split_families() {
            let res = font_system.db().query(&fontdb::Query {
                families: &[family.to_fontdb_family()],
                weight: fontdb::Weight::NORMAL,
                ..Default::default()
            });
            if res.is_some() {
                self.font_family_query_to_resolved_family_cache
                    .insert(query.clone(), family.clone());
                return family;
            }
        }

        // Fallback to SansSerif if no family is found
        let fallback = FontFamily::SansSerif;
        self.font_family_query_to_resolved_family_cache
            .insert(query, fallback.clone());
        fallback
    }
}

#[cfg(test)]
mod test {
    use crate::font_family_query::FontFamilyQuery;
    use crate::style::{FontFamily, Weight};
    use crate::utils::ArcCowStr;

    #[test]
    pub fn should_split_families_correctly() {
        let query = FontFamilyQuery {
            family_query_string: ArcCowStr::from("Helvetica, 'Segoe UI', Arial, sans-serif"),
            weight: Weight::NORMAL,
        };

        let families: Vec<FontFamily> = query.split_families().collect();
        assert_eq!(
            families,
            vec![
                FontFamily::Name("Helvetica".into()),
                FontFamily::Name("Segoe UI".into()),
                FontFamily::Name("Arial".into()),
                FontFamily::SansSerif,
            ]
        );
    }
}
