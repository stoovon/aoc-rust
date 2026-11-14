use svutils::{glyph_decoder, SolveError};

const WIDTH: usize = 25;
const HEIGHT: usize = 6;
const LAYER_SIZE: usize = WIDTH * HEIGHT;

fn layers(input: &str) -> Result<std::slice::ChunksExact<'_, u8>, SolveError> {
    let pixels = input.trim().as_bytes();
    if pixels.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let layers = pixels.chunks_exact(LAYER_SIZE);
    if !layers.remainder().is_empty() {
        return Err(SolveError::InvalidInput("incomplete image layer".into()));
    }
    if !pixels.iter().all(|pixel| matches!(pixel, b'0'..=b'2')) {
        return Err(SolveError::InvalidInput(
            "expected pixel digits 0, 1 or 2".into(),
        ));
    }
    Ok(layers)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let fewest_zeros = layers(input)?
        .map(|layer| {
            let mut counts = [0; 3];
            for &pixel in layer {
                counts[usize::from(pixel - b'0')] += 1;
            }
            counts
        })
        .min_by_key(|counts| counts[0])
        .ok_or(SolveError::EmptyInput)?;
    Ok((fewest_zeros[1] * fewest_zeros[2]).to_string())
}

fn composite(layers: std::slice::ChunksExact<'_, u8>) -> [[bool; WIDTH]; HEIGHT] {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            layers
                .clone()
                .map(|layer| layer[row * WIDTH + column])
                .find(|&pixel| pixel != b'2')
                == Some(b'1')
        })
    })
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    glyph_decoder::decode(&composite(layers(input)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_opaque_pixel_wins() {
        let mut front = vec![b'2'; LAYER_SIZE];
        let mut back = front.clone();
        front[..4].copy_from_slice(b"0122");
        back[..4].copy_from_slice(b"1012");
        front.extend(back);
        let input = String::from_utf8(front).unwrap();
        assert_eq!(
            composite(layers(&input).unwrap())[0][..4],
            [false, true, true, false]
        );
    }

    #[test]
    fn checksum_uses_the_first_layer_with_the_fewest_zeroes() {
        let input = format!(
            "{}{}{}{}",
            "0".repeat(LAYER_SIZE),
            "1",
            "2".repeat(LAYER_SIZE - 1),
            "1".repeat(LAYER_SIZE)
        );
        assert_eq!(part1(&input).unwrap(), "149");
    }

    #[test]
    fn rejects_incomplete_layers_and_invalid_pixels() {
        for solve in [part1, part2] {
            assert!(matches!(solve(""), Err(SolveError::EmptyInput)));
            assert!(matches!(solve("02"), Err(SolveError::InvalidInput(_))));
            assert!(matches!(
                solve(&"3".repeat(LAYER_SIZE)),
                Err(SolveError::InvalidInput(_))
            ));
        }
    }
}
