use crate::reader::RecordingHeaderReader;
use std::io::{Read, Write};

pub trait OptionalReadableElement: Sized {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> crate::Result<Option<Self>>;
}

impl<T: OptionalReadableElement> ReadableElement<Option<T>> for T {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> crate::Result<Option<T>> {
        OptionalReadableElement::read_from(input)
    }
}

pub trait ReadableElement<T>: Sized {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> crate::Result<T>;
}

pub trait ReadableHeaderElement: Sized {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> crate::Result<Self>;
}

impl<T: 'static + ReadableHeaderElement> ReadableElement<T> for T {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> crate::Result<T> {
        ReadableHeaderElement::read_from(input)
    }
}

pub trait WritableElement<T> {
    fn write_to<W: Write>(element: &T, output: &mut W) -> crate::Result<()>;
}

pub trait WritableHeaderElement {
    fn write_to<W: Write>(&self, output: &mut W) -> crate::Result<()> {
        // we need to use `output` otherwise we'll get warnings that it's not used
        // prefixing it would make any traits auto completed also be prefixed with _ and that's annoying
        let _ = output;
        unimplemented!()
    }
}

impl<T: WritableHeaderElement> WritableElement<T> for T {
    fn write_to<W: Write>(element: &T, output: &mut W) -> crate::Result<()> {
        WritableHeaderElement::write_to(element, output)
    }
}
