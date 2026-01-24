//! Driftless - Streamlined system configuration, inventory, and monitoring agent
//!
//! This crate provides a comprehensive agent for system configuration management,
//! facts collection, and log processing with continuous monitoring capabilities.

pub mod agent;
pub mod apply;
pub mod doc_extractor;
pub mod docs;
pub mod facts;
pub mod logs;
pub mod plugins;
