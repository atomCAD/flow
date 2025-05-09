// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Errors that can occur when parsing Docker image references.
///
/// These errors are returned when attempting to parse an invalid image reference
/// string into an [`ImageSelector`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ImageSelectorParseError {
    /// Returned when the repository name is missing in the image reference.
    ///
    /// Examples of inputs that trigger this error:
    /// - Empty string: `""`
    /// - Only namespace: `"namespace/"`
    /// - Only tag: `":tag"`
    /// - Only digest: `"@sha256=hash"`
    #[error("Missing image repository")]
    MissingRepository,

    /// Returned when the digest format is invalid.
    /// The enclosed string is the invalid digest from the input.
    ///
    /// The digest format must be `algorithm=hash`, where both algorithm and hash are non-empty.
    /// Examples of inputs that trigger this error:
    /// - Missing equals sign: `"ubuntu@sha256"`
    /// - Empty algorithm: `"ubuntu@=hash"`
    /// - Empty hash: `"ubuntu@sha256="`
    #[error("Invalid digest format: {0}")]
    InvalidDigestFormat(String),
}

/// Represents a content-addressable digest for an image.
///
/// Docker image digests consist of an algorithm and a hash value, typically
/// in the format `algorithm=hash`. The most common algorithm is SHA-256.
///
/// # Examples
///
/// A typical image digest might look like:
/// ```
/// use rivulet::container::ImageDigest;
///
/// let digest = ImageDigest {
///     algorithm: "sha256".to_string(),
///     hash: "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageDigest {
    /// The hashing algorithm used (e.g., "sha256")
    pub algorithm: String,

    /// The hash value (e.g., "a1b2c3d4e5f6...")
    pub hash: String,
}

/// Represents a parsed Docker image reference.
///
/// This struct parses and stores the components of a Docker image reference,
/// which follows the pattern: `[registry/][user/organization/]repository[:tag][@algorithm=hash]`
///
/// # Examples
///
/// Basic usage:
/// ```
/// use std::str::FromStr;
/// use rivulet::container::ImageSelector;
///
/// // Parse a simple image reference
/// let selector = ImageSelector::from_str("nginx:latest").unwrap();
/// assert_eq!(selector.repository, "nginx");
/// assert_eq!(selector.tag, Some("latest".to_string()));
///
/// // Parse a more complex image reference with registry and namespace
/// let selector = ImageSelector::from_str("docker.io/library/ubuntu:20.04").unwrap();
/// assert_eq!(selector.namespace, Some("docker.io/library".to_string()));
/// assert_eq!(selector.repository, "ubuntu");
/// assert_eq!(selector.tag, Some("20.04".to_string()));
///
/// // Parse an image reference with digest
/// let selector = ImageSelector::from_str("ubuntu@sha256=a1b2c3d4e5f6").unwrap();
/// assert_eq!(selector.repository, "ubuntu");
/// let digest = selector.digest.unwrap();
/// assert_eq!(digest.algorithm, "sha256");
/// assert_eq!(digest.hash, "a1b2c3d4e5f6");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageSelector {
    /// Optional namespace (includes registry if present).
    ///
    /// Examples:
    /// - "docker.io/library"
    /// - "ghcr.io/user"
    /// - "codeberg.org/forgejo"
    pub namespace: Option<String>,

    /// Repository name (required).
    ///
    /// This is the only required component of an image reference.
    pub repository: String,

    /// Optional tag reference.
    ///
    /// Examples:
    /// - "latest"
    /// - "3.9-slim"
    /// - "v1.0.0"
    pub tag: Option<String>,

    /// Optional digest reference.
    ///
    /// This provides content-addressable references to specific image versions.
    pub digest: Option<ImageDigest>,
}

