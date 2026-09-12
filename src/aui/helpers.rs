// # helpers / common functions for Abstract User Interface

use anstream::stream::{AsLockedWrite, RawStream};
use anstream::AutoStream;
use clap::ColorChoice;

/// Create an [anstream::AutoStream] from a [clap::ColorChoice]
///
/// AutoStream strips ANSI codes automatically when the output is
/// not a TTY or when NO_COLOR / --no-color is set.
///
/// # Example
/// ```norun
/// use clap::ColorChoice;
/// use free_surface::aui::helpers::color_choice_to_stream;
///
/// let mut stdio = std::io::stdout();
/// let stream = color_choice_to_stream(stdio, ColorChoice::Auto);
/// ```
pub fn color_choice_to_stream<'a, W>(out: W, color: ColorChoice) -> AutoStream<W>
where
    W: RawStream + AsLockedWrite + 'a,
{
    match color {
        ColorChoice::Always => AutoStream::always(out),
        ColorChoice::Never => AutoStream::never(out),
        ColorChoice::Auto => AutoStream::auto(out),
    }
}
