use libunftp::auth::UserDetail;
use std::{fmt::Formatter, path::PathBuf};
use unftp_sbe_fs::{Filesystem, Meta};
use unftp_sbe_rooter::{RooterVfs, UserWithRoot};

#[derive(Debug, PartialEq, Eq)]
pub struct User {
    pub username: String,
    pub root: Option<PathBuf>,
}

impl UserDetail for User {
    fn account_enabled(&self) -> bool {
        true
    }
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "User(username: {:?}", self.username,)
    }
}

impl UserWithRoot for User {
    fn user_root(&self) -> Option<PathBuf> {
        self.root.clone()
    }
}

// Return type omited for brevity.
pub fn create_rooted_storage_backend() -> Box<fn() -> RooterVfs<Filesystem, User, Meta>> {
    Box::new(move || RooterVfs::<Filesystem, User, Meta>::new(Filesystem::new("/storage/ftp")))
}
