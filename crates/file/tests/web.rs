//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]
use futures::unsync::oneshot::channel;
use futures::Future;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use gloo_file::callbacks::read_to_string;
use gloo_file::Blob;

wasm_bindgen_test_configure!(run_in_browser);

fn eq<A>(l: A, r: A) -> Result<(), JsValue>
where
    A: PartialEq,
{
    if l == r {
        Ok(())
    } else {
        Err(JsValue::from("Assertion failed"))
    }
}

#[wasm_bindgen_test(async)]
fn read_as_text() -> impl Future<Item = (), Error = JsValue> {
    let (sender, receiver) = channel();

    let blob = Blob::new("Hello world!");

    std::mem::forget(read_to_string(&blob, move |s| {
        sender.send(s).unwrap_throw();
    }));

    receiver.then(|x| match x.unwrap_throw() {
        Ok(x) => eq(x.as_str(), "Hello world!"),
        Err(x) => Err(JsValue::from(x.to_string())),
    })
}

#[wasm_bindgen_test]
fn read_as_text_abort() {
    let blob = Blob::new("Hello world!");

    let _ = read_to_string(&blob, move |_| {
        unreachable!();
    });
}
