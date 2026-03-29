#[cfg(unix)]
use crate::os::unix::uds_local_socket::tokio as uds_impl;
#[cfg(windows)]
use crate::os::windows::named_pipe::local_socket::tokio as np_impl;
use {
    super::r#trait,
    crate::local_socket::{tokio::Stream, Listener as SyncListener, ListenerOptions},
    std::io,
};

impmod! {local_socket::dispatch_tokio as dispatch}

mkenum!(
/// Tokio-based local socket server, listening for connections.
///
/// This struct is created by [`ListenerOptions`](crate::local_socket::ListenerOptions).
///
/// See the [module-level documentation of local sockets](crate::local_socket) for more details.
///
/// [Name reclamation](crate::local_socket::Listener#name-reclamation) is performed by default
/// when using local socket implementations that necessitate it.
///
/// # Examples
///
/// ## Basic server
/// ```no_run
#[cfg_attr(doc, doc = doctest_file::include_doctest!("examples/local_socket/tokio/listener.rs"))]
/// ```
Listener);
impl r#trait::Listener for Listener {
    type Stream = Stream;

    #[inline]
    fn from_options(options: ListenerOptions<'_>) -> io::Result<Self> {
        dispatch::listen(options)
    }
    #[inline]
    async fn accept(&self) -> io::Result<Stream> {
        dispatch!(Self: x in self => x.accept()).await.map(Stream::from)
    }
    #[inline]
    fn do_not_reclaim_name_on_drop(&mut self) {
        dispatch!(Self: x in self => x.do_not_reclaim_name_on_drop())
    }
}

impl TryFrom<SyncListener> for Listener {
    type Error = io::Error;
    fn try_from(sync: SyncListener) -> io::Result<Self> {
        Ok(match sync {
            #[cfg(unix)]
            SyncListener::UdSocket(inner) => Self::UdSocket(uds_impl::Listener::try_from(inner)?),
            #[cfg(windows)]
            SyncListener::NamedPipe(inner) => {
                Self::NamedPipe(np_impl::Listener::try_from(inner)?)
            }
        })
    }
}
