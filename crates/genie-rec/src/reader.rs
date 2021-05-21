use crate::{GameVariant, GameVersion};
use std::cmp::min;
use std::io;
use std::io::{BufRead, Read};

/// A struct implementing `BufRead` that uses a small, single-use, stack-allocated buffer, intended
/// for reading only the first few bytes from a file.
pub(crate) struct SmallBufReader<R>
where
    R: Read,
{
    buffer: [u8; 256],
    pointer: usize,
    reader: R,
}

impl<R> SmallBufReader<R>
where
    R: Read,
{
    pub(crate) fn new(reader: R) -> Self {
        Self {
            buffer: [0; 256],
            pointer: 0,
            reader,
        }
    }
}

impl<R> Read for SmallBufReader<R>
where
    R: Read,
{
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.reader.read(output)
    }
}

impl<R> BufRead for SmallBufReader<R>
where
    R: Read,
{
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.reader.read_exact(&mut self.buffer[self.pointer..])?;
        Ok(&self.buffer[self.pointer..])
    }

    fn consume(&mut self, len: usize) {
        self.pointer += len;
    }
}

/// dynamically allocated buffered reader, by method of "inflation"
/// The inflation allows us to "peek" into streams, without destroying data
/// Reading from the buffer also only advances the pointer inside the buffer
/// Only once another peek is done, data is rearranged (or simply overwritten if the whole buffer has been consumed)
#[derive(Debug)]
pub struct InflatableReader<R> {
    /// inner reader
    inner: R,
    /// buffer for peek support, only inflates when needed
    inflatable_buffer: Vec<u8>,
    /// Position in our inflatable buffer
    position_in_buffer: usize,
}

impl<R> InflatableReader<R> {
    pub fn new(inner: R) -> Self {
        Self::new_with_capacity(inner, 4096)
    }

    pub fn new_with_capacity(inner: R, capacity: usize) -> Self {
        InflatableReader {
            inner,
            inflatable_buffer: Vec::with_capacity(capacity),
            position_in_buffer: 0,
        }
    }
}

pub trait Peek {
    fn peek(&mut self, amount: usize) -> io::Result<&[u8]>;
}

impl<R: Read> Peek for &mut InflatableReader<R> {
    fn peek(&mut self, amount: usize) -> io::Result<&[u8]> {
        (*self).peek(amount)
    }
}

impl<R: Read> Peek for InflatableReader<R> {
    /// Peek into inner reader, returns a slice owned by the [InflatableReader],
    /// if data isn't available in the buffer yet, it will read it into the inner buffer
    /// and inflate the buffer if needed.
    fn peek(&mut self, amount: usize) -> io::Result<&[u8]> {
        // cache this info, since we fuck around with position_in_buffer
        let buffered_data_length = self.inflatable_buffer.len() - self.position_in_buffer;

        // quick return because we have all the data to peek already
        if buffered_data_length >= amount {
            return Ok(
                &self.inflatable_buffer[self.position_in_buffer..self.position_in_buffer + amount]
            );
        }

        // from this point on we can assume that we need to allocate more, and [amount] is always the bigger value

        // see how much we're missing
        let missing = amount - buffered_data_length;

        // we were at the end of our buffer, just reset the position without resizing yet.
        if self.position_in_buffer == self.inflatable_buffer.len() {
            self.position_in_buffer = 0;
        } else {
            // if we have enough capacity to house the missing data, just skip this part
            if self.inflatable_buffer.capacity() >= (missing + self.inflatable_buffer.len()) {
                let inflatable_buffer_len = self.inflatable_buffer.len();
                // copy the data to the front, so we can allocate the least amount of data,
                // or, skip the allocation altogether if we're lucky :)
                self.inflatable_buffer
                    .copy_within(self.position_in_buffer..inflatable_buffer_len, 0);
                self.position_in_buffer = 0;
            }
        }

        // Check what the length would be of our new buffer, and resize if needed
        let new_length = self.position_in_buffer + amount;
        if new_length != self.inflatable_buffer.len() {
            self.inflatable_buffer.resize(new_length, 0);
        }

        // read the missing data
        let actually_read = self
            .inner
            .read(&mut self.inflatable_buffer[self.position_in_buffer + buffered_data_length..])?;
        // if e.g. end of file or reading from network stream
        if actually_read != missing {
            // will always be shorter
            self.inflatable_buffer
                .truncate(self.position_in_buffer + buffered_data_length + actually_read);
        }

        Ok(&self.inflatable_buffer[self.position_in_buffer
            ..self.position_in_buffer + buffered_data_length + actually_read])
    }
}

