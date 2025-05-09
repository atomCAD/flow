// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! # Rivulet
//!
//! A scientific data pipeline workflow runner for high performance computing environments.
//!
//! Rivulet allows scientists and researchers to define, execute, and monitor data processing
//! workflows by connecting containerized analysis steps together. It focuses on maintaining
//! data provenance, preventing unnecessary recomputation, and simplifying reproducible research.
//!
//! ## Key Features
//!
//! - **Scientific Workflow Management**: Define complex multi-step data analysis pipelines
//! - **Container Integration**: Create and connect containers for analysis steps
//! - **Data Provenance Tracking**: Automatically track the origin and transformation history of data
//! - **Computation Efficiency**: Prevent redundant recomputation of unchanged data paths
//! - **HPC Integration**: Designed for high-performance computing environments
//! - **Reproducible Research**: Maintain consistent, reproducible scientific workflows
//!
//! ## Getting Started
//!
//! ```rust
//! use rivulet::prelude::*;
//!
//! // Create containers for analysis steps
//! let data_prep = Container::from("biocontainers/fastqc:latest");
//! let analysis = Container::from("tensorflow/tensorflow:latest-gpu");
//! let visualization = Container::from("rocker/tidyverse:latest");
//!
//! // Create a nested container for a specialized step
//! let custom_analysis = Container::from(&analysis);
//!
//! // Additional workflow setup would connect these containers and configure data flow
//! ```
//!
//! ## Container References
//!
//! Rivulet parses Docker image references following the pattern:
//! ```text
//! [registry/][namespace/]repository[:tag][@algorithm=hash]
//! ```
//!
//! Example of working with image references:
//!
//! ```rust
//! use std::str::FromStr;
//! use rivulet::prelude::*;
//!
//! // Parse an image reference for a scientific computing container
//! let image = ImageSelector::from_str("quay.io/biocontainers/salmon:1.5.2").unwrap();
//! assert_eq!(image.namespace, Some("quay.io/biocontainers".to_string()));
//! assert_eq!(image.repository, "salmon");
//! assert_eq!(image.tag, Some("1.5.2".to_string()));
//! ```
//!
//! ## Workflow Design
//!
//! Rivulet manages scientific workflows by connecting container-based processing steps:
//!
//! ```rust
//! use rivulet::prelude::*;
//!
//! // Set up containers for a genomics pipeline
//! let qc_container = Container::from("biocontainers/fastqc:0.11.9");
//! let alignment_container = Container::from("biocontainers/star:2.7.9a");
//! let counting_container = Container::from("biocontainers/salmon:1.5.2");
//!
//! // Additional code would connect these containers into a workflow
//! // and configure input/output paths and data provenance tracking
//! ```

pub mod container;

/// The prelude module re-exports the most commonly used types and traits.
///
/// Importing items from this module with `use rivulet::prelude::*` allows you to
/// access the core functionality of Rivulet without having to import each component
/// individually.
///
/// # Example
///
/// ```rust
/// // Import all commonly used items
/// use rivulet::prelude::*;
///
/// // Now you can use Container, ImageSelector, etc. directly
/// let container = Container::from("biocontainers/fastqc:latest");
/// ```
pub mod prelude {
    pub use super::container::{Container, ContainerBase, ImageSelector, ImageSelectorParseError};
}

// EOF
