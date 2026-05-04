use sled::{Db, Tree};
use crate::models::FileRecord;
use std::path::Path;
use std::io;
use bincode;

#[derive(Clone)]
pub struct Storage {
    db: Db,
    metadata: Tree,
    doc_index: Tree,
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let db = sled::open(path)?;
        let metadata = db.open_tree("metadata")?;
        let doc_index = db.open_tree("doc_index")?;

        Ok(Self {
            db,
            metadata,
            doc_index,
        })
    }

    /// Store a file record and update the language index
    #[allow(dead_code)]
    pub fn insert_record(&self, record: &FileRecord) -> io::Result<()> {
        let path_bytes = record.path.as_bytes();
        let encoded_record = bincode::serialize(record)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // metadata: path -> record
        self.metadata.insert(path_bytes, encoded_record.as_slice())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // doc_index: language | path -> ()
        let mut lang_key = record.language.as_bytes().to_vec();
        lang_key.extend_from_slice(b"|");
        lang_key.extend_from_slice(path_bytes);

        self.doc_index.insert(&lang_key, &[])
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }

    pub fn get_record(&self, path: &str) -> io::Result<Option<FileRecord>> {
        let res = self.metadata.get(path.as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        match res {
            Some(bytes) => {
                let record = bincode::deserialize::<FileRecord>(&bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(Some(record))
            }
            None => Ok(None),
        }
    }

    pub fn get_by_language(&self, lang: &str) -> io::Result<Vec<String>> {
        let mut prefix = lang.as_bytes().to_vec();
        prefix.push(b'|');

        let mut paths = Vec::new();
        for item in self.doc_index.scan_prefix(&prefix) {
            let (key, _) = item.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            if let Some(pos) = key.iter().position(|&b| b == b'|') {
                let path = String::from_utf8_lossy(&key[pos + 1..]).into_owned();
                paths.push(path);
            }
        }
        Ok(paths)
    }

    pub fn batch_insert(&self, records: Vec<FileRecord>) -> io::Result<()> {
        for record in records {
            let path_bytes = record.path.as_bytes();
            let encoded_record = bincode::serialize(&record)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            self.metadata
                .insert(path_bytes, encoded_record.as_slice())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let mut lang_key = record.language.as_bytes().to_vec();
            lang_key.extend_from_slice(b"|");
            lang_key.extend_from_slice(path_bytes);
            self.doc_index
                .insert(&lang_key, &[] as &[u8])
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }

        self.db.flush().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }
}