impl ImageSelector {
    /// Parse a string reference into an ImageSelector.
    ///
    /// This method parses a Docker image reference string into its components:
    /// namespace, repository, tag, and digest.
    ///
    /// # Arguments
    ///
    /// * `s` - The image reference string to parse
    ///
    /// # Returns
    ///
    /// A `Result` containing either the parsed `ImageSelector` or an `ImageSelectorParseError`
    ///
    /// # Examples
    ///
    /// ```
    /// use rivulet::container::ImageSelector;
    ///
    /// // Parse a simple image name
    /// let selector = ImageSelector::parse("ubuntu").unwrap();
    ///
    /// // Parse an image with tag
    /// let selector = ImageSelector::parse("nginx:latest").unwrap();
    ///
    /// // Parse an image with namespace and tag
    /// let selector = ImageSelector::parse("docker.io/library/redis:6.2").unwrap();
    ///
    /// // Parse an image with digest
    /// let selector = ImageSelector::parse("ubuntu@sha256=a1b2c3d4e5f6").unwrap();
    /// ```
    pub fn parse(s: &str) -> Result<Self, ImageSelectorParseError> {
        // Check for digest (@)
        let (s, digest) = match s.split_once('@') {
            Some((rest, digest_ref)) => {
                if let Some((algo, hash)) = digest_ref.split_once('=') {
                    if algo.is_empty() || hash.is_empty() {
                        return Err(ImageSelectorParseError::InvalidDigestFormat(
                            digest_ref.to_string(),
                        ));
                    }
                    (
                        rest,
                        Some(ImageDigest {
                            algorithm: algo.to_string(),
                            hash: hash.to_string(),
                        }),
                    )
                } else {
                    return Err(ImageSelectorParseError::InvalidDigestFormat(
                        digest_ref.to_string(),
                    ));
                }
            }
            None => (s, None),
        };

        // Check for tag (:)
        let (s, tag) = match s.rsplit_once(':') {
            Some((rest, tag)) => (rest, Some(tag.to_string())),
            None => (s, None),
        };

        // Check for namespace (/)
        let (namespace, s) = match s.rsplit_once('/') {
            Some((namespace, rest)) => (Some(namespace.to_string()), rest),
            None => (None, s),
        };

        // Repository is required
        if s.is_empty() {
            return Err(ImageSelectorParseError::MissingRepository);
        }

        Ok(ImageSelector {
            namespace,
            repository: s.to_string(),
            tag,
            digest,
        })
    }
}

impl FromStr for ImageSelector {
    type Err = ImageSelectorParseError;

    /// Parse a string into an ImageSelector using the `FromStr` trait.
    ///
    /// This allows using the standard library's `parse()` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use rivulet::container::ImageSelector;
    ///
    /// let selector: ImageSelector = "nginx:latest".parse().unwrap();
    /// assert_eq!(selector.repository, "nginx");
    /// assert_eq!(selector.tag, Some("latest".to_string()));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for ImageSelector {
    type Error = ImageSelectorParseError;

    /// Convert a string reference to an ImageSelector using the `TryFrom` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::convert::TryFrom;
    /// use rivulet::container::ImageSelector;
    ///
    /// let selector = ImageSelector::try_from("nginx:latest").unwrap();
    /// assert_eq!(selector.repository, "nginx");
    /// assert_eq!(selector.tag, Some("latest".to_string()));
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

/// Represents the base of a container, which can be either an external image reference
/// or a reference to another container.
///
/// This enum is used internally by the [`Container`] struct to represent
/// either an external image reference (like "nginx:latest") or a reference
/// to another container (for layering/nesting).
#[derive(Debug, Clone)]
pub enum ContainerBase {
    /// An external image reference.
    External(ImageSelector),

    /// A reference to another container (for nesting/layering).
    Internal(Arc<RwLock<Container>>),
}

impl From<ImageSelector> for ContainerBase {
    /// Convert an ImageSelector into a ContainerBase.
    ///
    /// This creates an External container base from an image selector.
    fn from(selector: ImageSelector) -> Self {
        Self::External(selector)
    }
}

impl From<Arc<RwLock<Container>>> for ContainerBase {
    /// Convert a container reference into a ContainerBase.
    ///
    /// This creates an Internal container base from a container reference.
    fn from(container: Arc<RwLock<Container>>) -> Self {
        Self::Internal(container)
    }
}

impl TryFrom<&str> for ContainerBase {
    type Error = ImageSelectorParseError;

    /// Try to convert a string into a ContainerBase.
    ///
    /// This parses the string as an image reference and creates an External container base.
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let selector = ImageSelector::try_from(s)?;
        Ok(ContainerBase::External(selector))
    }
}

