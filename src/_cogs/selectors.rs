use crate::_cogs::resources::Resource;
use regex::Regex;
use std::borrow::Borrow;
use std::sync::LazyLock;

#[derive(Default)]
pub struct Selector {
    pub group: Option<String>,
    pub version: Option<String>,
    pub plural: Option<String>,
    pub kind: Option<String>,
    pub singular: Option<String>,
    pub shortcut: Option<String>,
    pub category: Option<String>,
    pub any_name: Option<String>,
    pub fn_: Option<Box<dyn Fn(&Resource) -> bool>>,
}

// What we treat as an implied K8s resource version when parsing selectors.
// Matches: v1, v1alpha1, v1beta1; BUT NOT: velcro, v1alpha, v1gamma1.
static K8S_VERSION_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^v\\d+(?:(?:alpha|beta)\\d+)?$").unwrap());

/// A selector is a combination of criteria to apply to specific resources.
/// It can check if a specific resource matches, and select a bunch
/// of resources with some K8s-like logic (a priority of core v1 over others).
impl Selector {
    /// Make a builder to construct a selector with keyword parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use example1::Selector;
    /// let s = Selector::builder().plural("pods").category("cat1").build()
    /// assert_eq!(s.plural, "pods")
    /// assert_eq!(s.category, "cat1")
    /// ```
    pub fn builder() -> SelectorBuilder {
        SelectorBuilder::default()
    }

    /// The selector is specific if at least one of the certain names (plural,
    /// singular, kind, shortcuts) is defined. It is unspecific if all of these
    /// are undefined, probably because "any name" criterion is used.
    fn is_specific(&self) -> bool {
        self.plural.is_some()
            || self.kind.is_some()
            || self.singular.is_some()
            || self.shortcut.is_some()
    }

    /// Select all resources matching the criteria of a selector.
    ///
    /// Core v1 API group's priority is hard-coded in K8s and kubectl.
    /// We do the same. For example: whenever plural "pods" is specified,
    /// and "pods.v1" & "pods.v1beta1.metrics.k8s.io" are found, implicitly
    /// give priority to "v1" and hide the existence of non-"v1" groups.
    /// But not if they are specified by categories! -- In that case, keep all
    /// resources as is.
    ///
    /// # Examples
    ///
    /// ```
    /// use example1::{Resource, Selector};
    /// let r1 = Resource::builder().plural("deployments").build();
    /// let r2 = Resource::builder().plural("pods").build();
    /// let s = Selector::builder().any_name("pods").build();
    /// let r = s.select([&r1, &r2]);
    /// assert_eq!(r, vec![&r2]);
    /// ```
    pub fn select<I>(&self, resources: I) -> Vec<I::Item>
    where
        I: IntoIterator,
        I::Item: Borrow<Resource>,
    {
        let result: Vec<I::Item> = resources
            .into_iter()
            .filter(|r| self.matches(Borrow::<Resource>::borrow(r)))
            .collect();

        // Note: checking the match can be expensive as it contains a possible user-side callable.
        // We calculate the flags only once per resource, then split the results if needed.
        // The v1-check is cheap and deterministic, on the other hand, so we can repeat it.
        if self.is_specific() {
            let v1flags: Vec<bool> = result
                .iter()
                .map(|r| {
                    let r = Borrow::<Resource>::borrow(r);
                    r.group == Some(String::new()) && r.version == Some(String::from("v1"))
                })
                .collect();
            if v1flags.iter().any(|&b| b) {
                // Consume the results and generate a new vector by flags.
                result
                    .into_iter()
                    .zip(v1flags)
                    .filter_map(|(r, keep)| keep.then_some(r))
                    .collect()
            } else {
                result // no v1 resources
            }
        } else {
            result
        }
    }

