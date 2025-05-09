// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use rivulet::prelude::*;
use std::str::FromStr;

#[test]
fn test_image_selector_parse_all_formats() {
    // Common image reference formats used in Docker
    let test_cases = [
        // Format: (input, expected_namespace, expected_repository, expected_tag, expected_digest)
        ("ubuntu", None, "ubuntu", None, None),
        ("ubuntu:20.04", None, "ubuntu", Some("20.04"), None),
        ("library/ubuntu", Some("library"), "ubuntu", None, None),
        ("docker.io/ubuntu", Some("docker.io"), "ubuntu", None, None),
        (
            "docker.io/library/ubuntu:22.04",
            Some("docker.io/library"),
            "ubuntu",
            Some("22.04"),
            None,
        ),
        (
            "quay.io/prometheus/alertmanager:v0.24.0",
            Some("quay.io/prometheus"),
            "alertmanager",
            Some("v0.24.0"),
            None,
        ),
        (
            "k8s.gcr.io/kube-apiserver:v1.23.0",
            Some("k8s.gcr.io"),
            "kube-apiserver",
            Some("v1.23.0"),
            None,
        ),
        (
            "ubuntu@sha256=ab01",
            None,
            "ubuntu",
            None,
            Some(("sha256", "ab01")),
        ),
        (
            "docker.io/library/ubuntu@sha256=ab01",
            Some("docker.io/library"),
            "ubuntu",
            None,
            Some(("sha256", "ab01")),
        ),
        (
            "docker.io/library/ubuntu:20.04@sha256=ab01",
            Some("docker.io/library"),
            "ubuntu",
            Some("20.04"),
            Some(("sha256", "ab01")),
        ),
    ];

    for (input, expected_namespace, expected_repository, expected_tag, expected_digest) in
        test_cases
    {
        let selector =
            ImageSelector::from_str(input).unwrap_or_else(|_| panic!("Failed to parse: {input}"));

        assert_eq!(
            selector.namespace.as_deref(),
            expected_namespace,
            "Namespace mismatch for {}",
            input
        );
        assert_eq!(
            selector.repository, expected_repository,
            "Repository mismatch for {}",
            input
        );
        assert_eq!(
            selector.tag.as_deref(),
            expected_tag,
            "Tag mismatch for {}",
            input
        );

        match (selector.digest, expected_digest) {
            (Some(digest), Some((expected_algo, expected_hash))) => {
                assert_eq!(
                    digest.algorithm, expected_algo,
                    "Digest algorithm mismatch for {}",
                    input
                );
                assert_eq!(
                    digest.hash, expected_hash,
                    "Digest hash mismatch for {}",
                    input
                );
            }
            (None, None) => {
                // Both are None, which is expected
            }
            _ => {
                panic!("Digest mismatch for {}", input);
            }
        }
    }
}

#[test]
fn test_image_selector_error_handling() {
    // Invalid image references and expected error types
    let test_cases = [
        ("", ImageSelectorParseError::MissingRepository),
        (":", ImageSelectorParseError::MissingRepository),
        ("/", ImageSelectorParseError::MissingRepository),
        ("namespace/", ImageSelectorParseError::MissingRepository),
        (
            "@digest",
            ImageSelectorParseError::InvalidDigestFormat("digest".to_string()),
        ),
        (
            "repo@",
            ImageSelectorParseError::InvalidDigestFormat("".to_string()),
        ),
        (
            "repo@invalid",
            ImageSelectorParseError::InvalidDigestFormat("invalid".to_string()),
        ),
        (
            "repo@=hash",
            ImageSelectorParseError::InvalidDigestFormat("=hash".to_string()),
        ),
        (
            "repo@algo=",
            ImageSelectorParseError::InvalidDigestFormat("algo=".to_string()),
        ),
    ];

    for (input, expected_error) in test_cases {
        let result = ImageSelector::from_str(input);
        assert!(result.is_err(), "Expected error for {}", input);
        assert_eq!(
            result.unwrap_err(),
            expected_error,
            "Error type mismatch for {}",
            input
        );
    }
}

#[test]
fn test_image_selector_practical_use_cases() {
    // Test some practical use cases for parsing image references

    // Case 1: Basic image with default tag
    let selector = ImageSelector::from_str("nginx").unwrap();
    assert_eq!(selector.repository, "nginx");
    assert_eq!(selector.tag, None); // Default tag will be applied later in the workflow

    // Case 2: Specific version of an image
    let selector = ImageSelector::from_str("python:3.9-slim").unwrap();
    assert_eq!(selector.repository, "python");
    assert_eq!(selector.tag, Some("3.9-slim".to_string()));

    // Case 3: Image from a private registry
    let selector = ImageSelector::from_str("registry.example.com/myapp:1.0").unwrap();
    assert_eq!(selector.namespace, Some("registry.example.com".to_string()));
    assert_eq!(selector.repository, "myapp");
    assert_eq!(selector.tag, Some("1.0".to_string()));

    // Case 4: Image with digest for immutable reference
    let selector = ImageSelector::from_str("ubuntu@sha256=a1b2c3d4e5f6").unwrap();
    assert_eq!(selector.repository, "ubuntu");
    assert!(selector.digest.is_some());
    let digest = selector.digest.unwrap();
    assert_eq!(digest.algorithm, "sha256");
    assert_eq!(digest.hash, "a1b2c3d4e5f6");

    // Case 5: Multi-level namespace
    let selector = ImageSelector::from_str("ghcr.io/owner/project/image:tag").unwrap();
    assert_eq!(
        selector.namespace,
        Some("ghcr.io/owner/project".to_string())
    );
    assert_eq!(selector.repository, "image");
    assert_eq!(selector.tag, Some("tag".to_string()));
}

#[test]
fn test_image_selector_component_display() {
    // Create some image selectors
    let simple = ImageSelector::from_str("nginx").unwrap();
    let with_tag = ImageSelector::from_str("nginx:latest").unwrap();
    let with_digest = ImageSelector::from_str("nginx@sha256=abcdef").unwrap();
    let with_namespace = ImageSelector::from_str("docker.io/library/nginx").unwrap();
    let complex = ImageSelector::from_str("docker.io/library/nginx:latest@sha256=abcdef").unwrap();

    // Demonstrate how to reconstruct the original reference from components
    let reconstruct = |selector: &ImageSelector| -> String {
        let mut result = String::new();

        // Add namespace if present
        if let Some(namespace) = &selector.namespace {
            result.push_str(namespace);
            result.push('/');
        }

        // Add repository (always present)
        result.push_str(&selector.repository);

        // Add tag if present
        if let Some(tag) = &selector.tag {
            result.push(':');
            result.push_str(tag);
        }

        // Add digest if present
        if let Some(digest) = &selector.digest {
            result.push('@');
            result.push_str(&digest.algorithm);
            result.push('=');
            result.push_str(&digest.hash);
        }

        result
    };

    // Verify reconstructed references
    assert_eq!(reconstruct(&simple), "nginx");
    assert_eq!(reconstruct(&with_tag), "nginx:latest");
    assert_eq!(reconstruct(&with_digest), "nginx@sha256=abcdef");
    assert_eq!(reconstruct(&with_namespace), "docker.io/library/nginx");
    assert_eq!(
        reconstruct(&complex),
        "docker.io/library/nginx:latest@sha256=abcdef"
    );
}

// EOF
