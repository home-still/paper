# Walkthrough: Complete ArxivProvider

**Date:** 2026-03-03
**Status:** Planning
**Checkpoint:** f27498ee315e9e64234e2481c8f9839ee24c272a

## Goal

Complete the ArxivProvider so it can search arXiv's Atom API, parse full paper metadata (authors, abstract, dates, PDF URLs), and return properly structured results — with unit tests proving it works.

## Acceptance Criteria

- [ ] `search()` sends HTTP request and returns `SearchResult` with parsed papers
- [ ] `extract_paper()` extracts all fields: id, title, authors, abstract, publication_date, doi, download_url
- [ ] `get_by_doi()` searches arXiv by DOI and returns a single paper
- [ ] URL query parameters are properly encoded (spaces, special chars)
- [ ] Unit tests pass for URL construction, XML parsing, and field extraction
- [ ] `cargo build` succeeds with zero warnings
- [ ] `cargo test` passes all tests

## Technical Approach

### Architecture

ArxivProvider sits in the Provider Layer, implementing the `PaperProvider` trait from the Ports layer. It uses `reqwest` for HTTP, `roxmltree` for XML parsing, and maps arXiv's Atom feed entries to our domain `Paper` model.

```
PaperProvider trait (ports/provider.rs)
    ↑ implements
ArxivProvider (providers/arxiv.rs)
    → reqwest::Client (HTTP)
    → roxmltree (XML parsing)
    → Paper, SearchResult (domain models)
```

### Key Decisions

- **URL encoding**: Use the `url` crate's `Url::parse_with_params` instead of adding `urlencoding` — fewer deps, more correct
- **Author parsing**: arXiv nests `<name>` inside `<author>` elements — iterate children, not just text
- **PDF URL extraction**: Use `<link>` elements with `title="pdf"` attribute — more reliable than string-replacing `/abs/` with `/pdf/`
- **Date parsing**: Use `chrono::NaiveDate::parse_from_str` on the first 10 chars of the ISO 8601 timestamp
- **Total results**: arXiv provides `<opensearch:totalResults>` — parse it from a different namespace

### Dependencies

- `roxmltree` — already in Cargo.toml
- `reqwest` — already in Cargo.toml
- `url` — already in Cargo.toml
- `chrono` — already in Cargo.toml
- No new dependencies needed

### Files to Create/Modify

- `paper-fetch-core/src/providers/arxiv.rs`: Complete implementation (only file with code changes)

## Build Order

1. **Fix existing bugs**: Remove typo `{E`, fix `au;` → `au:` — unblocks compilation
2. **URL encoding**: Use `url` crate for proper query construction — unblocks correct API calls
3. **`extract_paper()`**: Full field extraction from XML — unblocks search returning real data
4. **`search()`**: Wire up HTTP call + parsing — the primary user-facing method
5. **`get_by_doi()`**: Reuse search with DOI query — secondary feature
6. **Total results parsing**: Extract `opensearch:totalResults` — makes pagination work
7. **Unit tests**: URL construction, XML parsing, field extraction — proves it all works

## Anticipated Challenges

- **XML namespaces**: Atom uses `http://www.w3.org/2005/Atom`, OpenSearch uses `http://a9.com/-/spec/opensearch/1.1/` — must use tuple syntax `(ns, tag)` with roxmltree
- **arXiv ID format**: IDs look like `http://arxiv.org/abs/1234.5678v1` — extract just the `1234.5678v1` part for our `Paper.id`
- **Missing fields**: Some entries may lack DOI, abstract, or PDF link — all optional fields, use `Option`
- **Date format**: arXiv uses `2023-01-15T12:00:00Z` — need to truncate to date portion for `NaiveDate`

## Steps (To Be Filled During Proof Phase)

[This section will be populated after we build and verify the implementation]

---
*Plan created: 2026-03-03*
*Implementation proven: [to be updated]*
*User implementation started: [to be updated]*
