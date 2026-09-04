// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Emily Albini <emily@emilyalbini.it>

use anyhow::{Context as _, Error, bail};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use memchr::{memchr2, memchr3};
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Write as _};
use std::path::Path;
use zip::read::ZipFile;
use zip::{ZipArchive, ZipWriter};

pub(crate) fn replace_username(
    archive: &Path,
    output: &Path,
    old: &str,
    new: &str,
) -> Result<(), Error> {
    let archive_dir = archive.file_prefix().unwrap().to_str().unwrap();
    let output_dir = output.file_prefix().unwrap().to_str().unwrap();

    let old = old.as_bytes();
    let new = new.as_bytes();
    if old.len() < new.len() {
        bail!("new username cannot be longer than the old username");
    }

    let mut zip = File::open(archive)
        .map_err(Error::from)
        .and_then(|file| ZipArchive::new(BufReader::new(file)).map_err(Error::from))
        .with_context(|| format!("failed to open {archive:?}"))?;

    let mut dest = ZipWriter::new(BufWriter::new(File::create(output)?));

    for idx in 0..zip.len() {
        let file = zip.by_index(idx)?;
        let file_name = file.name().to_string();
        let dest_name = file_name.replace(&format!("{archive_dir}/"), &format!("{output_dir}/"));
        let file_options = file.options();

        let mut copy_original = true;
        if is_level_dat(&file_name) {
            if let Some(new_content) = replace_in_file(file, old, new)? {
                dest.start_file(&dest_name, file_options)?;
                dest.write_all(&new_content)?;
                copy_original = false;
            }
        } else {
            drop(file);
        }

        if copy_original {
            let mut file = zip.by_index(idx)?;
            dest.start_file(dest_name, file_options)?;
            std::io::copy(&mut file, &mut dest)?;
        }
    }
    Ok(())
}

fn replace_in_file(
    file: ZipFile<'_, BufReader<File>>,
    old: &[u8],
    new: &[u8],
) -> Result<Option<Vec<u8>>, Error> {
    let file_name = file.name().to_string();
    let old_len: u8 = old.len().try_into().context("username is too long")?;

    let mut contents = Vec::new();
    std::io::copy(&mut ZlibDecoder::new(file), &mut contents)?;

    // Find the position at which the username is stored.
    let mut skip = 0;
    let mut offsets = Vec::new();
    loop {
        let position = match old {
            [] => bail!("old username cannot be empty"),
            [chr] => memchr2(1, *chr, &contents[skip..]),
            [chr1, chr2, ..] => memchr3(old_len, *chr1, *chr2, &contents[skip..]),
        };
        if let Some(position) = position {
            let end = position + skip + old.len() + 1;
            if end > contents.len() {
                break;
            }
            if &contents[(position + skip + 1)..end] == old {
                offsets.push(position + skip);
            }
            skip += position + 1;
        } else {
            break;
        }
    }
    if offsets.is_empty() {
        return Ok(None);
    }

    let mut new = new.to_vec();
    while new.len() < old.len() {
        new.push(0);
    }

    for offset in offsets {
        for (idx, chr) in new.iter().enumerate() {
            contents[offset + 1 + idx] = *chr;
        }

        println!("found old username in {file_name} at offset {offset:#x?}",);
    }

    let mut compressor = ZlibEncoder::new(Cursor::new(Vec::new()), Compression::none());
    compressor.write_all(&contents)?;

    Ok(Some(compressor.finish()?.into_inner()))
}

fn is_level_dat(path: &str) -> bool {
    let Some((_directory, file)) = path.split_once('/') else {
        return false;
    };
    let Some(remaining) = file.strip_prefix("level.dat") else {
        return false;
    };
    remaining.chars().all(|c| c.is_ascii_digit())
}