    /// Checks if a specified resource matches the criteria of the selector.
    ///
    /// # Examples
    ///
    /// ```
    /// use example1::{Resource, Selector};
    /// let r = Resource::builder().plural("pods").build();
    /// let s = Selector::builder().any_name("pods").build();
    /// assert_eq!(s.matches(r), true);
    /// ```
    pub fn matches(&self, resource: &Resource) -> bool {
        if let Some(any_name) = self.any_name.as_ref() {
            if Some(any_name) != resource.plural.as_ref()
                && Some(any_name) != resource.kind.as_ref()
                && Some(any_name) != resource.singular.as_ref()
                && !resource
                    .shortcuts
                    .as_ref()
                    .is_some_and(|s| s.contains(any_name.as_str()))
            {
                return false;
            };
        }
        if let Some(group) = self.group.as_ref() {
            if Some(group) != resource.group.as_ref() {
                return false;
            };
        }
        if let Some(version) = self.version.as_ref() {
            if Some(version) != resource.version.as_ref() {
                return false;
            };
        }
        if let Some(plural) = self.plural.as_ref() {
            if Some(plural) != resource.plural.as_ref() {
                return false;
            }
        }
        if let Some(kind) = self.kind.as_ref() {
            if Some(kind) != resource.kind.as_ref() {
                return false;
            }
        }
        if let Some(singular) = self.singular.as_ref() {
            if Some(singular) != resource.singular.as_ref() {
                return false;
            }
        }
        if let Some(category) = self.category.as_ref() {
            if !resource
                .categories
                .as_ref()
                .is_some_and(|c| c.contains(category.as_str()))
            {
                return false;
            };
        }
        if let Some(shortcut) = self.shortcut.as_ref() {
            if !resource
                .shortcuts
                .as_ref()
                .is_some_and(|c| c.contains(shortcut.as_str()))
            {
                return false;
            };
        }
        if let Some(fn_) = self.fn_.as_ref() {
            if !fn_(resource) {
                return false;
            };
        }
        true
    }
}

#[derive(Default)]
pub struct SelectorBuilder {
    pub group: Option<String>,
    pub version: Option<String>,
    pub plural: Option<String>,
    pub kind: Option<String>,
    pub singular: Option<String>,
    pub shortcut: Option<String>,
    pub category: Option<String>,
    pub any_name: Option<String>,
    pub fn_: Option<Box<dyn Fn(&Resource) -> bool>>,
}

impl SelectorBuilder {
    pub fn build(self) -> Selector {
        Selector {
            group: self.group,
            version: self.version,
            plural: self.plural,
            kind: self.kind,
            singular: self.singular,
            shortcut: self.shortcut,
            category: self.category,
            any_name: self.any_name,
            fn_: self.fn_,
        }
    }

    pub fn any_name(mut self, value: &str) -> Self {
        self.any_name = Some(value.to_owned());
        self
    }

    pub fn group(mut self, value: &str) -> Self {
        self.group = Some(value.to_owned());
        self
    }

    pub fn version(mut self, value: &str) -> Self {
        self.version = Some(value.to_owned());
        self
    }

    pub fn plural(mut self, value: &str) -> Self {
        self.plural = Some(value.to_owned());
        self
    }

    pub fn kind(mut self, value: &str) -> Self {
        self.kind = Some(value.to_owned());
        self
    }

    pub fn singular(mut self, value: &str) -> Self {
        self.singular = Some(value.to_owned());
        self
    }

    pub fn shortcut(mut self, value: &str) -> Self {
        self.shortcut = Some(value.to_owned());
        self
    }

    pub fn category(mut self, value: &str) -> Self {
        self.category = Some(value.to_owned());
        self
    }

    pub fn fn_(mut self, value: Box<dyn Fn(&Resource) -> bool>) -> Self {
        self.fn_ = Some(value);
        self
    }
}

