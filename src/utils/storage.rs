use std::{sync::Mutex, io::{ErrorKind, SeekFrom}};
use eframe::Storage;
use futures::Future;
use tokio::{fs::{OpenOptions, File}, io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt, BufWriter}, task::unconstrained};
use crate::runtime;

pub struct TokioStorage {
    file: Mutex<BufWriter<tokio::fs::File>>
}

impl TokioStorage {
    #[inline]
    pub fn new (app_name: &str) -> Option<Self> {
        return runtime().block_on(async move {
            let path = directories_next::ProjectDirs::from("com", "aandreba", app_name)?;
            let open = OpenOptions::new()
                .read(true)
                .write(true)
                .append(true)
                .create(true)
                .open(path.data_dir())
                .await;

            return match open {
                Ok(file) => Some(Self { file: Mutex::new(BufWriter::new(file)) }),
                Err(e) => {
                    eprintln!("{e}");
                    None
                }
            }
        });
    }
}

impl Storage for TokioStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        let mut file = match self.file.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner()
        };
        
        let fut = async move {
            let mut str = String::new();
            while read_str(&mut file, &mut str, true).await? {
                if &str == key {
                    read_str(&mut file, &mut str, false).await?;
                    return std::io::Result::Ok(Some(str)); 
                }
                skip_next(&mut file).await?;
            }
            return Ok(None)
        };

        return match runtime().block_on(unconstrained(fut)) {
            Ok(Some(result)) => Some(result),
            Ok(None) => None,
            Err(e) => {
                eprintln!("{e}");
                None
            },
        };
    }

    fn set_string(&mut self, key: &str, value: String) {
        let file = match self.file.get_mut() {
            Ok(x) => x,
            Err(e) => e.into_inner()
        };

        let fut = async move {
            // Move to the end of the file
            let mut str = String::new();
            while read_str(file, &mut str, true).await? {
                if &str == key {
                    let current_len = read_len(file).await?;
                    return match current_len.checked_sub(value.len()) {
                        // Equal
                        Some(0) => {
                            file.write_all(value.as_bytes()).await
                        },

                        // current < new
                        Some(_) => {
                            let digit_len = core::mem::size_of::<usize>() as u64;
                            let digit_len_i64;
                            cfg_if::cfg_if! {
                                if #[cfg(debug_assertions)] {
                                    digit_len_i64 = match i64::try_from(digit_len) {
                                        Ok(x) => -x,
                                        Err(e) => return Err(std::io::Error::new(ErrorKind::InvalidData, e))
                                    }
                                } else {
                                    digit_len_i64 = -(digit_len as i64);
                                }
                            }

                            // Write first segment
                            file.write_all(&value.as_bytes()[..current_len]).await?;

                            // Set new len
                            let len_pos = file.seek(SeekFrom::Current(digit_len_i64)).await?;
                            write_len(file, value.len()).await?;
                            let str_pos = len_pos + digit_len;

                            // Read remaining bytes
                            let segment_pos = file.seek(SeekFrom::Start(str_pos + (current_len as u64))).await?;
                            let mut remaining = str.into_bytes();
                            file.read_to_end(&mut remaining).await?;

                            // Write last segment
                            file.seek(SeekFrom::Start(segment_pos)).await?;
                            file.write_all(&value.as_bytes()[current_len..]).await?;

                            // Write bytes shifted
                            file.seek(SeekFrom::Start(str_pos + (value.len() as u64))).await?;
                            return file.write_all(&remaining).await
                        },

                        // current > new
                        None => {
                            let digit_len = core::mem::size_of::<usize>() as u64;
                            let digit_len_i64;
                            cfg_if::cfg_if! {
                                if #[cfg(debug_assertions)] {
                                    digit_len_i64 = match i64::try_from(digit_len) {
                                        Ok(x) => -x,
                                        Err(e) => return Err(std::io::Error::new(ErrorKind::InvalidData, e))
                                    }
                                } else {
                                    digit_len_i64 = -(digit_len as i64);
                                }
                            }

                            // Write new data
                            file.write_all(value.as_bytes()).await?;

                            // Set new len
                            let len_pos = file.seek(SeekFrom::Current(digit_len_i64)).await?;
                            write_len(file, value.len()).await?;
                            let str_pos = len_pos + digit_len;

                            // Read remaining bytes
                            file.seek(SeekFrom::Start(str_pos + (current_len as u64))).await?;
                            let mut remaining = str.into_bytes();
                            file.read_to_end(&mut remaining).await?;

                            // Write bytes shifted
                            file.seek(SeekFrom::Start(str_pos + (value.len() as u64))).await?;
                            return file.write_all(&remaining).await
                        }
                    }
                }
                skip_next(file).await?;
            }

            write_len(file, key.len()).await?;
            file.write_all(key.as_bytes()).await?;

            write_len(file, value.len()).await?;
            return file.write_all(value.as_bytes()).await;
        };

        if let Err(e) = runtime().block_on(unconstrained(fut)) {
            eprintln!("{e}")
        }
    }

    #[inline]
    fn flush(&mut self) {
        let file = match self.file.get_mut() {
            Ok(x) => x,
            Err(e) => e.into_inner()
        };
        
        if let Err(e) = runtime().block_on(unconstrained(file.flush())) {
            eprintln!("{e}")
        }
    }
}

