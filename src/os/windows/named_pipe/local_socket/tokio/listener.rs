use {
    super::Stream,
    crate::{
        local_socket::{traits::tokio as traits, ListenerOptions, NameInner},
        os::windows::named_pipe::{
            local_socket::listener::Listener as SyncListener, pipe_mode,
            tokio::PipeListener as GenericPipeListener, PipeListenerOptions,
        },
        Sealed,
    },
    std::{borrow::Cow, io, os::windows::io::OwnedHandle},
};

type PipeListener = GenericPipeListener<pipe_mode::Bytes, pipe_mode::Bytes>;

/// Wrapper around [`PipeListener`](GenericPipeListener) that implements the
/// [`Listener`](traits::Listener) trait.
#[derive(Debug)]
pub struct Listener(PipeListener);
impl Sealed for Listener {}
impl traits::Listener for Listener {
    type Stream = Stream;

    fn from_options(options: ListenerOptions<'_>) -> io::Result<Self> {
        let mut impl_options = PipeListenerOptions::new();
        let NameInner::NamedPipe(path) = options.name.0;
        impl_options.path = path;
        impl_options.security_descriptor = options.security_descriptor;
        impl_options.create_tokio().map(Self)
    }
    async fn accept(&self) -> io::Result<Stream> {
        let inner = self.0.accept().await?;
        Ok(Stream(inner))
    }
    fn do_not_reclaim_name_on_drop(&mut self) {}
}

/// Access to the underlying implementation.
impl Listener {
    /// Borrows the [`PipeListener`](GenericPipeListener) contained within, granting access to
    /// operations defined on it.
    #[inline(always)]
    pub fn inner(&self) -> &PipeListener { &self.0 }
    /// Mutably borrows the [`PipeListener`](GenericPipeListener) contained within, granting
    /// access to operations defined on it.
    #[inline(always)]
    pub fn inner_mut(&mut self) -> &mut PipeListener { &mut self.0 }
}

impl From<PipeListener> for Listener {
    #[inline(always)]
    fn from(l: PipeListener) -> Self { Self(l) }
}
impl From<Listener> for PipeListener {
    #[inline(always)]
    fn from(l: Listener) -> Self { l.0 }
}

/// Construction from existing handles.
impl Listener {
    /// Creates a listener from an existing named pipe server handle, using the given options.
    ///
    /// The handle must already be a listening named pipe server instance. The pipe path in `opts`
    /// and other options are used to create new instances on each [`accept()`](traits::Listener::accept) call.
    pub fn from_handle_with_options(handle: OwnedHandle, opts: ListenerOptions<'_>) -> io::Result<Self> {
        let NameInner::NamedPipe(path) = opts.name.0;
        let impl_options = PipeListenerOptions {
            path: Cow::Owned(path.into_owned()),
            security_descriptor: opts.security_descriptor,
            ..PipeListenerOptions::new()
        };
        GenericPipeListener::from_handle_and_options(handle, impl_options).map(Self)
    }
}

impl TryFrom<SyncListener> for Listener {
    type Error = io::Error;
    fn try_from(sync: SyncListener) -> io::Result<Self> {
        GenericPipeListener::from_handle_and_options(
            sync.listener.stored_instance.into_inner().map_err(crate::poison_error)?,
            sync.listener.config,
        )
        .map(Self)
    }
}

multimacro! {
    Listener,
    forward_as_ref(PipeListener),
    forward_as_mut(PipeListener),
}