/// A trick to overload Selector::parse() for variadic 1-3 strings as arguments
/// (not the extra parentheses for tuples):
/// ```
/// let s1 = Selector::parse("kopfexamples.v1.kopf.dev")
/// let s2 = Selector::parse(("kopfexamples.v1.kopf.dev",))
/// let s3 = Selector::parse(("kopf.dev/v1", "kopfexamples"))
/// let s4 = Selector::parse(("kopf.dev", "v1", "kopfexamples"))
/// ```
trait SelectorBuilderParseArgs<T> {
    /// Parses a string or a literal into a structured selector. In all cases,
    /// the parsed name goes into the "any name" criterion, not to the plural
    /// name. It is later checked against plural, singular, kind, and shortcuts.
    ///
    /// Supported notations:
    /// - pods
    /// - pods.v1
    /// - kex.v1
    /// - kex.kopf.dev
    /// - kex.v1.kopf.dev
    ///
    /// For versions, anything that is a typical K8s version is accepted:
    /// - v123
    /// - v123alpha456
    /// - v123beta456
    /// But not other v-starting words, such as "vvv" or "v1gamma1".
    ///
    /// # Complexity
    ///
    /// It performs at most 2 split-once calls on a string, and 1 regex check.
    ///
    /// # Example
    ///
    /// ```
    /// use example1::{Resource, Selector};
    /// let s = Selector::parse("kopfexamples.kopf.dev");
    /// let r = Resource::builder().group("kopf.dev").version("v1").plural("kopfexamples").build()
    /// assert_eq!(s.matches(r), true)
    /// ```
    fn parse(self, args: T) -> Self;
}

// Selector::builder()::parse("kopfexamples.kopf.dev").build()
impl SelectorBuilderParseArgs<&str> for SelectorBuilder {
    fn parse(self, value: &str) -> Self {
        match value.split_once('.') {
            // No dots, a single name, the most common case: e.g., "kex", "pods".
            None => self.any_name(value),

            // 1+ dots, so there is some structured name requiring an interpretation.
            Some((any_name, group_version)) => self.parse((group_version, any_name)),
        }
    }
}

// Selector::builder()::parse(("kopfexamples.kopf.dev",)).build()
impl SelectorBuilderParseArgs<(&str,)> for SelectorBuilder {
    fn parse(self, args: (&str,)) -> Self {
        let (arg,) = args;
        self.parse(arg)
    }
}

// Selector::builder()::parse(("kopf.dev/v1", "kopfexamples")).build()
impl SelectorBuilderParseArgs<(&str, &str)> for SelectorBuilder {
    fn parse(self, args: (&str, &str)) -> Self {
        let (group_version, any_name) = args;
        // todo: parse kopf.dev/v1 with slashes, if any (and different order)

        // Either v1, or v1.kopf.dev, but we need the 1st item only.
        let (maybe_version, leftover) = match group_version.split_once('.') {
            None => (group_version, None),
            Some((version, leftover)) => (version, Some(leftover)),
        };
        if K8S_VERSION_PATTERN.is_match(maybe_version) {
            match (maybe_version, leftover) {
                // v1
                ("v1", None) => self.any_name(any_name).version(maybe_version).group(""),

                // v2, v1alpha1
                (_, None) => self.any_name(any_name).version(maybe_version),

                // v1.kopf.dev
                (_, Some(group)) => self.any_name(any_name).version(maybe_version).group(group),
            }
        } else {
            // vvv.kopf.dev (no certain version)
            self.any_name(any_name).group(group_version)
        }
    }
}

// Selector::builder()::parse(("kopf.dev", "v1", "kopfexamples")).build()
impl SelectorBuilderParseArgs<(&str, &str, &str)> for SelectorBuilder {
    fn parse(self, args: (&str, &str, &str)) -> Self {
        let (group, version, any_name) = args;
        self.group(group).version(version).any_name(any_name)
    }
}

