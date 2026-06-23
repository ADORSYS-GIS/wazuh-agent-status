//! AI integration module for Wazuh Agent Status.
//!
//! Provides a provider-agnostic AI client that uses the OpenAI Chat Completions
//! API format, secure OS-native keychain storage for API keys, and SCA fix
//! generation orchestration.
//!
//! # Modules
//!
//! | Module       | Purpose                                           |
//! |--------------|---------------------------------------------------|
//! | `client`     | Provider-agnostic `AiClient` + `AiProviderConfig` |
//! | `keychain`   | OS credential store wrapper (`keyring` crate)     |
//! | `fixer`      | SCA fix prompt builder + orchestrator             |

pub mod client;
pub mod fixer;
pub mod keychain;