async fn read_str (file: &mut BufWriter<File>, str: &mut String, allow_eof: bool) -> std::io::Result<bool> {
    str.clear();
    let len = match read_len(file).await {
        Ok(x) => x,
        Err(e) if allow_eof && e.kind() == ErrorKind::UnexpectedEof => return Ok(false),
        Err(e) => return Err(e),
    };

    str.reserve(len);
    let bytes = unsafe { core::slice::from_raw_parts_mut(str.as_mut_ptr(), len) };
    file.read_exact(bytes).await?;

    if let Err(e) = std::str::from_utf8(bytes) {
        str.clear();
        return Err(
            std::io::Error::new(ErrorKind::InvalidData, e)
        )
    }

    return Ok(true)
}

#[inline]
async fn read_len (file: &mut BufWriter<File>) -> std::io::Result<usize> {
    let len;
    cfg_if::cfg_if! {
        if #[cfg(target_pointer_width = "8")] {
            cfg_if::cfg_if! {
                if #[cfg(target_endian = "big")] {
                    len = file.read_u8().await? as usize;
                } else {
                    len = file.read_u8_le().await? as usize;
                }
            }
        } else if #[cfg(target_pointer_width = "16")] {
            cfg_if::cfg_if! {
                if #[cfg(target_endian = "big")] {
                    len = file.read_u16().await? as usize;
                } else {
                    len = file.read_u16_le().await? as usize;
                }
            }
        } else if #[cfg(target_pointer_width = "32")] {
            cfg_if::cfg_if! {
                if #[cfg(target_endian = "big")] {
                    len = file.read_u32().await? as usize;
                } else {
                    len = file.read_u32_le().await? as usize;
                }
            }
        } else if #[cfg(target_pointer_width = "64")] {
            cfg_if::cfg_if! {
                if #[cfg(target_endian = "big")] {
                    len = file.read_u64().await? as usize;
                } else {
                    len = file.read_u64_le().await? as usize;
                }
            }
        } else {
            compile_error!("Unsupported pointer width");
        }
    }
    return Ok(len)
}

#[inline]
fn write_len<'a> (file: &'a mut BufWriter<File>, v: usize) -> impl 'a + Future<Output = std::io::Result<()>> {
    cfg_if::cfg_if! {
        if #[cfg(target_pointer_width = "8")] {
            cfg_if::cfg_if! {
                if #[cfg(target_endian = "big")] {
                    file.write_u8(v as u8)
                } else {
                    file.write_u8_le(v as u8)
                }
            }
        } else if #[cfg(target_pointer_width = "16")] {
            cfg_if::cfg_if! {
                if #[cfg(target_endian = "big")] {
                    file.write_u16(v as u16)
                } else {
                    file.write_u16_le(v as u16)
                }
            }
        } else if #[cfg(target_pointer_width = "32")] {
            cfg_if::cfg_if! {
                if #[cfg(target_endian = "big")] {
                    file.write_u32(v as u32)
                } else {
                    file.write_u32_le(v as u32)
                }
            }
        } else if #[cfg(target_pointer_width = "64")] {
            cfg_if::cfg_if! {
                if #[cfg(target_endian = "big")] {
                    file.write_u64(v as u64)
                } else {
                    file.write_u64_le(v as u64)
                }
            }
        } else {
            compile_error!("Unsupported pointer width");
        }
    }
}

#[inline]
async fn skip_next (file: &mut BufWriter<File>) -> std::io::Result<()> {
    let len;
    cfg_if::cfg_if! {
        if #[cfg(all(debug_assertions, target_pointer_width = "64"))] {
            len = match i64::try_from(read_len(file).await?) {
                Ok(x) => x,
                Err(e) => return Err(std::io::Error::new(ErrorKind::InvalidData, e))
            };
        } else {
            len = read_len(file).await? as i64;
        }
    }
    
    file.seek(SeekFrom::Current(len)).await?;
    return Ok(())
}