trait SelectorParseArgs<T> {
    /// A shortcut for the builder with only the parsing, no other criteria.
    /// These are equivalent:
    /// ```
    /// let s1 = Selector::parse("kopfexamples.kopf.dev");
    /// let s2 = Selector::builder().parse("kopfexamples.kopf.dev").build();
    /// ```
    fn parse(args: T) -> Self;
}

// Selector::parse("kopfexamples.kopf.dev")
impl SelectorParseArgs<&str> for Selector {
    fn parse(args: &str) -> Self {
        Self::builder().parse(args).build()
    }
}

// Selector::parse(("kopfexamples.kopf.dev",))
impl SelectorParseArgs<(&str,)> for Selector {
    fn parse(args: (&str,)) -> Self {
        Self::builder().parse(args).build()
    }
}

// Selector::parse(("kopf.dev/v1", "kopfexamples"))
impl SelectorParseArgs<(&str, &str)> for Selector {
    fn parse(args: (&str, &str)) -> Self {
        Self::builder().parse(args).build()
    }
}

// Selector::parse(("kopf.dev", "v1", "kopfexamples"))
impl SelectorParseArgs<(&str, &str, &str)> for Selector {
    fn parse(args: (&str, &str, &str)) -> Self {
        Self::builder().parse(args).build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn resource() -> Resource {
        Resource::builder()
            .group("kopf.dev")
            .version("v1")
            .plural("kopfexamples")
            .singular("kopfexample")
            .kind("KopfExample")
            .shortcuts(["kex"])
            .categories(["cat"])
            .build()
    }

    #[rstest]
    #[case("kopfexamples"            , "--"         , "--", "kopfexamples")]
    #[case("kopfexamples.vv"         , "vv"         , "--", "kopfexamples")]
    #[case("kopfexamples.vv.kopf.dev", "vv.kopf.dev", "--", "kopfexamples")]
    #[case("kopfexamples.v1"         , ""           , "v1", "kopfexamples")]
    #[case("kopfexamples.v2"         , "--"         , "v2", "kopfexamples")]
    #[case("kopfexamples.v1.kopf.dev", "kopf.dev"   , "v1", "kopfexamples")]
    fn parsing_str(
        #[case] text: &str,
        #[case] group: &str,
        #[case] version: &str,
        #[case] any_name: &str,
    ) {
        let s = Selector::parse(text);
        assert_eq!(s.group, if group == "--" {None} else {Some(group.to_string())});
        assert_eq!(s.version, if version == "--" {None} else {Some(version.to_string())});
        assert_eq!(s.any_name, if any_name == "--" {None} else {Some(any_name.to_string())});

        // Should be the same under the same inputs:
        let s2 = Selector::parse((text,));
        assert_eq!(s2.group, s.group);
        assert_eq!(s2.version, s.version);
        assert_eq!(s2.any_name, s.any_name);
    }

    #[rstest]
    #[case("vv"         , "kopfexamples", "vv"         , "--", "kopfexamples")]
    #[case("vv.kopf.dev", "kopfexamples", "vv.kopf.dev", "--", "kopfexamples")]
    #[case("v1"         , "kopfexamples", ""           , "v1", "kopfexamples")]
    #[case("v2"         , "kopfexamples", "--"         , "v2", "kopfexamples")]
    #[case("v1.kopf.dev", "kopfexamples", "kopf.dev"   , "v1", "kopfexamples")]
    fn parsing_2str_tuple(#[case] arg1: &str, #[case] arg2: &str, #[case] group: &str, #[case] version: &str, #[case] any_name: &str) {
        let s = Selector::parse((arg1, arg2));
        assert_eq!(s.group, if group == "--" {None} else {Some(group.to_string())});
        assert_eq!(s.version, if version == "--" {None} else {Some(version.to_string())});
        assert_eq!(s.any_name, if any_name == "--" {None} else {Some(any_name.to_string())});
    }

    #[rstest]
    #[case(""        , "v1", "kopfexamples", ""           , "v1", "kopfexamples")]
    #[case(""        , "v2", "kopfexamples", ""           , "v2", "kopfexamples")]
    #[case("kopf.dev", "v1", "kopfexamples", "kopf.dev"   , "v1", "kopfexamples")]
    fn parsing_3str_tuple(#[case] arg1: &str, #[case] arg2: &str, #[case] arg3: &str, #[case] group: &str, #[case] version: &str, #[case] any_name: &str) {
        let s = Selector::parse((arg1, arg2, arg3));
        assert_eq!(s.group, if group == "--" {None} else {Some(group.to_string())});
        assert_eq!(s.version, if version == "--" {None} else {Some(version.to_string())});
        assert_eq!(s.any_name, if any_name == "--" {None} else {Some(any_name.to_string())});
    }

    #[rstest]
    #[case("kopfexamples", true)]
    #[case("kopfexample", true)]
    #[case("KopfExample", true)]
    #[case("kex", true)]
    #[case("cat", false)]
    #[case("pod", false)]
    fn matching_by_any_name(resource: Resource, #[case] value: &str, #[case] expected: bool) {
        let s = Selector::builder().any_name(value).build();
        assert_eq!(s.matches(&resource), expected);
    }

    #[rstest]
    #[case("kopf.dev", true)]
    #[case("blah", false)]
    #[case("", false)]
    fn matching_by_group(resource: Resource, #[case] value: &str, #[case] expected: bool) {
        let s = Selector::builder().group(value).build();
        assert_eq!(s.matches(&resource), expected);
    }

    #[rstest]
    #[case("v1", true)]
    #[case("v2", false)]
    fn matching_by_version(resource: Resource, #[case] value: &str, #[case] expected: bool) {
        let s = Selector::builder().version(value).build();
        assert_eq!(s.matches(&resource), expected);
    }

    #[rstest]
    #[case("kopfexamples", true)]
    #[case("kopfexample", false)]
    fn matching_by_plural(resource: Resource, #[case] value: &str, #[case] expected: bool) {
        let s = Selector::builder().plural(value).build();
        assert_eq!(s.matches(&resource), expected);
    }

    #[rstest]
    #[case("KopfExample", true)]
    #[case("kopfexample", false)]
    fn matching_by_kind(resource: Resource, #[case] value: &str, #[case] expected: bool) {
        let s = Selector::builder().kind(value).build();
        assert_eq!(s.matches(&resource), expected);
    }

    #[rstest]
    #[case("kopfexample", true)]
    #[case("KopfExample", false)]
    fn matching_by_singular(resource: Resource, #[case] value: &str, #[case] expected: bool) {
        let s = Selector::builder().singular(value).build();
        assert_eq!(s.matches(&resource), expected);
    }

    #[rstest]
    #[case("kex", true)]
    #[case("pod", false)]
    fn matching_by_shortcut(resource: Resource, #[case] value: &str, #[case] expected: bool) {
        let s = Selector::builder().shortcut(value).build();
        assert_eq!(s.matches(&resource), expected);
    }

    #[rstest]
    #[case("cat", true)]
    #[case("xyz", false)]
    fn matching_by_category(resource: Resource, #[case] value: &str, #[case] expected: bool) {
        let s = Selector::builder().category(value).build();
        assert_eq!(s.matches(&resource), expected);
    }

    #[rstest]
    fn selecting_normal() {
        let r1 = Resource::builder().plural("plural1").build();
        let r2 = Resource::builder().plural("plural2").build();
        let s = Selector::builder().plural("plural1").build();
        let results = s.select([&r1, &r2]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], &r1);
    }

    #[rstest]
    fn selecting_v1_priority() {
        let r1 = Resource::builder().group("").version("v1").plural("pods").build();
        let r2 = Resource::builder().group("g").version("v1").plural("pods").build();
        let s = Selector::builder().plural("pods").build();
        let results = s.select([&r1, &r2]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], &r1);
    }
}
