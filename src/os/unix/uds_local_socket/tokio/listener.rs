use {
    super::Stream,
    crate::{
        local_socket::{
            prelude::*, traits::tokio as traits, ListenerNonblockingMode, ListenerOptions,
        },
        os::unix::uds_local_socket::{listener::Listener as SyncListener, ReclaimGuard},
        Sealed,
    },
    std::{
        fmt::{self, Debug, Formatter},
        io,
        os::unix::prelude::*,
    },
    tokio::net::UnixListener,
};

/// Wrapper around [`UnixListener`] that implements [`Listener`](traits::Listener).
pub struct Listener {
    listener: UnixListener,
    reclaim: ReclaimGuard,
}
impl Sealed for Listener {}
impl traits::Listener for Listener {
    type Stream = Stream;

    fn from_options(options: ListenerOptions<'_>) -> io::Result<Self> {
        options
            .nonblocking(ListenerNonblockingMode::Both)
            .create_sync_as::<SyncListener>()
            .and_then(|mut sync| {
                let reclaim = sync.reclaim.take();
                Ok(Self { listener: UnixListener::from_std(sync.into())?, reclaim })
            })
    }
    async fn accept(&self) -> io::Result<Stream> {
        let inner = self.listener.accept().await?.0;
        Ok(Stream::from(inner))
    }

    fn do_not_reclaim_name_on_drop(&mut self) { self.reclaim.forget(); }
}
/// Access to the underlying implementation.
impl Listener {
    /// Borrows the [`UnixListener`] contained within, granting access to operations defined on it.
    #[inline(always)]
    pub fn inner(&self) -> &UnixListener { &self.listener }
    /// Mutably borrows the [`UnixListener`] contained within, granting access to operations
    /// defined on it.
    #[inline(always)]
    pub fn inner_mut(&mut self) -> &mut UnixListener { &mut self.listener }
}

/// Does not assume that the sync `Listener` is in nonblocking mode, setting it to
/// `ListenerNonblockingMode::Both` automatically.
// FUTURE remove handholding and assume nonblocking
impl TryFrom<SyncListener> for Listener {
    type Error = io::Error;
    fn try_from(mut sync: SyncListener) -> io::Result<Self> {
        sync.set_nonblocking(ListenerNonblockingMode::Both)?;
        let reclaim = sync.reclaim.take();
        Ok(Self { listener: UnixListener::from_std(sync.into())?, reclaim })
    }
}

/// Construction from existing file descriptors.
impl Listener {
    /// Creates a listener from an already-listening file descriptor.
    ///
    /// No binding or `listen()` call is performed. If [name reclamation] is enabled in `opts`,
    /// the actual socket path is obtained via `getsockname` and used for cleanup on drop.
    ///
    /// [name reclamation]: crate::local_socket::Listener#name-reclamation
    pub fn from_fd_with_options(fd: OwnedFd, opts: ListenerOptions<'_>) -> io::Result<Self> {
        SyncListener::from_fd_with_options(fd, opts).and_then(Self::try_from)
    }
}

impl Debug for Listener {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Listener")
            .field("fd", &self.listener.as_raw_fd())
            .field("reclaim", &self.reclaim)
            .finish()
    }
}
impl AsFd for Listener {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> { self.listener.as_fd() }
}
impl TryFrom<Listener> for OwnedFd {
    type Error = io::Error;
    fn try_from(mut slf: Listener) -> io::Result<Self> {
        slf.listener.into_std().map(|s| {
            slf.reclaim.forget();
            s.into()
        })
    }
}
/// Does not assume that the listener is in nonblocking mode, setting it to
/// `ListenerNonblockingMode::Both` automatically.
impl TryFrom<OwnedFd> for Listener {
    type Error = io::Error;
    fn try_from(fd: OwnedFd) -> io::Result<Self> { Self::try_from(SyncListener::from(fd)) }
}
