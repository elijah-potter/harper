use std::path::{Path, PathBuf};

use harper_core::{Dictionary, FullDictionary};
use tokio::fs::File;
use tokio::io::{self, AsyncRead, AsyncReadExt, BufReader};
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub async fn save_dict(path: impl AsRef<Path>, dict: impl Dictionary) -> io::Result<()> {
    let file = File::create(path.as_ref()).await?;
    let write = BufReader::new(file);

    write_word_list(dict, write).await?;

    Ok(())
}

async fn write_word_list(dict: impl Dictionary, mut w: impl AsyncWrite + Unpin) -> io::Result<()> {
    let mut cur_str = String::new();

    for word in dict.words_iter() {
        cur_str.clear();
        cur_str.extend(word);

        w.write_all(cur_str.as_bytes()).await?;
        w.write_all(b"\n").await?;
    }

    Ok(())
}

pub async fn load_dict(path: impl AsRef<Path>) -> io::Result<FullDictionary> {
    let file = File::open(path.as_ref()).await?;
    let read = BufReader::new(file);

    dict_from_word_list(read).await
}

/// This function could definitely be optimized to use less memory.
/// Right now, it isn't an issue.
async fn dict_from_word_list(mut r: impl AsyncRead + Unpin) -> io::Result<FullDictionary> {
    let mut str = String::new();

    r.read_to_string(&mut str).await?;

    let mut dict = FullDictionary::new();
    dict.extend_words(str.lines().map(|l| l.chars().collect::<Vec<char>>()));

    Ok(dict)
}
