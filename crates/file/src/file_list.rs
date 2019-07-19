use crate::blob::File;
use wasm_bindgen::UnwrapThrowExt;

pub struct FileList {
    inner: Vec<File>,
}

impl FileList {
    pub fn from_raw(js: web_sys::FileList) -> Self {
        let length = js.length();

        let inner = (0..length)
            .map(|i| File::from_raw(js.get(i).unwrap_throw()))
            .collect();

        Self { inner }
    }
}

impl std::ops::Deref for FileList {
    type Target = [File];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
