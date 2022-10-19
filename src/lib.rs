use std::{collections::HashMap, io, time::SystemTime};

use memmap::Mmap;

pub fn readwords<'a>(
    file: &'a Mmap,
    bits_to_index: &mut HashMap<u32, usize>,
    index_to_bits: &mut Vec<u32>,
    index_to_word: &mut Vec<&'a [u8]>,
    letter_to_words_bits: &mut [Vec<u32>; 26],
    lettermask: &mut [u32; 26],
) -> io::Result<()> {
    struct Frequency {
        pub count: u32,
        pub letter: u8,
    }

    let now: SystemTime = SystemTime::now();

    let mut freq: [Frequency; 26] = array_init::array_init(|i: usize| Frequency {
        count: 0,
        letter: i as u8,
    });

    // read words
    let mut word_begin: usize = 0;
    let mut bits: u32 = 0;
    for (i, char) in file.iter().enumerate() {
        let char = *char;
        // _technically_ this loop will not work for the last word
        // In practice the last word has a duplicate letter so we don't care
        if char != b'\n' {
            bits |= 1 << (char as u32 - 'a' as u32);
            continue;
        }

        let len: usize = i - word_begin;
        let this_bits = bits;
        word_begin = i + 1;
        bits = 0;

        if len != 5 {
            continue;
        }

        if this_bits.count_ones() as usize != 5 {
            // Skip words with repeated letters
            continue;
        }

        if bits_to_index.contains_key(&this_bits) {
            // Skip anagrams
            continue;
        }

        // count letter frequency
        let slice: &[u8] = &file[i - 5..i];
        for c in slice.iter() {
            let index: usize = *c as usize - 'a' as usize;
            freq[index].count += 1;
        }

        bits_to_index.insert(this_bits, index_to_bits.len());
        index_to_bits.push(this_bits);
        index_to_word.push(slice);
    }

    println!("{:5}us Ingested file", now.elapsed().unwrap().as_micros());

    // rearrange letter order based on letter frequency (least used letter gets lowest index)
    freq.sort_by(|a, b| a.count.cmp(&b.count));

    let mut reverseletterorder: [usize; 26] = [0; 26];

    for i in 0..26 {
        lettermask[i] = 1_u32 << freq[i].letter;
        reverseletterorder[freq[i].letter as usize] = i;
    }

    for w in index_to_bits {
        let mut m: u32 = *w;
        let mut min = 26;

        while m != 0 {
            let letter = m.trailing_zeros() as usize;
            min = std::cmp::min(min, reverseletterorder[letter]);

            m ^= 1 << letter;
        }

        letter_to_words_bits[min].push(*w);
    }

    Ok(())
}
