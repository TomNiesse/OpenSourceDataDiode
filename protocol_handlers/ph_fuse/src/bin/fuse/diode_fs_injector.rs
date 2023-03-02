extern crate libc;
use crate::diode_fs::{FilesystemItem, FilesystemCommit, CommitType};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime};
use fuse::{FileAttr, FileType};
use libc::O_APPEND;

pub struct DiodeFSInjector {
    pub filesystem_items: Arc<Mutex<Vec<FilesystemItem>>>,
    pub filesystem_commits: Arc<Mutex<Vec<FilesystemCommit>>>
}

impl DiodeFSInjector {
    pub fn handle_commit(&mut self, commit: FilesystemCommit) {
        match commit.get_commit_type() {
            CommitType::Create => {
                let filename = commit.get_name();
                let parent_inode = commit.get_parent_inode().unwrap();
                let flags = commit.get_flags().unwrap();
                self.create(filename, parent_inode, flags);
            },
            CommitType::Write => {
                let inode = commit.get_inode().unwrap();
                let data = commit.get_data().unwrap();
                let data_offset = commit.get_data_offset().unwrap();
                let flags = commit.get_flags().unwrap();
                self.write(inode, data, data_offset, flags);
            },
            CommitType::Unlink => {
                let filename = commit.get_name();
                self.unlink(filename);
            }
            CommitType::Mkdir => {
                let dirname = commit.get_name();
                let parent_inode = commit.get_parent_inode().unwrap();
                self.mkdir(dirname, parent_inode);
            }
            CommitType::Rmdir => {
                let dirname = commit.get_name();
                self.rmdir(dirname);
            }
            CommitType::Rename => {
                let name = commit.get_name();
                let parent_inode = commit.get_parent_inode().unwrap();
                let new_name = commit.get_new_name().unwrap();
                let new_parent_inode = commit.get_new_parent_inode().unwrap();
                self.rename(name, parent_inode, new_name, new_parent_inode);
            }
            _ => todo!()
        }
    }
    pub fn create(&mut self, name: String, parent_inode: u64, flags: u32) {
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
            return;
        }
    }
    pub fn write(&self, inode: u64, data: Vec<u8>, data_offset: i64, flags: u32) {
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
                return;
            }
        }
    }
    pub fn unlink(&self, name: String) {
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
        }
    }
    pub fn mkdir(&self, name: String, parent_inode: u64) {
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
    }
    pub fn rmdir(&self, name: String) {
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
            }
        }
    }
    pub fn rename(&mut self, name: String, parent_inode: u64, new_name: String, new_parent_inode: u64) {
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
    }
}