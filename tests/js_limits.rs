//! Hostile JavaScript must end on its own, quickly, with a clear result.
//!
//! Each case runs the release-style CLI binary on a small page:
//! - the loop limit and the timer clamp let rendering continue (exit 0, PDF
//!   written, error reported);
//! - the watchdog ends stuck scripts with exit status 3;
//! - the heap limit ends allocation bombs with exit status 4;
//! - ordinary heavy scripts are not affected.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

fn page(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(format!("{name}.html"));
    std::fs::write(&path, format!("<!DOCTYPE html><meta charset=utf-8>{body}")).unwrap();
    path
}

fn run(input: &Path, args: &[&str]) -> (Output, Duration, PathBuf) {
    let out = input.with_extension("pdf");
    let _ = std::fs::remove_file(&out);
    let start = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .args(args)
        .arg("--debug-javascript")
        .arg(input)
        .arg(&out)
        .output()
        .expect("run bdhtml2pdf");
    (output, start.elapsed(), out)
}

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "bdhtml2pdf-js-limits-{name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn infinite_loop_is_stopped_and_the_page_still_renders() {
    let dir = tempdir("loop");
    let input = page(
        &dir,
        "loop",
        "<p>before</p><script>while (true) {}</script>",
    );
    let (o, took, pdf) = run(&input, &[]);
    assert!(o.status.success(), "exit {:?}: {}", o.status, stderr(&o));
    assert!(pdf.exists(), "PDF must still be written");
    assert!(stderr(&o).contains("RuntimeLimitError"), "{}", stderr(&o));
    assert!(took < Duration::from_secs(10), "took {took:?}");
}

#[test]
fn try_catch_cannot_swallow_the_loop_limit() {
    let dir = tempdir("catch");
    let input = page(
        &dir,
        "catch",
        "<script>while (true) { try { while (true) {} } catch (e) {} }</script>",
    );
    let (o, took, _) = run(&input, &[]);
    assert!(o.status.success(), "exit {:?}: {}", o.status, stderr(&o));
    assert!(took < Duration::from_secs(10), "took {took:?}");
}

#[test]
fn zero_delay_timer_flood_uses_up_the_virtual_time_budget() {
    let dir = tempdir("timers");
    let input = page(
        &dir,
        "timers",
        "<script>setInterval(function () {}, 0); (function again() { setTimeout(again, 0); })();</script>",
    );
    let (o, took, pdf) = run(&input, &[]);
    assert!(o.status.success(), "exit {:?}: {}", o.status, stderr(&o));
    assert!(pdf.exists());
    assert!(took < Duration::from_secs(5), "took {took:?}");
}

#[test]
fn endless_microtasks_hit_the_wall_clock_watchdog() {
    let dir = tempdir("microtasks");
    let input = page(
        &dir,
        "microtasks",
        "<script>function f() { return Promise.resolve().then(f); } f();</script>",
    );
    let (o, took, _) = run(&input, &["--javascript-timeout", "1"]);
    assert_eq!(o.status.code(), Some(3), "{}", stderr(&o));
    assert!(
        stderr(&o).contains("--javascript-timeout"),
        "{}",
        stderr(&o)
    );
    assert!(took < Duration::from_secs(10), "took {took:?}");
}

#[test]
fn allocation_bomb_hits_the_memory_limit() {
    let dir = tempdir("memory");
    let input = page(
        &dir,
        "memory",
        "<script>var a = new Array(400000000).fill(1.5);</script>",
    );
    let (o, _, _) = run(
        &input,
        &["--max-memory", "200", "--javascript-timeout", "120"],
    );
    assert_eq!(o.status.code(), Some(4), "{}", stderr(&o));
    assert!(stderr(&o).contains("--max-memory"), "{}", stderr(&o));
}

#[test]
fn heavy_but_finite_scripts_are_not_affected() {
    let dir = tempdir("finite");
    // A million iterations in one call, many calls with loops, and a short
    // chain of zero-delay timers (below the nesting clamp).
    let input = page(
        &dir,
        "finite",
        r#"<p id="o">pending</p><script>
        var s = 0; for (var i = 0; i < 1000000; i++) { s += i; }
        function f() { for (var j = 0; j < 1000; j++) {} }
        for (var k = 0; k < 2000; k++) f();
        var n = 0;
        (function step() { if (++n < 4) setTimeout(step, 0); else document.getElementById("o").textContent = "done " + s; })();
        </script>"#,
    );
    let (o, _, _) = run(&input, &[]);
    assert!(o.status.success(), "exit {:?}: {}", o.status, stderr(&o));
    assert!(!stderr(&o).contains("Error"), "{}", stderr(&o));
    let html = input.with_extension("out.html");
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg(&input)
        .arg(&html)
        .output()
        .unwrap();
    assert!(o.status.success());
    let dom = std::fs::read_to_string(&html).unwrap();
    assert!(dom.contains("done 499999500000"), "{dom}");
}
