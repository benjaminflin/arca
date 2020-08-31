use super::error::{Error, Result};
use super::volume::Volume;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
/// Serializable File descriptor which follows the ElFinder Protocol
/// {
///     "name"   : "Images",             // (String) name of file/dir. Required
///     "hash"   : "l0_SW1hZ2Vz",        // (String) hash of current file/dir path, first symbol must be letter, symbols before _underline_ - volume id, Required.
///     "phash"  : "l0_Lw",              // (String) hash of parent directory. Required except roots dirs.
///     "mime"   : "directory",          // (String) mime type. Required.
///     "ts"     : 1334163643,           // (Number) file modification time in unix timestamp. Required.
///     "date"   : "30 Jan 2010 14:25",  // (String) last modification time (mime). Depricated but yet supported. Use ts instead.
///     "size"   : 12345,                // (Number) file size in bytes
///     "dirs"   : 1,                    // (Number) Only for directories. Marks if directory has child directories inside it. 0 (or not set) - no, 1 - yes. Do not need to calculate amount.
///     "read"   : 1,                    // (Number) is readable
///     "write"  : 1,                    // (Number) is writable
///     "locked" : 0,                    // (Number) is file locked. If locked that object cannot be deleted,  renamed or moved
///     "tmb"    : 'bac0d45b625f8d4633435ffbd52ca495.png' // (String) Only for images. Thumbnail file name, if file do not have thumbnail yet, but it can be generated than it must have value "1"
///     "alias"  : "files/images",       // (String) For symlinks only. Symlink target path.
///     "thash"  : "l1_c2NhbnMy",        // (String) For symlinks only. Symlink target hash.
///     "dim"    : "640x480",            // (String) For images - file dimensions. Optionally.
///     "isowner": true,                 // (Bool) has ownership. Optionally.
///     "csscls" : "custom-icon",        // (String) CSS class name for holder icon. Optionally. It can include to options.
///     "volumeid" : "l1_",              // (String) Volume id. For directory only. It can include to options.
///     "netkey" : "",                   // (String) Netmount volume unique key, Required for netmount volume. It can include to options.
///     "options" : {}                   // (Object) For volume root only. This value is same to cwd.options.
/// }
#[derive(Serialize)]
pub struct File {
    name: String,
    hash: String,
    phash: Option<String>,
    mime: String,
    ts: u128,
    size: i64,
    dirs: i8,
    read: i8,
    write: i8,
    locked: i8,
    tmb: Option<String>,
    alias: Option<String>,
    thash: Option<String>,
    dim: Option<String>,
    isowner: Option<bool>,
    csscls: Option<String>,
    volumeid: Option<String>,
    netkey: Option<String>,
    options: Option<HashMap<String, String>>,
}

impl File {
    fn check_path(vol: &Volume, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = [vol.path.as_ref(), path.as_ref()]
            .iter()
            .collect::<PathBuf>();
        if path < vol.path {
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::PermissionDenied,
                "Cannot access directory outside of volume root",
            )
            .into());
        }
        Ok(path)
    }
    async fn has_subdirs(path: impl AsRef<Path>) -> Result<bool> {
        let mut dir = tokio::fs::read_dir(path).await?;
        let mut subdirs = false;
        while let Some(dir_entry) = dir.next_entry().await? {
            if dir_entry.file_type().await?.is_dir() {
                subdirs = true;
                break;
            }
        }
        Ok(subdirs)
    }

    async fn parent(vol: &Volume, path: impl AsRef<Path>) -> Result<Option<PathBuf>> {
        let path = Self::check_path(vol, path)?; 
        match tokio::fs::metadata(&path).await {
            Ok(_) => Ok(Some(path.canonicalize()?)),
            Err(err) if err.kind() == tokio::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    pub async fn info(vol: &Volume, path: impl AsRef<Path>) -> Result<Self> {
        let path = Self::check_path(vol, path)?; 
        let metadata = tokio::fs::metadata(&path).await?;
        let name = path
            .canonicalize()?
            .file_name()
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or(String::new());

        let hash = path
            .canonicalize()?
            .strip_prefix(&vol.path)
            .unwrap()
            .to_string_lossy()
            .to_string();

        let ts = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let mime = if metadata.file_type().is_dir() {
            "directory".to_owned()
        } else {
            "file".to_owned() /* TODO */
        };
        let dirs = Self::has_subdirs(&path).await?;
        let readonly = metadata.permissions().readonly();
        let phash = Self::parent(vol, &path)
            .await?
            .as_deref()
            .map(|p| p.to_string_lossy().to_string());
        let size = metadata.len() as i64;
        Ok(Self {
            name,
            hash,
            phash,
            mime,
            ts,
            size,
            dirs: if dirs { 1 } else { 0 },
            read: 1,
            write: if readonly { 0 } else { 1 },
            // TODO
            locked: 0,
            tmb: None,
            alias: None,
            thash: None,
            dim: None,
            isowner: Some(true),
            csscls: None,
            volumeid: None,
            netkey: None,
            options: None,
        })
    }
    pub async fn open_dir<P: AsRef<Path>>(vol: &Volume, path: P) -> Result<Vec<Self>> {
        let path = Self::check_path(vol, path)?; 
        let mut dir = tokio::fs::read_dir(path).await?;
        let mut all_dirs = Vec::new();

        while let Some(dir_entry) = dir.next_entry().await? {
            let metadata = dir_entry.metadata().await?;
            let ts = metadata
                .modified()?
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();
            let readonly = metadata.permissions().readonly();
            let name = dir_entry.file_name().into_string().unwrap_or(String::new());
            let hash = dir_entry
                .path()
                .canonicalize()?
                .strip_prefix(&vol.path)
                .unwrap()
                .to_string_lossy()
                .to_string();
            let mime = if dir_entry.file_type().await?.is_dir() {
                "directory".to_owned()
            } else {
                "file".to_owned() /* TODO */
            };
            let size = metadata.len() as i64;
            let phash = Self::parent(vol, &dir_entry.path())
                .await?
                .as_deref()
                .map(|p| p.to_string_lossy().to_string());
            let dirs = Self::has_subdirs(dir_entry.path()).await?;
            all_dirs.push(Self {
                name,
                hash,
                phash,
                mime,
                ts,
                size,
                dirs: if dirs { 1 } else { 0 },
                read: 1,
                write: if readonly { 0 } else { 1 },
                // TODO
                locked: 0,
                tmb: None,
                alias: None,
                thash: None,
                dim: None,
                isowner: Some(true),
                csscls: None,
                volumeid: None,
                netkey: None,
                options: None,
            });
        }
        Ok(all_dirs)
    }
    pub async fn chmod(vol: &Volume, path: impl AsRef<Path>, perm: std::fs::Permissions) -> Result<File> {
        let orig_path = path;
        let path = Self::check_path(vol, &orig_path)?;        
        tokio::fs::set_permissions(path, perm).await?;
        Self::info(vol, orig_path).await
    }

    pub async fn duplicate(vol: &Volume, path: impl AsRef<Path>) -> Result<File> {
        unimplemented!()
    }
}
