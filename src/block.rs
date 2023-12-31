use crate::{bindings, Note, Transaction};

#[derive(Debug)]
pub struct Blocks<'a> {
    ptr: *mut bindings::ndb_blocks,
    txn: Option<&'a Transaction>,
}

#[derive(Debug)]
pub struct Block<'a> {
    ptr: *mut bindings::ndb_block,
    txn: Option<&'a Transaction>,
}

pub struct BlockIter<'a> {
    iter: bindings::ndb_block_iterator,
    txn: Option<&'a Transaction>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum BlockType {
    Hashtag,
    Text,
    MentionIndex,
    MentionBech32,
    Url,
    Invoice,
}

impl<'a> Block<'a> {
    pub(crate) fn new_transactional(
        ptr: *mut bindings::ndb_block,
        txn: &'a Transaction,
    ) -> Block<'a> {
        Block {
            ptr,
            txn: Some(txn),
        }
    }

    pub(crate) fn new_owned(ptr: *mut bindings::ndb_block) -> Block<'static> {
        Block { ptr, txn: None }
    }

    pub(crate) fn new(ptr: *mut bindings::ndb_block, txn: Option<&'a Transaction>) -> Block<'a> {
        Block { ptr, txn }
    }

    pub fn as_ptr(&self) -> *mut bindings::ndb_block {
        self.ptr
    }

    pub fn as_str(&self) -> &'a str {
        unsafe {
            let str_block = bindings::ndb_block_str(self.as_ptr());
            if str_block.is_null() {
                return "";
            }
            let ptr = bindings::ndb_str_block_ptr(str_block) as *const u8;
            let len = bindings::ndb_str_block_len(str_block);
            let byte_slice = std::slice::from_raw_parts(ptr, len.try_into().unwrap());
            std::str::from_utf8_unchecked(byte_slice)
        }
    }

    pub fn blocktype(&self) -> BlockType {
        let typ = unsafe { bindings::ndb_get_block_type(self.as_ptr()) };
        match typ {
            1 => BlockType::Hashtag,
            2 => BlockType::Text,
            3 => BlockType::MentionIndex,
            4 => BlockType::MentionBech32,
            5 => BlockType::Url,
            6 => BlockType::Invoice,
            _ => panic!("Invalid blocktype {}", typ),
        }
    }
}

impl<'a> Blocks<'a> {
    pub(crate) fn new_transactional(
        ptr: *mut bindings::ndb_blocks,
        txn: &'a Transaction,
    ) -> Blocks<'a> {
        Blocks {
            ptr,
            txn: Some(txn),
        }
    }

    pub(crate) fn new_owned(ptr: *mut bindings::ndb_blocks) -> Blocks<'static> {
        Blocks { ptr, txn: None }
    }

    pub fn iter(&self, note: &Note<'a>) -> BlockIter<'a> {
        let content = note.content_ptr();
        match self.txn {
            Some(txn) => BlockIter::new_transactional(content, self.as_ptr(), txn),
            None => BlockIter::new_owned(content, self.as_ptr()),
        }
    }

    pub fn as_ptr(&self) -> *mut bindings::ndb_blocks {
        self.ptr
    }
}

impl<'a> BlockIter<'a> {
    pub(crate) fn new_transactional(
        content: *const ::std::os::raw::c_char,
        blocks: *mut bindings::ndb_blocks,
        txn: &'a Transaction,
    ) -> BlockIter<'a> {
        let type_ = bindings::ndb_block_type_BLOCK_TEXT;
        let mention_index: u32 = 1;
        let block = bindings::ndb_block__bindgen_ty_1 { mention_index };
        let block = bindings::ndb_block { type_, block };
        let p = blocks as *mut ::std::os::raw::c_uchar;
        let iter = bindings::ndb_block_iterator {
            content,
            blocks,
            p,
            block,
        };
        let mut block_iter = BlockIter {
            iter,
            txn: Some(txn),
        };
        unsafe { bindings::ndb_blocks_iterate_start(content, blocks, &mut block_iter.iter) };
        block_iter
    }

    pub(crate) fn new_owned(
        content: *const ::std::os::raw::c_char,
        blocks: *mut bindings::ndb_blocks,
    ) -> BlockIter<'static> {
        let type_ = bindings::ndb_block_type_BLOCK_TEXT;
        let mention_index: u32 = 1;
        let block = bindings::ndb_block__bindgen_ty_1 { mention_index };
        let block = bindings::ndb_block { type_, block };
        let p = blocks as *mut ::std::os::raw::c_uchar;
        let mut iter = bindings::ndb_block_iterator {
            content,
            blocks,
            p,
            block,
        };
        unsafe { bindings::ndb_blocks_iterate_start(content, blocks, &mut iter) };
        BlockIter { iter, txn: None }
    }

    pub fn as_ptr(&self) -> *const bindings::ndb_block_iterator {
        &self.iter
    }

    pub fn as_mut_ptr(&self) -> *mut bindings::ndb_block_iterator {
        self.as_ptr() as *mut bindings::ndb_block_iterator
    }
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = Block<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let block = unsafe { bindings::ndb_blocks_iterate_next(self.as_mut_ptr()) };
        if block.is_null() {
            return None;
        }

        Some(Block::new(block, self.txn))
    }
}

