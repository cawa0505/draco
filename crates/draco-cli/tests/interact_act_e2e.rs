//! End-to-end proof that `act` captures a fetch-less reactive render: a click
//! handler mounts a modal DIV with no network. Exercises the real V8 isolate
//! through `draco_core::open_interact_session` + `Session::act`, proving the
//! faithful-event dispatch fires the page's own listener AND the DOM-content-
//! settled pump captures the mount. tier2 + serve gated.
//!
//! The marker text is CONCATENATED in the page script (`'MODAL-' + 'OPENED'`)
//! so the literal never appears in the inline `<script>` source — `serialize()`
//! returns `outerHTML`, which includes script text, so a literal marker would
//! trip the "before" assertion without any click.
#![cfg(all(feature = "tier2", feature = "serve"))]

use axum::response::Html;
use axum::routing::get;
use axum::Router;
use draco_core::Action;

#[tokio::test]
async fn click_captures_a_fetchless_reactive_modal() {
    let app = Router::new().route(
        "/",
        get(|| async {
            Html(
                "<!doctype html><html><head><title>t</title></head><body>\
                 <button id=\"open\">Open</button>\
                 <script>\
                 document.getElementById('open').addEventListener('click', function () {\
                   var d = document.createElement('div');\
                   d.id = 'modal';\
                   d.textContent = 'MODAL-' + 'OPENED';\
                   document.body.appendChild(d);\
                 });\
                 </script></body></html>",
            )
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let base = format!("http://127.0.0.1:{port}/");
    let config = draco_core::Config {
        respect_robots: false,
        ..draco_core::Config::default()
    };

    let session = draco_core::open_interact_session(&base, &config)
        .await
        .expect("session opens");

    let before = session
        .serialize()
        .await
        .expect("serialize delivered")
        .expect("some html");
    assert!(
        !before.contains("MODAL-OPENED"),
        "modal should not exist before the click"
    );

    let report = session
        .act(vec![Action::Click {
            selector: "#open".to_string(),
        }])
        .await
        .expect("act delivered");
    assert!(report.ok, "act should succeed: {:?}", report.steps);

    let after = session
        .serialize()
        .await
        .expect("serialize delivered")
        .expect("some html");
    assert!(
        after.contains("MODAL-OPENED"),
        "the click must fire the page listener and the settle pump must capture \
         the fetch-less modal mount; got: {}",
        after.chars().take(300).collect::<String>()
    );

    session.close().await.expect("close");
}

#[tokio::test]
async fn click_ref_self_healing_reactive() {
    let app = Router::new().route(
        "/",
        get(|| async {
            Html(
                "<!doctype html><html><head><title>Self Healing Test</title></head><body>\
                 <div id=\"app\">\
                   <button id=\"target\">Click Me</button>\
                 </div>\
                 <div id=\"status\">idle</div>\
                 <script>\
                 // Helper to register click handler on target\n\
                 function wire() {\n\
                   const btn = document.getElementById('target');\n\
                   btn.addEventListener('click', function () {\n\
                     document.getElementById('status').textContent = 'CLICKED';\n\
                   });\n\
                 }\n\
                 wire();\n\
                 // Simulate React/Vue unmount-remount (destroy old element reference, recreate identical one)\n\
                 function recreate() {\n\
                   const app = document.getElementById('app');\n\
                   app.innerHTML = '<button id=\"target\">Click Me</button>';\n\
                   wire();\n\
                 }\n\
                 </script></body></html>",
            )
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let base = format!("http://127.0.0.1:{port}/");
    let config = draco_core::Config {
        respect_robots: false,
        ..draco_core::Config::default()
    };

    let session = draco_core::open_interact_session(&base, &config)
        .await
        .expect("session opens");

    // Capture initial snapshot to assign refs
    let snapshot = session.snapshot().await.expect("snapshot succeeds");
    let btn_node = snapshot
        .nodes
        .iter()
        .find(|n| n.role == "button" && n.name == "Click Me")
        .expect("find button");
    let ref_id = btn_node.r#ref.as_ref().expect("has a ref").clone();

    // Now, unmount/remount the button in the page context (breaking the weak map reference pointer)
    session
        .exec(
            "recreate();".to_string(),
            draco_core::ExecOptions::default(),
        )
        .await
        .expect("simulate unmount/remount");

    // Click using the original ref_id (e.g. "e1").
    // Self-healing should search by (role="button", name="Click Me", nth=1) and click the new element!
    let report = session
        .act(vec![Action::ClickRef { r#ref: ref_id }])
        .await
        .expect("act delivered");
    assert!(
        report.ok,
        "act clickRef should succeed with self-healing: {:?}",
        report.steps
    );

    // Verify action succeeded and status changed to CLICKED
    let after = session
        .serialize()
        .await
        .expect("serialize succeeds")
        .expect("some html");
    assert!(
        after.contains("CLICKED"),
        "The self-healed click did not trigger the click listener!"
    );

    session.close().await.expect("close");
}

#[tokio::test]
async fn interactive_only_and_promotion() {
    let app = Router::new().route(
        "/",
        get(|| async {
            Html(
                "<!doctype html><html><head><title>Promotion Test</title></head><body>\
                 <h1>Heading 1</h1>\
                 <ul><li>List Item 1</li></ul>\
                 <div id=\"clickable\" onclick=\"void(0)\">Clickable Div</div>\
                 <div id=\"pointer\" style=\"cursor: pointer\">Pointer Div</div>\
                 <span id=\"tabindex\" tabindex=\"0\">Tabindex Span</span>\
                 </body></html>",
            )
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let base = format!("http://127.0.0.1:{port}/");
    let config = draco_core::Config {
        respect_robots: false,
        ..draco_core::Config::default()
    };

    let session = draco_core::open_interact_session(&base, &config)
        .await
        .expect("session opens");

    let snapshot = session.snapshot().await.expect("snapshot succeeds");

    // 1. Plain content roles (heading, list item) must NOT receive refs (interactive-only rule)
    let heading = snapshot
        .nodes
        .iter()
        .find(|n| n.name == "Heading 1")
        .expect("find heading");
    assert!(
        heading.r#ref.is_none(),
        "Heading 1 should not have a ref: {:?}",
        heading
    );

    let list_item = snapshot
        .nodes
        .iter()
        .find(|n| n.name == "List Item 1")
        .expect("find list item");
    assert!(
        list_item.r#ref.is_none(),
        "List Item 1 should not have a ref: {:?}",
        list_item
    );

    // 2. Clickable / promoted content elements MUST receive refs (promotion rule)
    let clickable = snapshot
        .nodes
        .iter()
        .find(|n| n.name == "Clickable Div")
        .expect("find clickable");
    assert!(
        clickable.r#ref.is_some(),
        "Clickable Div should have been promoted to receive a ref"
    );

    let pointer = snapshot
        .nodes
        .iter()
        .find(|n| n.name == "Pointer Div")
        .expect("find pointer");
    assert!(
        pointer.r#ref.is_some(),
        "Pointer Div should have been promoted to receive a ref"
    );

    let tabindex = snapshot
        .nodes
        .iter()
        .find(|n| n.name == "Tabindex Span")
        .expect("find tabindex");
    assert!(
        tabindex.r#ref.is_some(),
        "Tabindex Span should have been promoted to receive a ref"
    );

    session.close().await.expect("close");
}
