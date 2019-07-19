#[derive(Debug)]
pub struct FileReadError {
    error: web_sys::DomException,
}

impl std::fmt::Display for FileReadError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.error.message().fmt(f)
    }
}

impl std::error::Error for FileReadError {}

pub mod callbacks {
    use super::FileReadError;
    use crate::blob::BlobLike;
    use gloo_events::EventListener;
    use wasm_bindgen::{JsValue, UnwrapThrowExt};

    fn get_result(reader: &web_sys::FileReader) -> Result<JsValue, FileReadError> {
        if let Some(error) = reader.error() {
            Err(FileReadError { error })
        } else {
            Ok(reader.result().unwrap_throw())
        }
    }

    fn as_string(js: JsValue) -> String {
        js.as_string().unwrap_throw()
    }

    #[derive(Debug)]
    pub struct FileReader {
        reader: web_sys::FileReader,
        on_loadend: Option<EventListener>,
    }

    fn read_as<R, F>(blob: &web_sys::Blob, read: R, callback: F) -> FileReader
    where
        R: Fn(&web_sys::FileReader, &web_sys::Blob) -> Result<(), JsValue>,
        F: FnOnce(Result<JsValue, FileReadError>) + 'static,
    {
        let reader = web_sys::FileReader::new().unwrap_throw();

        let on_loadend = Some(EventListener::once(&reader, "loadend", {
            let reader = reader.clone();
            move |_| {
                callback(get_result(&reader));
            }
        }));

        read(&reader, blob).unwrap_throw();

        FileReader { reader, on_loadend }
    }

    #[inline]
    pub fn read_to_string<B, F>(blob: &B, callback: F) -> FileReader
    where
        B: BlobLike,
        F: FnOnce(Result<String, FileReadError>) + 'static,
    {
        read_as(blob.as_raw(), web_sys::FileReader::read_as_text, move |x| {
            callback(x.map(as_string))
        })
    }

    #[inline]
    pub fn read_to_data_url<B, F>(blob: &B, callback: F) -> FileReader
    where
        B: BlobLike,
        F: FnOnce(Result<String, FileReadError>) + 'static,
    {
        read_as(
            blob.as_raw(),
            web_sys::FileReader::read_as_data_url,
            move |x| callback(x.map(as_string)),
        )
    }

    #[inline]
    pub fn read_to_array_buffer<B, F>(blob: &B, callback: F) -> FileReader
    where
        B: BlobLike,
        F: FnOnce(Result<js_sys::ArrayBuffer, FileReadError>) + 'static,
    {
        read_as(
            blob.as_raw(),
            web_sys::FileReader::read_as_array_buffer,
            move |x| callback(x.map(Into::into)),
        )
    }

    impl Drop for FileReader {
        fn drop(&mut self) {
            if self.reader.ready_state() != web_sys::FileReader::DONE {
                // This is necessary to remove the EventListener so it isn't called by abort
                self.on_loadend.take();
                self.reader.abort();
            }
        }
    }
}
