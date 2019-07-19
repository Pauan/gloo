use std::time::Duration;
use wasm_bindgen::UnwrapThrowExt;

// Max integer safely representable by f64
// Equivalent to `(2.0f64).powi(53) - 1.0`
const MAX_F64_INTEGER: f64 = 9007199254740991.0;

fn from_u64(number: u64) -> f64 {
    assert!(
        number <= (MAX_F64_INTEGER as u64),
        "{} is too large and cannot be sent to JavaScript",
        number
    );

    number as f64
}

fn from_u128(number: u128) -> f64 {
    assert!(
        number <= (MAX_F64_INTEGER as u128),
        "{} is too large and cannot be sent to JavaScript",
        number
    );

    number as f64
}

fn blob_slice(blob: &web_sys::Blob, start: u64, end: u64) -> web_sys::Blob {
    let start = from_u64(start);
    let end = from_u64(end);
    blob.slice_with_f64_and_f64(start, end).unwrap_throw()
}

pub trait BlobLike {
    #[inline]
    fn size(&self) -> u64 {
        self.as_raw().size() as u64
    }

    #[cfg(feature = "mime")]
    #[inline]
    fn mime_type(&self) -> Result<mime::Mime, mime::FromStrError> {
        self.raw_mime_type().parse()
    }

    #[inline]
    fn raw_mime_type(&self) -> String {
        self.as_raw().type_()
    }

    fn as_raw(&self) -> &web_sys::Blob;

    fn slice(&self, start: u64, end: u64) -> Self;
}

#[derive(Debug, Clone)]
pub struct BlobContents {
    inner: wasm_bindgen::JsValue,
}

impl BlobContents {
    #[inline]
    fn from_raw(inner: wasm_bindgen::JsValue) -> BlobContents {
        BlobContents { inner }
    }
}

impl From<&str> for BlobContents {
    #[inline]
    fn from(str: &str) -> Self {
        BlobContents::from_raw(wasm_bindgen::JsValue::from_str(str))
    }
}

impl From<&[u8]> for BlobContents {
    #[inline]
    fn from(buffer: &[u8]) -> Self {
        BlobContents::from_raw(js_sys::Uint8Array::from(buffer).into())
    }
}

impl From<Blob> for BlobContents {
    #[inline]
    fn from(blob: Blob) -> Self {
        BlobContents::from_raw(blob.inner.into())
    }
}

impl From<File> for BlobContents {
    #[inline]
    fn from(file: File) -> Self {
        BlobContents::from_raw(file.inner.into())
    }
}

impl From<web_sys::Blob> for BlobContents {
    #[inline]
    fn from(blob: web_sys::Blob) -> Self {
        BlobContents::from_raw(blob.into())
    }
}

impl From<web_sys::File> for BlobContents {
    #[inline]
    fn from(blob: web_sys::File) -> Self {
        BlobContents::from_raw(blob.into())
    }
}

impl From<js_sys::ArrayBuffer> for BlobContents {
    #[inline]
    fn from(buffer: js_sys::ArrayBuffer) -> Self {
        BlobContents::from_raw(buffer.into())
    }
}

#[derive(Debug, Clone)]
pub struct Blob {
    inner: web_sys::Blob,
}

impl Blob {
    pub fn new<T>(content: T) -> Blob
    where
        T: Into<BlobContents>,
    {
        let parts = js_sys::Array::of1(&content.into().inner);
        let inner = web_sys::Blob::new_with_buffer_source_sequence(&parts).unwrap_throw();
        Blob::from_raw(inner)
    }

    pub fn new_with_options<T>(content: T, mime_type: String) -> Blob
    where
        T: Into<BlobContents>,
    {
        let mut properties = web_sys::BlobPropertyBag::new();
        properties.type_(&mime_type);

        let parts = js_sys::Array::of1(&content.into().inner);
        let inner = web_sys::Blob::new_with_buffer_source_sequence_and_options(&parts, &properties)
            .unwrap_throw();

        Blob::from_raw(inner)
    }

    #[inline]
    pub fn from_raw(inner: web_sys::Blob) -> Blob {
        Blob { inner }
    }
}

impl BlobLike for Blob {
    #[inline]
    fn as_raw(&self) -> &web_sys::Blob {
        &self.inner
    }

    #[inline]
    fn slice(&self, start: u64, end: u64) -> Self {
        Blob::from_raw(blob_slice(self.as_raw(), start, end))
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub(crate) inner: web_sys::File,
}

impl File {
    pub fn new<T>(name: String, contents: T) -> File
    where
        T: Into<BlobContents>,
    {
        let parts = js_sys::Array::of1(&contents.into().inner);
        let inner = web_sys::File::new_with_buffer_source_sequence(&parts, &name).unwrap_throw();

        File::from_raw(inner)
    }

    pub fn new_with_options<T>(
        name: String,
        contents: T,
        mime_type: Option<String>,
        last_modified_since_epoch: Option<Duration>,
    ) -> File
    where
        T: Into<BlobContents>,
    {
        let mut options = web_sys::FilePropertyBag::new();

        if let Some(mime_type) = mime_type {
            options.type_(&mime_type);
        }

        if let Some(last_modified) = last_modified_since_epoch {
            options.last_modified(from_u128(last_modified.as_millis()));
        }

        let parts = js_sys::Array::of1(&contents.into().inner);
        let inner =
            web_sys::File::new_with_buffer_source_sequence_and_options(&parts, &name, &options)
                .unwrap_throw();

        File::from_raw(inner)
    }

    #[inline]
    pub fn from_raw(inner: web_sys::File) -> File {
        File { inner }
    }

    #[inline]
    pub fn name(&self) -> String {
        self.inner.name()
    }

    #[inline]
    pub fn last_modified_since_epoch(&self) -> Duration {
        Duration::from_millis(self.inner.last_modified() as u64)
    }

    #[inline]
    pub fn size(&self) -> u64 {
        self.inner.size() as u64
    }
}

impl BlobLike for File {
    #[inline]
    fn as_raw(&self) -> &web_sys::Blob {
        &self.inner
    }

    fn slice(&self, start: u64, end: u64) -> Self {
        let blob = blob_slice(self.as_raw(), start, end);

        let raw_mime_type = self.raw_mime_type();

        let mime_type = if raw_mime_type == "" {
            None
        } else {
            Some(raw_mime_type)
        };

        File::new_with_options(
            self.name(),
            blob,
            mime_type,
            Some(self.last_modified_since_epoch()),
        )
    }
}
