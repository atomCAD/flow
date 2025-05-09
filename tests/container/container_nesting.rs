// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use rivulet::prelude::*;

#[test]
fn test_container_chaining() {
    // Create a chain of containers referencing each other
    let base_container = Container::from("nginx:latest");
    let middle_container = Container::from(&base_container);
    let top_container = Container::from(&middle_container);

    // Verify the chain of references
    let top_guard = top_container.read().unwrap();
    assert!(matches!(top_guard.base,
        ContainerBase::Internal(ref middle_arc) if {
            let middle_guard = middle_arc.read().unwrap();
            matches!(middle_guard.base,
                ContainerBase::Internal(ref base_arc) if {
                    let base_guard = base_arc.read().unwrap();
                    matches!(base_guard.base,
                        ContainerBase::External(ref s) if s.repository == "nginx"
                    )
                }
            )
        }
    ));
}

#[test]
fn test_container_deep_nesting() {
    // Create a series of nested containers (like a container builder pattern)
    let base = Container::from("alpine:latest");
    let with_python = Container::from(&base); // Imagine this adds Python
    let with_deps = Container::from(&with_python); // Adds dependencies
    let with_app = Container::from(&with_deps); // Adds application code
    let with_config = Container::from(&with_app); // Adds configuration

    // Verify we can traverse the entire chain of containers
    let mut current = with_config;
    let mut depth = 0;

    loop {
        let guard = current.read().unwrap();
        match &guard.base {
            ContainerBase::Internal(arc) => {
                // Move to the next container in the chain
                let next_container = arc.clone();
                drop(guard); // Release the borrow before reassigning
                current = next_container;
                depth += 1;
            }
            ContainerBase::External(selector) => {
                // We've reached the base container
                assert_eq!(selector.repository, "alpine");
                assert_eq!(selector.tag, Some("latest".to_string()));
                break;
            }
        }

        // Avoid infinite loops in test (shouldn't happen, but safety first)
        if depth > 10 {
            panic!("Too much nesting, possible cycle detected");
        }
    }

    // Verify we found the expected depth (should be 4 levels deep)
    assert_eq!(depth, 4);
}

#[test]
fn test_container_real_world_usage() {
    // Simulate a real-world container management scenario

    // 1. Start with a base image
    let base_image = Container::from("ubuntu:20.04");

    // 2. Derive a container for development
    let dev_container = Container::from(&base_image);

    // 3. Create a production container from the dev container
    let prod_container = Container::from(&dev_container);

    // 4. Create a specialized container for a specific task
    let task_container = Container::from(&prod_container);

    // Verify the lineage of containers
    let task_guard = task_container.read().unwrap();
    if let ContainerBase::Internal(prod_ref) = &task_guard.base {
        let prod_guard = prod_ref.read().unwrap();
        if let ContainerBase::Internal(dev_ref) = &prod_guard.base {
            let dev_guard = dev_ref.read().unwrap();
            if let ContainerBase::Internal(base_ref) = &dev_guard.base {
                let base_guard = base_ref.read().unwrap();
                if let ContainerBase::External(selector) = &base_guard.base {
                    assert_eq!(selector.repository, "ubuntu");
                    assert_eq!(selector.tag, Some("20.04".to_string()));
                    // The full lineage is validated
                    return;
                }
            }
        }
    }

    panic!("Failed to verify the complete container lineage");
}

// EOF
