# Draco Fork Handoff: Transitioning to Phase 2 (Stateful Session Diagnostics)

We have successfully designed, implemented, verified, and released **Phase 1 (Observation-First Action-by-Ref)** along with **Option A (Ref Self-healing & Interactive Selection)** and **CJK Charset Sniffing** in **v0.22.0**.

The repository is in a 100% clean, verified state, and the local daemon is active. Below is the transition plan and handoff for the upcoming phases.

---

## 🏁 Phase 1 (Option A) Achievements & Delivered Architecture

### 1. Zero-Guesswork A11y Snapshot
- **File**: `crates/draco-runtime/js/glue.js` (`g.__dracoSerializeA11y`)
- **API**: `draco_interact_snapshot` (REST: `POST /v1/interact/:id/snapshot`)
- **Mechanism**: Walks the shadow DOM inside the V8 isolate to compute roles, names, and states (checked, disabled, expanded, pressed, selected, etc.) dynamically. Assigns stable sequential `eN` reference keys.

### 2. Vue/React-Resilient Ref Self-Healing
- **File**: `crates/draco-runtime/js/glue.js` (`g.__dracoRefEl`)
- **Mechanism**: Captures identity triples `(role, name, nth)` for each assigned ref during the initial snapshot walk. If React/Svelte/Vue unmounts and recreates the DOM node, the element resolver walks the DOM in the same order, re-identifies the new node, and seamlessly restores the target pointer.
- **E2E Test**: `click_ref_self_healing_reactive` (verifies click succeeds after direct DOM node destruction and reconstruction).

### 3. CJK Charset-Aware Decodes
- **File**: `crates/draco-core/src/lib.rs` (`decode_body`, `content_type_of`)
- **Mechanism**: Implements WHATWG encoding sniffing (BOM $\to$ Content-Type charset $\to$ meta prescan $\to$ UTF-8 fallback) using `encoding_rs` across all 8 fetch, crawl, search, and map entry points.

---

## 🚧 Next Phase: Phase 2 (Stateful Session Diagnostics)

Our next objective is to implement **Phase 2 (Stateful Session Diagnostics)** under the **MCP Ergonomics Spec (R2)**.

### Target Requirements
AI agents running multi-step automation tasks need to inspect the status, memory usage, and lifecycle of their active browser sessions to prevent leaks, clean up orphaned resources, and troubleshoot hangs.

1. **`list_sessions` MCP Tool** (REST: `GET /v1/interact`)
   - Returns an array of active session metadata:
     ```json
     {
       "id": "ses_034f9...",
       "url": "https://example.com/dashboard",
       "created_at": 171234567890,
       "last_active": 171234569990,
       "ttl_seconds": 280,
       "v8_heap_used_bytes": 1420584
     }
     ```
2. **`session_status` / Diagnostics Tool** (REST: `GET /v1/interact/:id/status`)
   - Returns a detailed diagnostics payload for a single session:
     - JS Console Logs (`w.console.logs` or captured runtime trace)
     - Pending network requests count
     - Pending timers or microtask counts (via happy-dom/V8 hooks if accessible)
3. **Session Auto-Expiry & Garbage Collection**
   - Active sessions should automatically drop and clean up V8 isolates after 5 minutes of inactivity.
   - Clean up must free V8 heaps immediately.

### Action Items for Next Session
1. **Extend `SessionStore`** (`crates/draco-runtime/src/session.rs`):
   - Add a method `pub fn list_sessions(&self) -> Vec<SessionMeta>` that iterates over `sessions` and extracts active properties.
2. **Register MCP Tools** (`crates/draco-cli/src/mcp.rs`):
   - Register `draco_interact_list_sessions` tool.
   - Register `draco_interact_session_status` tool.
3. **Connect REST Routes** (`crates/draco-cli/src/serve/interact.rs`):
   - Wire `GET /v1/interact` to lists.
   - Wire `GET /v1/interact/:id/status` to diagnostics.