impl<R: Read> Read for InflatableReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let fulfilled: usize = if self.inflatable_buffer.len() != self.position_in_buffer {
            let to_consume = min(
                buf.len(),
                self.inflatable_buffer.len() - self.position_in_buffer,
            );
            buf[..to_consume].copy_from_slice(
                &self.inflatable_buffer
                    [self.position_in_buffer..to_consume + self.position_in_buffer],
            );
            to_consume
        } else {
            0
        };

        if buf.len() - fulfilled == 0 {
            self.position_in_buffer += fulfilled;
            return Ok(fulfilled);
        }

        match self.inner.read(&mut buf[fulfilled..]) {
            Ok(size) => {
                self.position_in_buffer += fulfilled;
                Ok(fulfilled + size)
            }
            err => err,
        }
    }
}

#[derive(Debug)]
/// Light wrapper around a reader, which allows us to store state
pub struct RecordingHeaderReader<R> {
    /// Inner reader, wrapped in an inflatable reader for peeking support
    inner: InflatableReader<R>,
    /// Current state tracker by reader, stores version and map info
    state: RecordingState,
}

impl<R> RecordingHeaderReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner: InflatableReader::new(inner),
            state: Default::default(),
        }
    }

    pub fn version(&self) -> f32 {
        self.state.version
    }

    pub fn game_version(&self) -> GameVersion {
        self.state.game_version
    }

    pub fn variant(&self) -> GameVariant {
        self.state.variant
    }

    pub fn tile_count(&self) -> usize {
        self.state.tile_count
    }

    pub fn map_width(&self) -> u32 {
        self.state.map_width
    }

    pub fn map_height(&self) -> u32 {
        self.state.map_height
    }

    pub fn num_players(&self) -> u16 {
        self.state.num_players
    }

    pub fn set_version<V: Into<GameVersion>>(&mut self, game_version: V, version: f32) {
        let game_version = game_version.into();
        // Should we actually throw here or smth?
        if let Some(variant) = GameVariant::resolve_variant(&game_version, version) {
            self.state.variant = variant;
        }

        self.state.version = version;
        self.state.game_version = game_version;
    }

    pub fn set_map_size(&mut self, width: u32, height: u32) {
        self.state.map_width = width;
        self.state.map_height = height;
        self.state.tile_count = width as usize * height as usize;
    }

    pub fn set_num_players(&mut self, num_players: u16) {
        self.state.num_players = num_players
    }
}

impl<R: Read> Read for RecordingHeaderReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read> Peek for RecordingHeaderReader<R> {
    fn peek(&mut self, amount: usize) -> io::Result<&[u8]> {
        self.inner.peek(amount)
    }
}

#[derive(Copy, Clone, Debug)]
struct RecordingState {
    version: f32,
    game_version: GameVersion,
    variant: GameVariant,
    num_players: u16,
    map_width: u32,
    map_height: u32,
    /// width * height, for ease of use
    tile_count: usize,
}

impl Default for RecordingState {
    fn default() -> Self {
        RecordingState {
            version: 0.0,
            game_version: Default::default(),
            variant: GameVariant::Trial,
            num_players: 0,
            map_width: 0,
            map_height: 0,
            tile_count: 0,
        }
    }
}

mod tests {
    use crate::reader::{InflatableReader, Peek};
    use std::io::{Cursor, Read};

    #[test]
    pub fn test_inflatable_buffer() {
        let data = (0..20).into_iter().collect::<Vec<_>>();
        let cursor = Cursor::new(data);
        let mut inflatable_buffer = InflatableReader::new_with_capacity(cursor, 4);
        let mut buffer = [0; 4];
        inflatable_buffer.read(&mut buffer).expect("Failed to read");
        assert_eq!(&[0, 1, 2, 3], &buffer);
        assert_eq!(&[4, 5], inflatable_buffer.peek(2).expect("Failed to peek"));
        assert_eq!(
            &[4, 5, 6, 7],
            inflatable_buffer.peek(4).expect("Failed to peek")
        );
        assert_eq!(
            &[4, 5, 6, 7, 8, 9],
            inflatable_buffer.peek(6).expect("Failed to peek")
        );
        let mut buffer = [0; 8];
        inflatable_buffer.read(&mut buffer).expect("Failed to read");
        assert_eq!(&[4, 5, 6, 7, 8, 9, 10, 11], &buffer);
    }
}
