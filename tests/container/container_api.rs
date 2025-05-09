// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use rivulet::prelude::*;

#[test]
fn test_container_creation_from_image_references() {
    // Test creating containers from different image reference formats
    let images = [
        "nginx",
        "nginx:latest",
        "python:3.9-slim",
        "docker.io/library/redis:6.2",
        "codeberg.org/forgejo/forgejo:10.0.1",
        "ubuntu@sha256=a1b2c3d4e5f6",
    ];

    for image in images {
        let container = Container::from(image);

        // Extract the base name for comparison
        let base_name = image.split('@').next().unwrap().split(':').next().unwrap();

        // Verify the container was created with the correct image reference
        let container_guard = container.read().unwrap();
        match &container_guard.base {
            ContainerBase::External(s) => {
                assert!(
                    s.repository == base_name
                        || base_name.ends_with(&s.repository)
                        || s.namespace.as_ref().is_some_and(|n| n.contains(base_name))
                );
            }
            _ => panic!("Expected ContainerBase::External"),
        }
    }
}

#[test]
fn test_container_from_custom_selector() {
    // Manually construct an ImageSelector
    let selector = ImageSelector {
        namespace: Some("custom.registry".to_string()),
        repository: "myapp".to_string(),
        tag: Some("v1.0".to_string()),
        digest: None,
    };

    // Create a container from the selector
    let container = Container::from(selector);

    // Verify the container has the correct image reference
    let container_guard = container.read().unwrap();
    assert!(matches!(container_guard.base,
        ContainerBase::External(ref s) if
            s.namespace == Some("custom.registry".to_string()) &&
            s.repository == "myapp" &&
            s.tag == Some("v1.0".to_string())
    ));
}

#[test]
fn test_container_parse_errors() {
    // Test various invalid container references
    let invalid_refs = [
        "",          // Empty string
        ":",         // Missing repository and tag
        "@",         // Missing repository and digest
        "/",         // Missing repository
        "registry/", // Missing repository
        "@invalid",  // Invalid digest format
        "@algo=",    // Empty hash
        "@=value",   // Empty algorithm
    ];

    for invalid_ref in invalid_refs {
        // Using parse directly to avoid panic
        let result = invalid_ref.parse::<Container>();
        assert!(
            result.is_err(),
            "Expected error for invalid ref: {}",
            invalid_ref
        );
    }
}

#[test]
#[should_panic(expected = "Failed to parse image reference")]
fn test_from_string_panic() {
    // This should panic with an appropriate message
    let _container = Container::from("invalid@digest");
}

// EOF
