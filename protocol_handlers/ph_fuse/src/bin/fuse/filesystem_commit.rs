#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommitType {
    Create,
    Write,
    Unlink,
    Mkdir,
    Rmdir,
    Rename,
    Invalid
}

#[derive(PartialEq, Eq)]
pub struct FilesystemCommit {
    pub commit_type: CommitType,
    pub name: String,
    pub new_name: Option<String>,
    pub flags: Option<u32>,
    pub inode: Option<u64>,
    pub parent_inode: Option<u64>,
    pub new_parent_inode: Option<u64>,
    pub data_offset: Option<i64>,
    pub data: Option<Vec<u8>>
}

impl FilesystemCommit {
    pub fn create(filename: String, parent_inode: u64, flags: u32) -> Self {
        Self {
            commit_type: CommitType::Create,
            name: filename,
            parent_inode: Some(parent_inode),
            flags: Some(flags),
            // unused fields
            inode: Option::None,
            data_offset: Option::None,
            data: Option::None,
            new_name: Option::None,
            new_parent_inode: Option::None
        }
    }
    pub fn write(filename: String, inode: u64, data: Vec<u8>, data_offset: i64, flags: u32) -> Self {
        Self {
            commit_type: CommitType::Write,
            name: filename,
            inode: Some(inode),
            data_offset: Some(data_offset),
            data: Some(data),
            flags: Some(flags),
            // unused fields
            parent_inode: Option::None,
            new_name: Option::None,
            new_parent_inode: Option::None
        }
    }
    pub fn unlink(filename: String) -> Self {
        Self {
            commit_type: CommitType::Unlink,
            name: filename,
            // unused fields
            inode: Option::None,
            parent_inode: Option::None,
            flags: Option::None,
            data_offset: Option::None,
            data: Option::None,
            new_name: Option::None,
            new_parent_inode: Option::None
        }
    }
    pub fn mkdir(dirname: String, parent_inode: u64) -> Self {
        Self {
            commit_type: CommitType::Mkdir,
            name: dirname,
            parent_inode: Some(parent_inode),
            // unused fields
            inode: Option::None,
            flags: Option::None,
            data_offset: Option::None,
            data: Option::None,
            new_name: Option::None,
            new_parent_inode: Option::None
        }
    }
    pub fn rmdir(dirname: String) -> Self {
        Self {
            commit_type: CommitType::Rmdir,
            name: dirname,
            // unused fields
            inode: Option::None,
            parent_inode: Option::None,
            flags: Option::None,
            data_offset: Option::None,
            data: Option::None,
            new_name: Option::None,
            new_parent_inode: Option::None
        }
    }
    pub fn rename(name: String, parent_inode: u64, new_name: String, new_parent_inode: u64) -> Self {
        Self {
            commit_type: CommitType::Rename,
            name: name,
            parent_inode: Some(parent_inode),
            new_name: Some(new_name),
            new_parent_inode: Some(new_parent_inode),
            // unused fields
            inode: Option::None,
            flags: Option::None,
            data_offset: Option::None,
            data: Option::None
        }
    }