/*
impl<'a> IntoIterator for Blocks<'a> {
    type Item = Block<'a>;
    type IntoIter = BlockIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.txn {
            Some(txn) => BlockIter::new_transactional(self.as_ptr(), txn),
            None => BlockIter::new_owned(self.as_ptr()),
        }
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;
    use crate::{Config, Ndb};

    #[test]
    fn note_blocks_work() {
        let db = "target/testdbs/note_blocks";

        {
            let mut ndb = Ndb::new(db, &Config::new()).expect("ndb");
            ndb.process_event("[\"EVENT\",\"s\",{\"id\":\"d28ac02e277c3cf2744b562a414fd92d5fea554a737901364735bfe74577f304\",\"pubkey\":\"b5b1b5d2914daa2eda99af22ae828effe98730bf69dcca000fa37bfb9e395e32\",\"created_at\": 1703989205,\"kind\": 1,\"tags\": [],\"content\": \"#hashtags, are neat nostr:nprofile1qqsr9cvzwc652r4m83d86ykplrnm9dg5gwdvzzn8ameanlvut35wy3gpz3mhxue69uhhyetvv9ujuerpd46hxtnfduyu75sw https://github.com/damus-io\",\"sig\": \"07af3062616a17ef392769cadb170ac855c817c103e007c72374499bbadb2fe8917a0cc5b3fdc5aa5d56de086e128b3aeaa8868f6fe42a409767241b6a29cc94\"}]").expect("process ok");
        }

        {
            let ndb = Ndb::new(db, &Config::new()).expect("ndb");
            let id =
                hex::decode("d28ac02e277c3cf2744b562a414fd92d5fea554a737901364735bfe74577f304")
                    .expect("hex id");
            let txn = Transaction::new(&ndb).expect("txn");
            let id_bytes: [u8; 32] = id.try_into().expect("id bytes");
            let note = ndb.get_note_by_id(&txn, &id_bytes).unwrap();
            let blocks = ndb
                .get_blocks_by_key(&txn, note.key().unwrap())
                .expect("note");
            let mut c = 0;
            for block in blocks.iter(&note) {
                match c {
                    0 => {
                        assert_eq!(block.blocktype(), BlockType::Hashtag);
                        assert_eq!(block.as_str(), "hashtags");
                    }

                    1 => {
                        assert_eq!(block.blocktype(), BlockType::Text);
                        assert_eq!(block.as_str(), ", are neat ");
                    }

                    2 => {
                        assert_eq!(block.blocktype(), BlockType::MentionBech32);
                        assert_eq!(block.as_str(), "nprofile1qqsr9cvzwc652r4m83d86ykplrnm9dg5gwdvzzn8ameanlvut35wy3gpz3mhxue69uhhyetvv9ujuerpd46hxtnfduyu75sw");
                    }

                    3 => {
                        assert_eq!(block.blocktype(), BlockType::Text);
                        assert_eq!(block.as_str(), " ");
                    }

                    4 => {
                        assert_eq!(block.blocktype(), BlockType::Url);
                        assert_eq!(block.as_str(), "https://github.com/damus-io");
                    }

                    _ => assert!(false),
                }

                c += 1;
            }
        }

        test_util::cleanup_db(&db);
    }
}