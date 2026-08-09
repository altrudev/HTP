// SPDX-License-Identifier: Apache-2.0

//! Stable dimensional vocabulary used by HTP's public DDC integration boundary.
//!
//! This module intentionally does not contain the private Crystalline/DDC transaction
//! closure engine. HTP uses the same eight dimensional names for interoperable change
//! classification while keeping canonical protocol verification independent of private
//! infrastructure.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dimension {
    Semantic,
    Authority,
    State,
    Resource,
    Security,
    Physical,
    Frequency,
    Lineage,
}

impl Dimension {
    pub const ALL: [Dimension; 8] = [
        Dimension::Semantic,
        Dimension::Authority,
        Dimension::State,
        Dimension::Resource,
        Dimension::Security,
        Dimension::Physical,
        Dimension::Frequency,
        Dimension::Lineage,
    ];
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Dimension::Semantic => "semantic",
            Dimension::Authority => "authority",
            Dimension::State => "state",
            Dimension::Resource => "resource",
            Dimension::Security => "security",
            Dimension::Physical => "physical",
            Dimension::Frequency => "frequency",
            Dimension::Lineage => "lineage",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Effect {
    pub dimension: Dimension,
    pub boundary: String,
}

impl Effect {
    pub fn new(dimension: Dimension, boundary: impl Into<String>) -> Self {
        Self {
            dimension,
            boundary: boundary.into(),
        }
    }
}

/// Canonical HTP dimensional change summary.
///
/// The shape is deliberately compatible with HTP 0.2's previous public DDC-facing
/// change representation, but its portable derivation rules are defined inside HTP.
/// A full Crystalline/DDC implementation may perform additional non-normative analysis
/// outside the signed HTP witness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DimensionalChange {
    #[serde(default)]
    pub changed_dimensions: BTreeMap<Dimension, BTreeSet<String>>,
    #[serde(default)]
    pub conserved_dimensions: BTreeSet<Dimension>,
}

impl DimensionalChange {
    pub fn from_changed(changed_dimensions: BTreeMap<Dimension, BTreeSet<String>>) -> Self {
        let conserved_dimensions = Dimension::ALL
            .into_iter()
            .filter(|dimension| !changed_dimensions.contains_key(dimension))
            .collect();
        Self {
            changed_dimensions,
            conserved_dimensions,
        }
    }
}