    pub fn get_commit_type(&self) -> CommitType {
        self.commit_type
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_new_name(&self) -> Option<String> {
        self.new_name.clone()
    }
    pub fn get_flags(&self) -> Option<u32> {
        self.flags
    }
    pub fn get_inode(&self) -> Option<u64> {
        self.inode
    }
    pub fn get_parent_inode(&self) -> Option<u64> {
        self.parent_inode
    }
    pub fn get_new_parent_inode(&self) -> Option<u64> {
        self.new_parent_inode
    }
    pub fn get_data_offset(&self) -> Option<i64> {
        self.data_offset
    }
    pub fn get_data(&self) -> Option<Vec<u8>> {
        self.data.clone()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = vec![];

        out.push(self.commit_type as u8);
        out.extend(self.name.len().to_ne_bytes().to_vec());
        out.extend(self.name.as_bytes());
        if self.new_name.is_some() {
            out.extend(self.new_name.as_ref().unwrap().len().to_ne_bytes().to_vec());
            out.extend(self.new_name.as_ref().unwrap().as_bytes());
        }
        if self.flags.is_some() {
            out.extend(self.flags.unwrap().to_ne_bytes().to_vec());
        }
        if self.inode.is_some() {
            out.extend(self.inode.unwrap().to_ne_bytes().to_vec());
        }
        if self.parent_inode.is_some() {
            out.extend(self.parent_inode.unwrap().to_ne_bytes().to_vec());
        }
        if self.new_parent_inode.is_some() {
            out.extend(self.new_parent_inode.unwrap().to_ne_bytes().to_vec());
        }
        if self.data_offset.is_some() {
            out.extend(self.data_offset.unwrap().to_ne_bytes().to_vec());
        }
        if self.data.is_some() {
            out.extend(self.data.as_ref().unwrap());
        }

        out
    }
    pub fn from_bytes(data: Vec<u8>) -> Self {
        let mut data_ptr: usize = 0;
        let commit_type: CommitType = match data[data_ptr] {
            0 => CommitType::Create,
            1 => CommitType::Write,
            2 => CommitType::Unlink,
            3 => CommitType::Mkdir,
            4 => CommitType::Rmdir,
            5 => CommitType::Rename,
            _ => CommitType::Invalid
        };
        data_ptr+=1;
        let name_length = usize::from_ne_bytes(data[data_ptr..data_ptr+8].try_into().unwrap());
        data_ptr+=8;
        let name = str::from_utf8(&data[data_ptr..data_ptr+name_length]).unwrap().to_string();
        data_ptr+=name_length;
        match commit_type {
            CommitType::Create => {
                // Extract flags
                let flags = u32::from_ne_bytes(data[data_ptr..data_ptr+4].try_into().unwrap());
                data_ptr+=4;
                // Extract parent_inode
                let parent_inode = u64::from_ne_bytes(data[data_ptr..data_ptr+8].try_into().unwrap());
                FilesystemCommit::create(name, parent_inode, flags)
            }
            CommitType::Write => {
                // Extract flags
                let flags = u32::from_ne_bytes(data[data_ptr..data_ptr+4].try_into().unwrap());
                data_ptr+=4;
                // Extract inode
                let inode = u64::from_ne_bytes(data[data_ptr..data_ptr+8].try_into().unwrap());
                data_ptr+=8;
                // Extract data_offset
                let data_offset = i64::from_ne_bytes(data[data_ptr..data_ptr+8].try_into().unwrap());
                data_ptr+=8;
                // Extract data
                let _data = data[data_ptr..].to_vec();
                FilesystemCommit::write(name, inode, _data, data_offset, flags)
            }
            CommitType::Unlink => {
                FilesystemCommit::unlink(name)
            }
            CommitType::Mkdir => {
                // Extract parent_inode
                let parent_inode = u64::from_ne_bytes(data[data_ptr..data_ptr+8].try_into().unwrap());
                FilesystemCommit::mkdir(name, parent_inode)
            }
            CommitType::Rmdir => {
                FilesystemCommit::rmdir(name)
            }
            CommitType::Rename => {
                let new_name_length = usize::from_ne_bytes(data[data_ptr..data_ptr+8].try_into().unwrap());
                data_ptr+=8;
                let new_name = str::from_utf8(&data[data_ptr..data_ptr+new_name_length]).unwrap().to_string();
                data_ptr+=new_name_length;
                let parent_inode = u64::from_ne_bytes(data[data_ptr..data_ptr+8].try_into().unwrap());
                data_ptr+=8;
                let new_parent_inode = u64::from_ne_bytes(data[data_ptr..data_ptr+8].try_into().unwrap());
                FilesystemCommit::rename(name, parent_inode, new_name, new_parent_inode)
            }
            CommitType::Invalid => {
                panic!("Received invalid commit type for name: {}", name);
            }
        }
    }
}