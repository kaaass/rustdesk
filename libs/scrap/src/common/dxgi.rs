use crate::{common::TraitCapturer, dxgi, Pixfmt};
use std::{
    io::{
        self,
        ErrorKind::{NotFound, TimedOut, WouldBlock},
    },
    time::Duration,
};

pub struct Capturer {
    inner: dxgi::Capturer,
    width: usize,
    height: usize,
}

impl Capturer {
    pub fn new(display: Display) -> io::Result<Capturer> {
        let width = display.width();
        let height = display.height();
        let inner = dxgi::Capturer::new(display.0)?;
        Ok(Capturer {
            inner,
            width,
            height,
        })
    }

    pub fn cancel_gdi(&mut self) {
        self.inner.cancel_gdi()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl TraitCapturer for Capturer {
    fn frame<'a>(&'a mut self, timeout: Duration) -> io::Result<Frame<'a>> {
        match self.inner.frame(timeout.as_millis() as _) {
            Ok(frame) => Ok(Frame::new(frame, self.height)),
            Err(ref error) if error.kind() == TimedOut => Err(WouldBlock.into()),
            Err(error) => Err(error),
        }
    }

    fn is_gdi(&self) -> bool {
        self.inner.is_gdi()
    }

    fn set_gdi(&mut self) -> bool {
        self.inner.set_gdi()
    }
}

pub struct Frame<'a> {
    data: &'a [u8],
    stride: Vec<usize>,
}

impl<'a> Frame<'a> {
    pub fn new(data: &'a [u8], h: usize) -> Self {
        let stride = data.len() / h;
        let mut v = Vec::new();
        v.push(stride);
        Frame { data, stride: v }
    }
}

impl<'a> crate::TraitFrame for Frame<'a> {
    fn data(&self) -> &[u8] {
        self.data
    }

    fn stride(&self) -> Vec<usize> {
        self.stride.clone()
    }

    fn pixfmt(&self) -> Pixfmt {
        Pixfmt::BGRA
    }
}

pub struct Display(dxgi::Display);

impl Display {
    pub fn primary() -> io::Result<Display> {
        // not implemented yet
        Err(NotFound.into())
    }

    pub fn all() -> io::Result<Vec<Display>> {
        let tmp = Self::all_().unwrap_or(Default::default());
        if tmp.is_empty() {
            println!("Display got from gdi");
            return Ok(dxgi::Displays::get_from_gdi()
                .drain(..)
                .map(Display)
                .collect::<Vec<_>>());
        }
        Ok(tmp)
    }

    fn all_() -> io::Result<Vec<Display>> {
        Ok(dxgi::Displays::new()?.map(Display).collect::<Vec<_>>())
    }

    pub fn width(&self) -> usize {
        self.0.width() as usize
    }

    pub fn height(&self) -> usize {
        self.0.height() as usize
    }

    pub fn name(&self) -> String {
        use std::ffi::OsString;
        use std::os::windows::prelude::*;
        OsString::from_wide(self.0.name())
            .to_string_lossy()
            .to_string()
    }

    pub fn is_online(&self) -> bool {
        self.0.is_online()
    }

    pub fn origin(&self) -> (i32, i32) {
        self.0.origin()
    }

    pub fn is_primary(&self) -> bool {
        // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-devmodea
        self.origin() == (0, 0)
    }
}

pub struct CapturerMag {
    inner: dxgi::mag::CapturerMag,
    data: Vec<u8>,
}

impl CapturerMag {
    pub fn is_supported() -> bool {
        dxgi::mag::CapturerMag::is_supported()
    }

    pub fn new(origin: (i32, i32), width: usize, height: usize) -> io::Result<Self> {
        Ok(CapturerMag {
            inner: dxgi::mag::CapturerMag::new(origin, width, height)?,
            data: Vec::new(),
        })
    }

    pub fn exclude(&mut self, cls: &str, name: &str) -> io::Result<bool> {
        self.inner.exclude(cls, name)
    }
    // ((x, y), w, h)
    pub fn get_rect(&self) -> ((i32, i32), usize, usize) {
        self.inner.get_rect()
    }
}

impl TraitCapturer for CapturerMag {
    fn frame<'a>(&'a mut self, _timeout_ms: Duration) -> io::Result<Frame<'a>> {
        self.inner.frame(&mut self.data)?;
        Ok(Frame::new(&self.data, self.inner.get_rect().2))
    }

    fn is_gdi(&self) -> bool {
        false
    }

    fn set_gdi(&mut self) -> bool {
        false
    }
}
