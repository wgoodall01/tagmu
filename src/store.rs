use crate::id::Id;
use sled;
use sled::{IVec, TransactionError};
use snafu::{Backtrace, ResultExt, Snafu};
use std::convert::{TryFrom, TryInto};

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("Storage error: {}", source))]
    #[snafu(context(false))]
    StorageError { source: sled::Error },

    #[snafu(display("in tx: {}", source))]
    #[snafu(context(false))]
    FailedTransaction { source: TransactionError },

    #[snafu(display("key \"{}\" not found", key))]
    NotFound { key: u64 },

    #[snafu(display("value \"{}\" not found", val))]
    ValueNotFound { val: String },

    #[snafu(display("Internal error"))]
    InternalError {},
}

pub struct Store {
    sled: sled::Db,

    // Information governing tags
    tag_id_names: sled::Tree,
    tag_name_ids: sled::Tree,

    // Forward and reverse tag indices
    tag_items: sled::Tree,
    item_tags: sled::Tree,
}

type Result<T, E = Error> = std::result::Result<T, E>;

impl Store {
    pub fn open(path: String) -> Result<Store> {
        let config = sled::Config::new().path(path);
        Self::from_sled(config)
    }

    pub fn open_temporary() -> Result<Store> {
        let config = sled::Config::new().temporary(true);
        Self::from_sled(config)
    }

    fn from_sled(config: sled::Config) -> Result<Store> {
        let sled = config
            .cache_capacity(10 * 1000 * 1000 /* 10 MiB */)
            .open()?;

        // Open the tag indices
        let tag_id_names = sled.open_tree("tag_id_names")?;
        let tag_name_ids = sled.open_tree("tag_name_ids")?;

        // Open the forward and reverse indices
        let tag_items = sled.open_tree("tag_items")?;
        let item_tags = sled.open_tree("item_tags")?;

        Ok(Store {
            sled,
            tag_items,
            item_tags,
            tag_id_names,
            tag_name_ids,
        })
    }

    pub fn id(&self) -> Result<u64> {
        Ok(self.sled.generate_id()?)
    }

    pub fn tag_string(&mut self, item: ItemID, tag_name: &str) -> Result<()> {
        // Look up the right tag
        let found_tag: Option<IVec> = self.tag_name_ids.get(tag_name.as_bytes())?;

        let tag: TagID;
        if let Some(tag_vec) = found_tag {
            tag = TagID::from(must_u8_8(&tag_vec)?);
        } else {
            // If the tag doesn't exist, create it.
            tag = TagID::from(self.id()?);
            self.update_tag(tag, tag_name)?;
        }

        // Tag the item with it
        self.tag(item, tag)
    }

    pub fn get_tag_id(&self, tag_name: &str) -> Result<Option<TagID>> {
        let found_tag: Option<IVec> = self.tag_name_ids.get(tag_name.as_bytes())?;

        match found_tag {
            None => Ok(None),
            Some(vec) => Ok(Some(TagID::from(must_u8_8(&vec)?))),
        }
    }

    pub fn tag(&mut self, item: ItemID, tag: TagID) -> Result<()> {
        self.tag_items.insert(compound_key(tag, item), &[])?;
        self.item_tags.insert(compound_key(item, tag), &[])?;

        Ok(())
    }

    pub fn untag(&mut self, item: ItemID, tag: TagID) -> Result<()> {
        self.tag_items.remove(compound_key(tag, item))?;
        self.item_tags.remove(compound_key(item, tag))?;

        Ok(())
    }

    pub fn update_tag(&mut self, id: TagID, name: &str) -> Result<Tag> {
        let tag = Tag {
            id,
            name: name.into(),
        };

        self.tag_id_names.insert(&id.to_bytes(), name.as_bytes())?;
        self.tag_name_ids.insert(name.as_bytes(), &id.to_bytes())?;

        Ok(tag)
    }

    pub fn remove_tag(&mut self, id: TagID) -> Result<()> {
        let removed: Option<IVec> = self.tag_id_names.remove(&id.to_bytes())?;

        let old_name: IVec = match removed {
            Some(name) => name,
            None => return Err(Error::NotFound { key: id.into() }),
        };

        self.tag_name_ids.remove(old_name)?;

        Ok(())
    }

    pub fn get_item_tag_ids(&self, id: ItemID) -> impl Iterator<Item = Result<TagID>> + '_ {
        let item_tags_iter = self.item_tags.scan_prefix(id.to_bytes());

        item_tags_iter.map(move |el| -> Result<TagID> {
            // Get the tag key from the compound key
            let (key_vec, _val) = el?;
            let (_item_id, tag_id): (ItemID, TagID) = from_compound_key(&must_u8_16(&key_vec)?);
            Ok(tag_id)
        })
    }

    pub fn get_item_tags(&self, id: ItemID) -> impl Iterator<Item = Result<Tag>> + '_ {
        let tags_iter = self.get_item_tag_ids(id);

        tags_iter.map(move |tag_result| -> Result<Tag> {
            let tag_id = tag_result?;

            // Join to get the tag name
            let tag_vec = self
                .tag_id_names
                .get(tag_id.to_bytes())?
                .ok_or(Error::InternalError {})?;

            let tag_name: &str = std::str::from_utf8(&tag_vec)
                .map_err(|_| snafu::NoneError)
                .context(InternalError)?;

            Ok(Tag {
                id: tag_id,
                name: tag_name.to_string(),
            })
        })
    }

    pub fn get_tag_item_ids(&self, id: TagID) -> impl Iterator<Item = Result<ItemID>> + '_ {
        let tag_items_iter = self.tag_items.scan_prefix(id.to_bytes());

        tag_items_iter.map(move |el| -> Result<ItemID> {
            // Get the item key from the compound key
            let (key_vec, _val) = el?;
            let (_tag_id, item_id): (TagID, ItemID) = from_compound_key(&must_u8_16(&key_vec)?);

            Ok(item_id)
        })
    }
}

fn compound_key<T1: Id, T2: Id>(a: T1, b: T2) -> [u8; 16] {
    let a_bytes: [u8; 8] = a.into();
    let b_bytes: [u8; 8] = b.into();
    let mut dest: [u8; 16] = [0; 16];

    for i in 0..8 {
        dest[i] = a_bytes[i];
        dest[8 + i] = b_bytes[i];
    }

    dest
}

fn from_compound_key<T1: Id, T2: Id>(compound: &[u8; 16]) -> (T1, T2) {
    let mut a_bytes: [u8; 8] = [0u8; 8];
    let mut b_bytes: [u8; 8] = [0u8; 8];

    for i in 0..8 {
        a_bytes[i] = compound[i];
        b_bytes[i] = compound[8 + i];
    }

    (T1::from(a_bytes), T2::from(b_bytes))
}

fn must_u8_16(slice: &[u8]) -> Result<[u8; 16]> {
    let arr: [u8; 16] = slice
        .try_into()
        .map_err(|_| snafu::NoneError)
        .context(InternalError)?;
    Ok(arr)
}

fn must_u8_8(slice: &[u8]) -> Result<[u8; 8]> {
    let arr: [u8; 8] = slice
        .try_into()
        .map_err(|_| snafu::NoneError)
        .context(InternalError)?;
    Ok(arr)
}

generate_id!(TagID);
generate_id!(ItemID);

#[derive(Debug, PartialEq, Eq)]
pub struct Tag {
    pub id: TagID,
    pub name: String,
}
