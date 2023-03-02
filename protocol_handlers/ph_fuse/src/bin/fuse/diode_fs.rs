extern crate libc;

use std::thread;
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::time::Duration;
use std::time::{SystemTime};
use libc::{ENOENT, ENOSYS};
use std::ffi::OsStr;
use fuse::{Filesystem, Request, ReplyAttr, ReplyData, ReplyEntry, ReplyCreate, ReplyOpen, ReplyWrite, ReplyDirectory, ReplyEmpty};
use libc::O_APPEND;
use std::str;

include!("filesystem_item.rs");
include!("filesystem_commit.rs");

pub struct DiodeFS {
    filesystem_items: Arc<Mutex<Vec<FilesystemItem>>>,
    filesystem_commits: Arc<Mutex<Vec<FilesystemCommit>>>
}

impl DiodeFS {
    pub fn new() -> Self {
        // Add mountpoint information
        let filesystem_items: Arc<Mutex<Vec<FilesystemItem>>> = Default::default();
        filesystem_items.lock().unwrap().push(FilesystemItem::new(
            "/".to_string(),
            [].to_vec(),
            1,
            0,
            FileAttr {
                ino: 1,
                size: 0,
                blocks: 0,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
                crtime: SystemTime::now(),
                kind: FileType::Directory,
                perm: 0o755,
                nlink: 0,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
            }
        ));
        // Create commit buffer
        let filesystem_commits: Arc<Mutex<Vec<FilesystemCommit>>> = Arc::new(Mutex::new(vec![]));

        Self {
            filesystem_items,
            filesystem_commits: filesystem_commits.clone()
        }
    }
    pub fn _create(&mut self, name: String, parent_inode: u64, flags: u32, reply: Option<ReplyCreate>) {
        // Add to commits
        self.filesystem_commits.lock().unwrap().push(FilesystemCommit::create(name.clone(), parent_inode, flags));

        // Create the file
        if !name.is_empty() {
            let items_length = self.filesystem_items.lock().unwrap().len();
            let attr = FileAttr {
                ino: items_length as u64+1,
                size: name.len() as u64,
                blocks: 0,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
                crtime: SystemTime::now(),
                kind: FileType::RegularFile,
                perm: 0o644,
                nlink: 0,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: flags,
            };
            self.filesystem_items.lock().unwrap().push(
                FilesystemItem::new(
                    name,
                    [].to_vec(),
                    items_length as u64+1,
                    parent_inode,
                    attr
                )
            );
            if reply.is_some() {
                reply.unwrap().created(&Duration::from_millis(0), &attr, 0, 0, flags);
            }
            return;
        }
        if reply.is_some() {
            reply.unwrap().error(ENOENT);
        }
    }
    pub fn _write(&self, inode: u64, data: Vec<u8>, data_offset: i64, flags: u32, reply: Option<ReplyWrite>) {
        for item in self.filesystem_items.lock().unwrap().iter_mut() {
            if item.get_inode() == inode {
                // Add to commits
                let name = item.get_name();
                self.filesystem_commits.lock().unwrap().push(FilesystemCommit::write(name, inode, data.clone(), data_offset, flags));

                // Get current data from file
                let mut file_data = item.get_data();

                // Check if the existing data buffer needs to be enlarged
                let flag_append = flags & (O_APPEND as u32) == 0;
                if flag_append || data_offset as usize + data.len() > file_data.len() {
                    let new_size = data_offset as usize+data.len();
                    file_data.resize(new_size, 0u8);
                }

                // Copy data to file_data
                for pos in 0..data.len() {
                    let data_byte = data[pos];
                    file_data[data_offset as usize+pos] = data_byte;
                }
                item.set_data(file_data.to_vec());

                // Let the OS know bytes were written
                if reply.is_some() {
                    reply.unwrap().written(data.to_vec().len() as u32);
                }
                return;
            }
        }
        if reply.is_some() {
            reply.unwrap().error(ENOENT);
        }
    }
    pub fn _unlink(&self, name: String, reply: Option<ReplyEmpty>) {
        // Add to commits
        self.filesystem_commits.lock().unwrap().push(FilesystemCommit::unlink(name.clone()));

        // Perform the unlink task
        let mut found: bool = false;
        let mut index: usize = 0;
        for item in self.filesystem_items.lock().unwrap().iter() {
            if item.get_name() == name {
                found = true;
                break;
            }
            index+=1;
        }
        if found {
            self.filesystem_items.lock().unwrap().remove(index);
            if reply.is_some() {
                reply.unwrap().ok();
            }
            return;
        }
        if reply.is_some() {
            reply.unwrap().error(ENOENT);
        }
    }
    pub fn _mkdir(&self, name: String, parent_inode: u64, reply: Option<ReplyEntry>) {
        // Add to commits
        self.filesystem_commits.lock().unwrap().push(FilesystemCommit::mkdir(name.clone(), parent_inode));
        // Think up some attibutes
        let items_length = self.filesystem_items.lock().unwrap().len();
        let attr = FileAttr {
            ino: items_length as u64+1,
            size: name.len() as u64,
            blocks: 0,
            atime: SystemTime::now(),
            mtime: SystemTime::now(),
            ctime: SystemTime::now(),
            crtime: SystemTime::now(),
            kind: FileType::Directory,
            perm: 0o644,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        };
        // Add the directory to the list of filesystem items
        self.filesystem_items.lock().unwrap().push(
            FilesystemItem::new(
                name,
                [].to_vec(),
                items_length as u64+1,
                parent_inode,
                attr
            )
        );
        // Let the OS know that a directory was created
        if reply.is_some() {
            reply.unwrap().entry(&Duration::from_millis(0), &attr, 0);
        }
    }
    pub fn _rmdir(&self, name: String, reply: Option<ReplyEmpty>) {
        let mut found = false;
        let mut item_index = 0;
        let mut item_inode = 0;
        for item in self.filesystem_items.lock().unwrap().iter() {
            let _item_name = item.get_name();
            if item.get_name() == name {
                found = true;
                item_inode = item.get_inode();
                break;
            }
            item_index+=1;
        }
        if found {
            let mut directory_empty = true;
            // Check if the directory is empty
            for item in self.filesystem_items.lock().unwrap().iter() {
                if item.get_parent_inode() == item_inode {
                    directory_empty = false;
                    break;
                }
            }
            // If the directory is empty, delete it
            if directory_empty {
                self.filesystem_items.lock().unwrap().remove(item_index);
                if reply.is_some() {
                    reply.unwrap().ok();
                }
                return;
            }
        }
        // Reply with "function not implemented".
        // TODO: auto delete a folder recursively,
        // since UI would be more intuitive that way.
        if reply.is_some() {
            reply.unwrap().error(ENOSYS);
        }
    }
    pub fn _rename(&mut self, name: String, parent_inode: u64, new_name: String, new_parent_inode: u64, reply: Option<ReplyEmpty>) {
        // Add to commits
        self.filesystem_commits.lock().unwrap().push(FilesystemCommit::rename(name.clone(), parent_inode, new_name.clone(), new_parent_inode));
        // Rename the file
        for item in self.filesystem_items.lock().unwrap().iter_mut() {
            let name = item.get_name();
            if item.get_name() == name && item.get_parent_inode() == parent_inode {
                item.set_name(new_name);
                item.set_parent_inode(new_parent_inode);
                break;
            }
        }
        // Let the OS know we did some work
        if reply.is_some() {
            reply.unwrap().ok();
        }
    }
    pub fn get_filesystem_items(&self) -> Arc<Mutex<Vec<FilesystemItem>>> {
        self.filesystem_items.clone()
    }
    pub fn get_filesystem_commits(&self) -> Arc<Mutex<Vec<FilesystemCommit>>> {
        self.filesystem_commits.clone()
    }
}

