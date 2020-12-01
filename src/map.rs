use std::borrow::Cow;
use std::fmt::{self, Debug};

#[derive(Clone)]
pub struct Map {
    inner: Vec<(Cow<'static, str>, Cow<'static, str>)>,
}

impl Map {
    #[inline]
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        match self.inner.binary_search_by(|a| a.0.as_ref().cmp(key)) {
            Ok(i) => self.inner.get(i).map(|kv| kv.1.as_ref()),
            Err(_) => None,
        }
    }

    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let key = key.into();
        let value = value.into();

        let i = self.inner.binary_search_by(|a| a.0.cmp(&key));
        match i {
            Ok(i) => {
                let old_value = self.inner.get_mut(i).expect("i can't be out of bounds");
                let new_value = Cow::Owned(format!("{}, {}", old_value.1, value));
                *old_value = (key, new_value);
            }
            Err(i) => self.inner.insert(i, (key, value)),
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<(Cow<'static, str>, Cow<'static, str>)> {
        match self.inner.binary_search_by(|a| a.0.as_ref().cmp(key)) {
            Ok(i) => Some(self.inner.remove(i)),
            Err(_) => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + Clone {
        self.inner.iter().map(|t| (t.0.as_ref(), t.1.as_ref()))
    }
}

impl Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl Default for Map {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map() {
        let mut map = Map::new();
        {
            assert_eq!(map.len(), 0);
            assert!(map.is_empty());
            assert!(map.get("nothing").is_none());

            let mut iter = map.iter();
            assert!(iter.next().is_none());
        }

        {
            map.insert("content-type", "text/plain");
            assert_eq!(map.len(), 1);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(map.get("content-type"), Some("text/plain"));

            let iter = map.iter();
            iter.eq(vec![("content-type", "text/plain")].into_iter());
        }

        {
            map.insert("cache-control", "public, max-age=86400");
            assert_eq!(map.len(), 2);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(map.get("content-type"), Some("text/plain"));
            assert_eq!(map.get("cache-control"), Some("public, max-age=86400"));

            let iter = map.iter();
            iter.eq(vec![
                ("cache-control", "public, max-age=86400"),
                ("content-type", "text/plain"),
            ]
            .into_iter());
        }

        {
            map.insert("x-amz-storage-class", "standard");
            assert_eq!(map.len(), 3);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(map.get("content-type"), Some("text/plain"));
            assert_eq!(map.get("cache-control"), Some("public, max-age=86400"));
            assert_eq!(map.get("x-amz-storage-class"), Some("standard"));

            let iter = map.iter();
            iter.eq(vec![
                ("cache-control", "public, max-age=86400"),
                ("content-type", "text/plain"),
                ("x-amz-storage-class", "standard"),
            ]
            .into_iter());
        }

        {
            map.remove("content-type");
            assert_eq!(map.len(), 2);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(map.get("cache-control"), Some("public, max-age=86400"));
            assert_eq!(map.get("x-amz-storage-class"), Some("standard"));

            let iter = map.iter();
            iter.eq(vec![
                ("cache-control", "public, max-age=86400"),
                ("x-amz-storage-class", "standard"),
            ]
            .into_iter());
        }

        {
            map.remove("x-amz-look-at-how-many-headers-you-have");
            assert_eq!(map.len(), 2);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(map.get("cache-control"), Some("public, max-age=86400"));
            assert_eq!(map.get("x-amz-storage-class"), Some("standard"));

            let iter = map.iter();
            iter.eq(vec![
                ("cache-control", "public, max-age=86400"),
                ("x-amz-storage-class", "standard"),
            ]
            .into_iter());
        }

        {
            map.insert("cache-control", "immutable");
            assert_eq!(map.len(), 2);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(
                map.get("cache-control"),
                Some("public, max-age=86400, immutable")
            );
            assert_eq!(map.get("x-amz-storage-class"), Some("standard"));

            let iter = map.iter();
            iter.eq(vec![
                ("cache-control", "public, max-age=86400, immutable"),
                ("x-amz-storage-class", "standard"),
            ]
            .into_iter());
        }
    }
}
