use std::collections::HashSet;

/// A specific resource as discovered from the K8s API. Usually, all fields
/// are filled, except as in tests and docs, where the resource can be partial.
#[derive(Debug)]
pub struct Resource {
    pub group: Option<String>,
    pub version: Option<String>,
    pub plural: Option<String>,
    pub kind: Option<String>,
    pub singular: Option<String>,
    pub shortcuts: Option<HashSet<String>>,
    pub categories: Option<HashSet<String>>,
}

impl Resource {
    /// Make a builder with keyword arguments for easier construction.
    /// Missing fields remain `None`. Some parameters can fill several fields:
    /// for example, `group("v1")` sets both version to v1 and group to "".
    ///
    /// # Examples
    ///
    /// ```
    /// use example1::Resource;
    /// let r = Resource::builder().plural('pods').group('v1').build()
    /// assert_eq!(r.group, "")
    /// assert_eq!(r.version, "v1")
    /// assert_eq!(r.plural, "pods")
    /// ```
    pub fn builder() -> ResourceBuilder {
        ResourceBuilder::default()
    }
}

impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        self.group == other.group && self.version == other.version && self.plural == other.plural
    }
}

#[derive(Default)]
pub struct ResourceBuilder {
    group: Option<String>,
    version: Option<String>,
    plural: Option<String>,
    kind: Option<String>,
    singular: Option<String>,
    shortcuts: Option<HashSet<String>>,
    categories: Option<HashSet<String>>,
}

impl ResourceBuilder {
    pub fn build(self) -> Resource {
        Resource {
            group: self.group,
            version: self.version,
            plural: self.plural,
            kind: self.kind,
            singular: self.singular,
            shortcuts: self.shortcuts,
            categories: self.categories,
        }
    }

    pub fn group(mut self, group: impl Into<String>) -> Self {
        let group = group.into();
        if group == "v1" {
            self.group = Some(String::new());
            self.version = Some(group);
        } else {
            self.group = Some(group);
        }
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn plural(mut self, plural: impl Into<String>) -> Self {
        self.plural = Some(plural.into());
        self
    }

    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    pub fn singular(mut self, singular: impl Into<String>) -> Self {
        self.singular = Some(singular.into());
        self
    }

    pub fn shortcuts(mut self, shortcuts: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.shortcuts = Some(shortcuts.into_iter().map(Into::into).collect());
        self
    }

    pub fn categories(mut self, categories: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.categories = Some(categories.into_iter().map(Into::into).collect());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn empty() {
        let r = Resource::builder().build();
        assert_eq!(r.group, None);
        assert_eq!(r.version, None);
        assert_eq!(r.plural, None);
        assert_eq!(r.kind, None);
        assert_eq!(r.singular, None);
        assert_eq!(r.shortcuts, None);
        assert_eq!(r.categories, None);
    }

    #[rstest]
    fn full() {
        let r = Resource::builder()
            .group("kopf.dev")
            .version("v1")
            .plural("kopfexamples")
            .kind("KopfExample")
            .singular("kopfexample")
            .shortcuts(["kex1", "kex2"])
            .categories(["cat1", "cat2"])
            .build();
        assert_eq!(r.group, Some(String::from("kopf.dev")));
        assert_eq!(r.version, Some(String::from("v1")));
        assert_eq!(r.plural, Some(String::from("kopfexamples")));
        assert_eq!(r.kind, Some(String::from("KopfExample")));
        assert_eq!(r.singular, Some(String::from("kopfexample")));
        assert_eq!(r.shortcuts, Some(HashSet::from([String::from("kex1"), String::from("kex2")])));
        assert_eq!(r.categories, Some(HashSet::from([String::from("cat1"), String::from("cat2")])));
    }

    #[rstest]
    fn overrides() {
        let r = Resource::builder()
            .plural("plural1")
            .plural("plural2")
            .build();
        assert_eq!(r.plural, Some(String::from("plural2")));
    }

    #[rstest]
    fn v1core() {
        let r = Resource::builder().group("v1").build();
        assert_eq!(r.group, Some(String::new()));
        assert_eq!(r.version, Some(String::from("v1")));
    }
}
