use fuse::{FileType, FileAttr};

pub struct FilesystemItem {
    name: String,       // The name of the file or folder
    data: Vec<u8>,      // Contents of the file
    inode: u64,         // Index node
    parent_inode: u64,  // Parent inode (directory the item is in)
    attr: FileAttr      // File attributes
}

impl FilesystemItem {
    pub fn new(name: String, data: Vec<u8>, inode: u64, parent_inode: u64, attr: FileAttr) -> Self {
        Self {
            name,
            data,
            inode,
            parent_inode,
            attr
        }
    }
    pub fn get_filetype(&self) -> FileType {
        self.attr.kind
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data.resize(data.len(), 0u8);
        self.data = data.clone();
        self.attr.size = data.len() as u64;
    }
    pub fn get_inode(&self) -> u64 {
        self.inode
    }
    pub fn set_inode(&mut self, inode: u64) {
        self.inode = inode
    }
    pub fn get_parent_inode(&self) -> u64 {
        self.parent_inode
    }
    pub fn set_parent_inode(&mut self, parent_inode: u64) {
        self.parent_inode = parent_inode
    }
    pub fn get_attr(&self) -> FileAttr {
        self.attr
    }
    pub fn set_attr(&mut self, attr: FileAttr) {
        self.attr = attr;
    }
    pub fn get_size(&self) -> u64 {
        self.attr.size
    }
}