use std::borrow::Cow;
use std::fmt::{self, Debug};

/// A map used for holding query string paramenters or headers
#[derive(Clone)]
pub struct Map<'a> {
    inner: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> Map<'a> {
    /// Construct a new empty `Map`
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Get the number of elements in this `Map`
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    /// Return `true` if this `Map` is empty
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the value of an element of this `Map`, or `None` if it doesn't contain `key`
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner
            .binary_search_by(|a| a.0.as_ref().cmp(key))
            .map_or(None, |i| self.inner.get(i).map(|kv| kv.1.as_ref()))
    }

    /// Insert a new element in this `Map`
    ///
    /// If the `key` is already present, the `value` overwrites the existing value:
    ///
    /// ```
    /// let mut map = rusty_s3::Map::new();
    /// map.insert("k", "a");
    /// assert_eq!(map.get("k"), Some("a"));
    /// map.insert("k", "b");
    /// assert_eq!(map.get("k"), Some("b"));
    /// ```
    ///
    /// # Panics
    ///
    /// In case of out of bound inner index access
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        let key = key.into();
        let value = value.into();

        let i = self.inner.binary_search_by(|a| a.0.cmp(&key));
        match i {
            Ok(i) => {
                let old_value = self.inner.get_mut(i).expect("i can't be out of bounds");
                *old_value = (key, value);
            }
            Err(i) => self.inner.insert(i, (key, value)),
        }
    }

    /// Insert a new element in this `Map`
    ///
    /// If the `key` is already present, the `value` is appended to the existing value:
    ///
    /// ```
    /// let mut map = rusty_s3::Map::new();
    /// map.append("k", "a");
    /// assert_eq!(map.get("k"), Some("a"));
    /// map.append("k", "b");
    /// assert_eq!(map.get("k"), Some("a, b"));
    /// ```
    ///
    /// # Panics
    ///
    /// In case of out of bound inner index access
    pub fn append<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
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

    /// Remove an element from this `Map` and return it
    pub fn remove(&mut self, key: &str) -> Option<(Cow<'a, str>, Cow<'a, str>)> {
        match self.inner.binary_search_by(|a| a.0.as_ref().cmp(key)) {
            Ok(i) => Some(self.inner.remove(i)),
            Err(_) => None,
        }
    }

    /// Return an `Iterator` over this map
    ///
    /// The elements are always sorted in alphabetical order based on the key.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + Clone {
        self.inner.iter().map(|t| (t.0.as_ref(), t.1.as_ref()))
    }
}

/// The names of the elements whose values are secret, hidden from the
/// `Debug` output
const SENSITIVE: &[&str] = &[
    "authorization",
    "x-amz-copy-source-server-side-encryption-customer-key",
    "x-amz-security-token",
    "x-amz-server-side-encryption-customer-key",
];

impl Debug for Map<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for (key, value) in self.iter() {
            if SENSITIVE.iter().any(|name| key.eq_ignore_ascii_case(name)) {
                map.entry(&key, &format_args!("Sensitive"));
            } else {
                map.entry(&key, &value);
            }
        }
        map.finish()
    }
}

impl Default for Map<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

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

            assert!(map.iter().eq(vec![("content-type", "text/plain")]));
        }

        {
            map.insert("cache-control", "public, max-age=86400");
            assert_eq!(map.len(), 2);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(map.get("content-type"), Some("text/plain"));
            assert_eq!(map.get("cache-control"), Some("public, max-age=86400"));

            assert!(map.iter().eq(vec![
                ("cache-control", "public, max-age=86400"),
                ("content-type", "text/plain"),
            ]));
        }

        {
            map.insert("x-amz-storage-class", "standard");
            assert_eq!(map.len(), 3);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(map.get("content-type"), Some("text/plain"));
            assert_eq!(map.get("cache-control"), Some("public, max-age=86400"));
            assert_eq!(map.get("x-amz-storage-class"), Some("standard"));

            assert!(map.iter().eq(vec![
                ("cache-control", "public, max-age=86400"),
                ("content-type", "text/plain"),
                ("x-amz-storage-class", "standard"),
            ]));
        }

        {
            map.remove("content-type");
            assert_eq!(map.len(), 2);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(map.get("cache-control"), Some("public, max-age=86400"));
            assert_eq!(map.get("x-amz-storage-class"), Some("standard"));

            assert!(map.iter().eq(vec![
                ("cache-control", "public, max-age=86400"),
                ("x-amz-storage-class", "standard"),
            ]));
        }

        {
            map.remove("x-amz-look-at-how-many-headers-you-have");
            assert_eq!(map.len(), 2);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(map.get("cache-control"), Some("public, max-age=86400"));
            assert_eq!(map.get("x-amz-storage-class"), Some("standard"));

            assert!(map.iter().eq(vec![
                ("cache-control", "public, max-age=86400"),
                ("x-amz-storage-class", "standard"),
            ]));
        }

        {
            map.append("cache-control", "immutable");
            assert_eq!(map.len(), 2);
            assert!(!map.is_empty());
            assert!(map.get("nothing").is_none());
            assert_eq!(
                map.get("cache-control"),
                Some("public, max-age=86400, immutable")
            );
            assert_eq!(map.get("x-amz-storage-class"), Some("standard"));

            assert!(map.iter().eq(vec![
                ("cache-control", "public, max-age=86400, immutable"),
                ("x-amz-storage-class", "standard"),
            ]));
        }
    }

    #[test]
    fn debug_hides_secrets() {
        let mut map = Map::new();
        map.insert("x-amz-server-side-encryption-customer-algorithm", "AES256");
        map.insert("x-amz-server-side-encryption-customer-key", "c2VjcmV0");
        map.insert("x-amz-server-side-encryption-customer-key-MD5", "bWQ1");
        map.insert(
            "X-Amz-Copy-Source-Server-Side-Encryption-Customer-Key",
            "c2VjcmV0",
        );

        assert_eq!(
            format!("{map:?}"),
            r#"{"X-Amz-Copy-Source-Server-Side-Encryption-Customer-Key": Sensitive, "x-amz-server-side-encryption-customer-algorithm": "AES256", "x-amz-server-side-encryption-customer-key": Sensitive, "x-amz-server-side-encryption-customer-key-MD5": "bWQ1"}"#
        );
    }
}