impl Filesystem for DiodeFS {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        for item in self.filesystem_items.lock().unwrap().iter() {
            if item.get_inode() == ino {
                reply.attr(&Duration::from_millis(0), &item.get_attr());
                return;
            }
        }
        reply.error(ENOENT);
    }
    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if offset == 0 {
            reply.add(1, 0, FileType::Directory, Path::new("."));
            reply.add(1, 1, FileType::Directory, Path::new(".."));
            for item in self.filesystem_items.lock().unwrap().iter() {
                if item.get_name() == "/" {
                    continue;
                }
                if item.get_parent_inode() == ino {
                    reply.add(item.get_inode(), offset+2, item.get_filetype(), Path::new(&item.get_name()));
                }
            }
            reply.ok();
            return;
        }
        reply.error(ENOENT);
    }
    fn lookup(&mut self, _req: &Request, _parent: u64, name: &OsStr, reply: ReplyEntry) {
        for item in self.filesystem_items.lock().unwrap().iter() {
            let _item_name = item.get_name();
            if item.get_name() == name.to_str().unwrap() {
                let attr: FileAttr = item.get_attr();
                reply.entry(&Duration::from_millis(0), &attr, 0);
                return;
            }
        }
        reply.error(ENOENT);
    }
    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, size: u32, reply: ReplyData) {
        for item in self.filesystem_items.lock().unwrap().iter() {
            if item.get_inode() == ino {
                // Prevent out of bounds errors by checking if the requested chunk is too large
                if item.get_data().len() > offset as usize + size as usize {
                    reply.data(&item.get_data()[offset as usize..offset as usize+size as usize]);
                } else {
                    reply.data(&item.get_data()[offset as usize..item.get_data().len()]);
                }
                return;
            }
        }
        reply.error(ENOENT);
    }
    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        for item in self.filesystem_items.lock().unwrap().iter() {
            if item.get_inode() == ino {
                reply.opened(item.get_inode(), flags);
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn create(&mut self, _req: &Request, parent: u64, name: &OsStr, _mode: u32, flags: u32, reply: ReplyCreate) {
        self._create(name.to_str().unwrap().to_string().clone(), parent, flags, Some(reply));
    }
    fn write(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, data: &[u8], flags: u32, reply: ReplyWrite) {
        self._write(ino, data.to_vec(), offset, flags, Some(reply));
    }
    fn unlink(&mut self, _req: &Request, _parent: u64, name: &OsStr, reply: ReplyEmpty) {
        self._unlink(name.to_str().unwrap().to_string(), Some(reply));
    }
    fn mkdir(&mut self, _req: &Request, parent: u64, name: &OsStr, _mode: u32, reply: ReplyEntry) {
        self._mkdir(name.to_str().unwrap().to_string(), parent, Some(reply));
    }
    fn rmdir(&mut self, _req: &Request, _parent: u64, name: &OsStr, reply: ReplyEmpty) {
        self._rmdir(name.to_str().unwrap().to_string(), Some(reply));
    }
    fn rename(&mut self, _req: &Request, parent_inode: u64, name: &OsStr, new_parent_inode: u64, new_name: &OsStr, reply: ReplyEmpty) {
        self._rename(name.to_str().unwrap().to_string(), parent_inode, new_name.to_str().unwrap().to_string(), new_parent_inode, Some(reply));
    }
}