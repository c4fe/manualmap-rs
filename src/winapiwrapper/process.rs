use super::error::Error;
use super::processaccess::ProcessAccess;
use super::snapshot::Snapshot;
use super::snapshotflags::SnapshotFlags;
use super::thread::Thread;
use super::threadaccess::ThreadAccess;
use std::ops::Drop;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetProcessId, OpenProcess};
use winapi::um::winnt::HANDLE;

pub struct Process {
    handle: HANDLE,
}

impl Process {
    pub fn from_pid(pid: u32, access: ProcessAccess, inherit: bool) -> Result<Self, Error> {
        let handle = unsafe { OpenProcess(access.bits(), inherit as i32, pid) };

        if handle.is_null() {
            Err(Error::new("OpenProcess returned NULL".to_string()))
        } else {
            Ok(Self { handle })
        }
    }

    pub fn close(&mut self) -> Result<(), Error> {
        if self.handle.is_null() {
            return Err(Error::new(
                "Null handle passed to Process::close".to_string(),
            ));
        }

        let ret = unsafe { CloseHandle(self.handle) };

        if ret == 0 {
            Err(Error::new("CloseHandle failed".to_string()))
        } else {
            self.handle = 0 as HANDLE;
            Ok(())
        }
    }

    pub fn pid(&self) -> Result<u32, Error> {
        let pid = unsafe { GetProcessId(self.handle) };

        if pid == 0 {
            Err(Error::new("GetProcessId returned NULL".to_string()))
        } else {
            Ok(pid)
        }
    }

    pub fn snapshot(&self, flags: SnapshotFlags) -> Result<Snapshot, Error> {
        Snapshot::from_pid(self.pid()?, flags)
    }

    // FIXME: This returns the first thread of many. Maybe turn it into an iterator?
    pub fn main_thread(
        &self,
        access: ThreadAccess,
        inherit_handle: bool,
    ) -> Result<Option<Thread>, Error> {
        let snapshot = self.snapshot(SnapshotFlags::TH32CS_SNAPTHREAD)?;
        let pid = self.pid()?;

        for thread_entry in snapshot.thread_entries() {
            if pid == thread_entry.th32OwnerProcessID {
                return Ok(Some(unsafe {
                    Thread::from_id(thread_entry.th32ThreadID, access, inherit_handle)?
                }));
            }
        }

        Ok(None)
    }

    pub fn handle(&self) -> Result<HANDLE, Error> {
        if self.handle.is_null() {
            Err(Error::new("Attempted to retrieve NULL handle".to_string()))
        } else {
            Ok(self.handle)
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.close().unwrap();
    }
}
