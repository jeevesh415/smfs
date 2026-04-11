//! Supermemory HTTP API client.
//!
//! Typed wrapper over the Supermemory REST endpoints. The client is
//! stateless — the caller passes a token and container tag, and the client
//! injects them into every request. Retries network errors and 5xx responses
//! with exponential backoff; surfaces 4xx as typed errors without retrying.
//!
//! Fully mockable against [`wiremock`](https://crates.io/crates/wiremock)
//! so the sync-engine tests never hit the network.
//!
//! ## Planned contents (M6)
//!
//! - DTOs: `Document`, `CreateDocumentReq`, `ListDocumentsReq`/`Resp`,
//!   `DocumentContent`
//! - `ApiClient` backed by `reqwest` with `rustls-tls` (no OpenSSL dep)
//! - Endpoint methods: `list_documents`, `get_document`, `get_document_content`
//!   (streaming), `create_document`, `update_document`, `delete_document`,
//!   `update_document_filepath` (for rename)
//! - Retry-with-backoff wrapper (5 attempts, 100ms → 1600ms exponential)
//! - Error taxonomy: `ApiError::{NetworkError, HttpStatus, Auth, NotFound, Conflict}`
//!
//! ## Auth model
//!
//! The Rust binary never touches browser OAuth or keyring storage — the
//! existing `@repo/cli` TypeScript CLI owns that flow. When a user runs
//! `supermemory mount ~/work work-tag`, the TS CLI reads its credentials
//! file and execs `smfs daemon-inner --token <tok>`. This module just
//! accepts a token on construction.
//!
//! See `.plan/v0-plan.md` milestone M6 for the detailed spec.

// TODO(M6): DTOs, ApiClient, retry wrapper, typed errors.
