use ie_base::IEBuf;
use wasm_global_shared_data::ParseError;

pub fn parse_port<T>(
    source: &[IEBuf],
    is_optional: bool,
    min_size: u8,
    max_size: Option<u8>,
) -> Result<usize, ParseError> {
    let mut len = 0;
    if source.get(len).is_some() {
        let err = ParseError::NotTerminated;
        while source.get(len).ok_or(err)?.is_valid() {
            len = len.wrapping_add(1);
        }
    }
    if len == 0 && is_optional {
        return Ok(0);
    }
    if len < min_size.into() {
        return Err(ParseError::NotEnoughData);
    }
    if max_size.map_or(false, |m: u8| len > m as usize) {
        return Err(ParseError::TooMuchData);
    }

    Ok(len)
}