/// Represents a container that can be based on either an external image or another container.
///
/// Containers can be created from:
/// - Docker image references (strings like "nginx:latest")
/// - Image selectors (parsed image references)
/// - Other containers (for nesting/layering)
///
/// # Examples
///
/// Creating containers from image references:
/// ```
/// use rivulet::container::Container;
///
/// // Create a container from a string image reference
/// let nginx_container = Container::from("nginx:latest");
///
/// // Create a container from a more complex image reference
/// let ubuntu_container = Container::from("docker.io/library/ubuntu:20.04");
/// ```
///
/// Creating containers that reference other containers:
/// ```
/// use rivulet::container::Container;
///
/// // Create a base container
/// let base_container = Container::from("alpine:latest");
///
/// // Create a container that references the base container
/// let derived_container = Container::from(&base_container);
/// ```
#[derive(Debug, Clone)]
pub struct Container {
    /// The base of this container (either an external image or a reference to another container).
    pub base: ContainerBase,
}

impl FromStr for Container {
    type Err = ImageSelectorParseError;

    /// Parse a string into a Container using the `FromStr` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use rivulet::container::Container;
    ///
    /// let container: Container = "nginx:latest".parse().unwrap();
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let base = ContainerBase::try_from(s)?;
        Ok(Self { base })
    }
}

// Special implementation for &str to allow Container::from("image:tag") syntax
impl From<&str> for Container {
    /// Create a Container from a string image reference.
    ///
    /// This method will panic if the string is not a valid image reference.
    /// If you need to handle parsing errors, use `Container::from_str()` instead.
    ///
    /// # Panics
    ///
    /// This method will panic if the string cannot be parsed as a valid image reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivulet::container::Container;
    ///
    /// // Create a container from a string image reference
    /// let container = Container::from("nginx:latest");
    /// ```
    fn from(image_ref: &str) -> Self {
        match Container::from_str(image_ref) {
            Ok(container) => container,
            Err(e) => panic!("Failed to parse image reference '{image_ref}': {e}"),
        }
    }
}

impl From<ImageSelector> for Container {
    /// Create a Container from an ImageSelector.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use rivulet::container::{Container, ImageSelector};
    ///
    /// let selector = ImageSelector::from_str("nginx:latest").unwrap();
    /// let container = Container::from(selector);
    /// ```
    fn from(selector: ImageSelector) -> Self {
        Self {
            base: ContainerBase::External(selector),
        }
    }
}

impl From<&Arc<RwLock<Container>>> for Container {
    /// Create a Container that references another Container.
    ///
    /// This is used for container nesting/layering, where one container
    /// is based on another container.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivulet::container::Container;
    ///
    /// // Create a base container
    /// let base_container = Container::from("alpine:latest");
    ///
    /// // Create a container that references the base container
    /// let derived_container = Container::from(&base_container);
    /// ```
    fn from(container: &Arc<RwLock<Container>>) -> Self {
        Self {
            base: ContainerBase::Internal(container.clone()),
        }
    }
}

