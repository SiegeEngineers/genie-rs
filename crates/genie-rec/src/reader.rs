use crate::{GameVariant, GameVersion};
use std::io::{BufRead, BufReader, Read};
use std::{cmp::min, io::Seek};
use std::{io, slice};

pub trait Peek {
    fn peek(&mut self, amount: usize) -> io::Result<&[u8]>;
}

#[derive(Debug)]
/// Light wrapper around a reader, which allows us to store state
pub struct RecordingHeaderReader<R> {
    /// Inner reader, BufReader with peeking support
    inner: BufReader<R>,
    /// Current state tracker by reader, stores version and map info
    state: RecordingState,
    /// Our current position in the header
    position: usize,
}

impl<R: Read> RecordingHeaderReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner: BufReader::new(inner),
            state: Default::default(),
            position: 0,
        }
    }

    pub fn position(&self) -> usize {
        self.position
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
        let read = self.inner.read(buf)?;
        self.position += read;
        Ok(read)
    }
}

impl<R: Read> Peek for RecordingHeaderReader<R> {
    fn peek(&mut self, amount: usize) -> io::Result<&[u8]> {
        self.inner.peek(amount)
    }
}

impl<R: Read> Peek for BufReader<R> {
    fn peek(&mut self, amount: usize) -> io::Result<&[u8]> {
        let buffer = self.buffer();
        Ok(&buffer[..amount])
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

#[cfg(test)]
mod tests {
    use crate::reader::Peek;
    use std::io::{BufReader, Cursor, Read};

    #[test]
    pub fn test_inflatable_buffer() {
        let data = (0..20).into_iter().collect::<Vec<_>>();
        let cursor = Cursor::new(data);
        let mut buffered_reader = BufReader::with_capacity(10, cursor);

        let mut buffer = [0; 4];
        buffered_reader
            .read_exact(&mut buffer)
            .expect("Failed to read");
        assert_eq!(&[0, 1, 2, 3], &buffer);
        assert_eq!(&[4, 5], buffered_reader.peek(2).expect("Failed to peek"));
        assert_eq!(
            &[4, 5, 6, 7],
            buffered_reader.peek(4).expect("Failed to peek")
        );
        assert_eq!(
            &[4, 5, 6, 7, 8, 9],
            buffered_reader.peek(6).expect("Failed to peek")
        );
        assert_eq!(&[4, 5, 6], buffered_reader.peek(3).expect("Failed to peek"));

        let mut buffer = [0; 8];
        buffered_reader
            .read_exact(&mut buffer)
            .expect("Failed to read");
        assert_eq!(&[4, 5, 6, 7, 8, 9, 10, 11], &buffer);
    }
}
