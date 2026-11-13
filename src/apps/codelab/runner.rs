//! Sandboxed execution of visitor JavaScript. Each run spawns a fresh Web
//! Worker from an in-memory Blob, posts the code and test arguments, and
//! terminates the worker on completion — or after a 4-second deadline, so an
//! infinite loop can never freeze the desktop.

use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const WORKER_JS: &str = r#"
const fmt = v => { if (typeof v === 'string') return v; try { return JSON.stringify(v) } catch (e) { return String(v) } };
self.onmessage = e => {
  const { code, fn, tests } = e.data;
  let cur = [];
  const log = (...a) => cur.push(a.map(fmt).join(' '));
  try { console.log = console.info = console.warn = console.error = console.debug = log } catch (err) {}
  let f;
  try { f = (new Function(code + '\n;return typeof ' + fn + '==="function"?' + fn + ':null;'))() }
  catch (err) { postMessage({ compileError: String(err) }); return }
  if (!f) { postMessage({ compileError: 'Function ' + fn + ' is not defined. Keep the starter signature.' }); return }
  const res = [];
  for (const args of tests) {
    cur = [];
    const t0 = performance.now();
    try {
      let out = f(...JSON.parse(JSON.stringify(args)));
      const ms = performance.now() - t0;
      out = out === undefined ? null : JSON.parse(JSON.stringify(out));
      res.push({ out, ms, logs: cur });
    } catch (err) { res.push({ err: String(err), ms: performance.now() - t0, logs: cur }) }
  }
  postMessage({ res });
};
"#;

pub const DEADLINE_MS: u64 = 4000;

#[derive(Clone, PartialEq, Debug)]
pub struct CaseOut {
    pub out: Value,
    pub err: Option<String>,
    pub ms: f64,
    pub logs: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum RunResult {
    Compile(String),
    Timeout,
    Cases(Vec<CaseOut>),
}

fn parse(v: &Value) -> RunResult {
    if let Some(e) = v.get("compileError").and_then(|e| e.as_str()) {
        return RunResult::Compile(e.to_string());
    }
    let cases = v
        .get("res")
        .and_then(|r| r.as_array())
        .map(|a| {
            a.iter()
                .map(|c| CaseOut {
                    out: c.get("out").cloned().unwrap_or(Value::Null),
                    err: c.get("err").and_then(|e| e.as_str()).map(String::from),
                    ms: c.get("ms").and_then(|m| m.as_f64()).unwrap_or(0.0),
                    logs: c.get("logs").and_then(|l| l.as_array()).map(|l| l.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();
    RunResult::Cases(cases)
}

type Done = Rc<RefCell<Option<Box<dyn FnOnce(RunResult)>>>>;

pub fn run(code: &str, func: &str, tests: &[Vec<Value>], done: impl FnOnce(RunResult) + 'static) {
    let done: Done = Rc::new(RefCell::new(Some(Box::new(done))));
    let fail = |done: &Done, msg: String| {
        if let Some(d) = done.borrow_mut().take() {
            d(RunResult::Compile(msg));
        }
    };

    let parts = js_sys::Array::of1(&JsValue::from_str(WORKER_JS));
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("text/javascript");
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts) else {
        return fail(&done, "Could not create the sandbox.".into());
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return fail(&done, "Could not create the sandbox URL.".into());
    };
    let Ok(worker) = web_sys::Worker::new(&url) else {
        return fail(&done, "Web Workers are unavailable in this browser.".into());
    };

    // Closures must outlive the worker callbacks; they are freed in a later
    // macrotask once the run settles.
    let keep: Rc<RefCell<Vec<Closure<dyn FnMut(JsValue)>>>> = Rc::new(RefCell::new(Vec::new()));
    let settle = {
        let (worker, keep, url) = (worker.clone(), keep.clone(), url.clone());
        move || {
            worker.terminate();
            let _ = web_sys::Url::revoke_object_url(&url);
            let keep = keep.clone();
            leptos::set_timeout(move || keep.borrow_mut().clear(), Duration::ZERO);
        }
    };

    let on_msg = {
        let (done, settle) = (done.clone(), settle.clone());
        Closure::<dyn FnMut(JsValue)>::new(move |e: JsValue| {
            let e: web_sys::MessageEvent = e.unchecked_into();
            let text: String = js_sys::JSON::stringify(&e.data()).map(String::from).unwrap_or_default();
            let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
            if let Some(d) = done.borrow_mut().take() {
                d(parse(&v));
            }
            settle();
        })
    };
    let on_err = {
        let (done, settle) = (done.clone(), settle.clone());
        Closure::<dyn FnMut(JsValue)>::new(move |e: JsValue| {
            let msg = e.dyn_ref::<web_sys::ErrorEvent>().map(|e| e.message()).unwrap_or_else(|| "Worker error".into());
            if let Some(d) = done.borrow_mut().take() {
                d(RunResult::Compile(msg));
            }
            settle();
        })
    };
    worker.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));
    worker.set_onerror(Some(on_err.as_ref().unchecked_ref()));
    keep.borrow_mut().extend([on_msg, on_err]);

    let payload = serde_json::json!({ "code": code, "fn": func, "tests": tests }).to_string();
    match js_sys::JSON::parse(&payload) {
        Ok(v) => {
            if worker.post_message(&v).is_err() {
                fail(&done, "Could not send code to the sandbox.".into());
                settle();
                return;
            }
        }
        Err(_) => return fail(&done, "Could not encode the tests.".into()),
    }

    leptos::set_timeout(
        move || {
            if let Some(d) = done.borrow_mut().take() {
                d(RunResult::Timeout);
                settle();
            }
        },
        Duration::from_millis(DEADLINE_MS),
    );
}