impl Container {
    /// Create a wrapped Container instance from any type that can be converted to a Container.
    ///
    /// This is the primary factory method for creating containers. It returns the container
    /// wrapped in an `Arc<RwLock>` for thread-safe reference counting and mutability.
    ///
    /// # Arguments
    ///
    /// * `value` - Any value that can be converted into a Container
    ///
    /// # Returns
    ///
    /// The container wrapped in an `Arc<RwLock>`
    ///
    /// # Examples
    ///
    /// From a string image reference:
    /// ```
    /// use rivulet::container::Container;
    ///
    /// // Create a container from a string image reference
    /// let container = Container::from("nginx:latest");
    /// ```
    ///
    /// From an ImageSelector:
    /// ```
    /// use std::str::FromStr;
    /// use rivulet::container::{Container, ImageSelector};
    ///
    /// let selector = ImageSelector::from_str("nginx:latest").unwrap();
    /// let container = Container::from(selector);
    /// ```
    ///
    /// From another container (creating a nested container):
    /// ```
    /// use rivulet::container::Container;
    ///
    /// // Create a base container
    /// let base_container = Container::from("alpine:latest");
    ///
    /// // Create a container that references the base container
    /// let derived_container = Container::from(&base_container);
    /// ```
    pub fn from<T: Into<Self>>(value: T) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(value.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ImageSelector parsing tests
    mod image_selector_parsing {
        use super::*;

        #[test]
        fn test_simple_repository() {
            let selector = ImageSelector::parse("ubuntu").unwrap();
            assert!(matches!(selector,
                ImageSelector {
                    namespace: None,
                    repository: r,
                    tag: None,
                    digest: None,
                } if r == "ubuntu"
            ));
        }

        #[test]
        fn test_with_tag() {
            let selector = ImageSelector::parse("python:3.9-slim").unwrap();
            assert!(matches!(selector,
                ImageSelector {
                    namespace: None,
                    repository: r,
                    tag: Some(t),
                    digest: None,
                } if r == "python" && t == "3.9-slim"
            ));
        }

        #[test]
        fn test_with_namespace() {
            let selector = ImageSelector::parse("docker.io/library/redis").unwrap();
            assert!(matches!(selector,
                ImageSelector {
                    namespace: Some(n),
                    repository: r,
                    tag: None,
                    digest: None,
                } if n == "docker.io/library" && r == "redis"
            ));
        }

        #[test]
        fn test_with_namespace_and_tag() {
            let selector = ImageSelector::parse("docker.io/library/redis:6.2").unwrap();
            assert!(matches!(selector,
                ImageSelector {
                    namespace: Some(n),
                    repository: r,
                    tag: Some(t),
                    digest: None,
                } if n == "docker.io/library" && r == "redis" && t == "6.2"
            ));
        }

        #[test]
        fn test_with_digest() {
            let selector = ImageSelector::parse("ubuntu@sha256=a1b2c3d4e5f6").unwrap();
            assert!(matches!(selector,
                ImageSelector {
                    namespace: None,
                    repository: r,
                    tag: None,
                    digest: Some(d),
                } if r == "ubuntu" && d.algorithm == "sha256" && d.hash == "a1b2c3d4e5f6"
            ));
        }

        #[test]
        fn test_complex_image_reference() {
            let selector = ImageSelector::parse("codeberg.org/forgejo/forgejo:10.0.1").unwrap();
            assert!(matches!(selector,
                ImageSelector {
                    namespace: Some(n),
                    repository: r,
                    tag: Some(t),
                    digest: None,
                } if n == "codeberg.org/forgejo" && r == "forgejo" && t == "10.0.1"
            ));
        }

        #[test]
        fn test_multi_level_namespace() {
            let selector = ImageSelector::parse("docker.io/library/user/repo:tag").unwrap();
            assert!(matches!(selector,
                ImageSelector {
                    namespace: Some(n),
                    repository: r,
                    tag: Some(t),
                    digest: None,
                } if n == "docker.io/library/user" && r == "repo" && t == "tag"
            ));
        }

        #[test]
        fn test_with_tag_and_digest() {
            // When both tag and digest are present, only digest should be used
            let selector = ImageSelector::parse("ubuntu:latest@sha256=a1b2c3d4e5f6").unwrap();
            assert!(matches!(selector,
                ImageSelector {
                    namespace: None,
                    repository: r,
                    tag: Some(t),
                    digest: Some(d),
                } if r == "ubuntu" && t == "latest" && d.algorithm == "sha256" && d.hash == "a1b2c3d4e5f6"
            ));
        }
    }

    // ImageSelector error tests
    mod image_selector_errors {
        use super::*;

        #[test]
        fn test_invalid_digest_formats() {
            // Empty algorithm
            let result = ImageSelector::parse("ubuntu@=hash");
            assert!(matches!(result,
                Err(ImageSelectorParseError::InvalidDigestFormat(s)) if s == "=hash"
            ));

            // Empty hash
            let result = ImageSelector::parse("ubuntu@sha256=");
            assert!(matches!(result,
                Err(ImageSelectorParseError::InvalidDigestFormat(s)) if s == "sha256="
            ));

            // No equals sign
            let result = ImageSelector::parse("ubuntu@sha256");
            assert!(matches!(result,
                Err(ImageSelectorParseError::InvalidDigestFormat(s)) if s == "sha256"
            ));

            // Empty digest
            let result = ImageSelector::parse("ubuntu@");
            assert!(matches!(result,
                Err(ImageSelectorParseError::InvalidDigestFormat(s)) if s.is_empty()
            ));
        }

        #[test]
        fn test_missing_repository() {
            let inputs = [
                // Empty string
                "",
                // Only namespace
                "namespace/",
                // Multiple trailing slashes
                "namespace///",
                // Only tag
                ":tag",
                // Only digest
                "@sha256=hash",
            ];

            for input in inputs {
                let result = ImageSelector::parse(input);
                assert_eq!(result, Err(ImageSelectorParseError::MissingRepository));
            }
        }
    }

    // Trait implementation tests
    mod trait_implementations {
        use super::*;

        #[test]
        fn test_image_selector_from_str() {
            let selector: ImageSelector = "nginx:latest".parse().unwrap();
            assert!(matches!(selector,
                ImageSelector {
                    namespace: None,
                    repository: r,
                    tag: Some(t),
                    digest: None,
                } if r == "nginx" && t == "latest"
            ));
        }

        #[test]
        fn test_image_selector_try_from() {
            let selector = <ImageSelector as TryFrom<&str>>::try_from("redis:6.2").unwrap();
            assert!(matches!(selector,
                ImageSelector {
                    namespace: None,
                    repository: r,
                    tag: Some(t),
                    digest: None,
                } if r == "redis" && t == "6.2"
            ));
        }

        #[test]
        fn test_container_base_from_image_selector() {
            let selector = ImageSelector::parse("nginx:latest").unwrap();
            let base = ContainerBase::from(selector);

            assert!(matches!(base,
                ContainerBase::External(s) if s.repository == "nginx" && s.tag == Some("latest".to_string())
            ));
        }

        #[test]
        fn test_container_base_from_container() {
            let selector = ImageSelector::parse("redis:6.2").unwrap();
            let container = Container::from(selector);

            let base = ContainerBase::from(container.clone());
            assert!(matches!(base,
                ContainerBase::Internal(arc) if Arc::ptr_eq(&arc, &container)
            ));
        }

        #[test]
        fn test_container_base_try_from_str() {
            let base = ContainerBase::try_from("nginx:latest").unwrap();
            assert!(matches!(base,
                ContainerBase::External(s) if s.repository == "nginx" && s.tag == Some("latest".to_string())
            ));
        }
    }

    // Container API tests
    mod container_api {
        use super::*;

        #[test]
        fn test_from_str() {
            // Test FromStr trait implementation
            let container: Container = "nginx:latest".parse().unwrap();
            assert!(matches!(container.base,
                ContainerBase::External(s) if s.repository == "nginx" && s.tag == Some("latest".to_string())
            ));
        }

        #[test]
        fn test_from_str_error() {
            // Test error handling for invalid image references
            let result: Result<Container, _> = "ubuntu@invalid".parse();
            assert!(matches!(result.unwrap_err(),
                ImageSelectorParseError::InvalidDigestFormat(s) if s == "invalid"
            ));
        }

        #[test]
        fn test_from_image_selector() {
            // Test Container::from with ImageSelector
            let selector = ImageSelector::parse("nginx:latest").unwrap();
            let container = Container::from(selector);

            let guard = container.read().unwrap();
            assert!(matches!(guard.base,
                ContainerBase::External(ref s) if s.repository == "nginx" && s.tag == Some("latest".to_string())
            ));
        }

        #[test]
        fn test_from_container_reference() {
            // Test Container::from with another container reference
            let selector = ImageSelector::parse("redis:6.2").unwrap();
            let container1 = Container::from(selector);
            let container2 = Container::from(&container1);

            // Verify container2 references container1
            let guard = container2.read().unwrap();
            assert!(matches!(guard.base,
                ContainerBase::Internal(ref arc) if Arc::ptr_eq(arc, &container1)
            ));

            // Also verify original selector data is accessible
            assert!(matches!(guard.base,
                ContainerBase::Internal(ref arc) if
                    matches!(arc.read().unwrap().base,
                        ContainerBase::External(ref s) if s.repository == "redis" && s.tag == Some("6.2".to_string())
                    )
            ));
        }

        #[test]
        fn test_from_string() {
            // Test Container::from with string
            let container = Container::from("nginx:latest");

            let guard = container.read().unwrap();
            assert!(matches!(guard.base,
                ContainerBase::External(ref s) if s.repository == "nginx" && s.tag == Some("latest".to_string())
            ));
        }

        #[test]
        #[should_panic(expected = "Failed to parse image reference")]
        fn test_from_string_panic() {
            // This should panic with an appropriate message
            let _container = Container::from("invalid@digest");
        }
    }
}

// EOF
