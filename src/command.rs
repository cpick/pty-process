use async_process::unix::CommandExt as _;

/// Wrapper around [`async_process::Command`]
pub struct Command {
    inner: async_process::Command,
    stdin: bool,
    stdout: bool,
    stderr: bool,
    pre_exec_set: bool,
    pre_exec: Option<
        Box<dyn FnMut() -> std::io::Result<()> + Send + Sync + 'static>,
    >,
}

impl Command {
    /// See [`async_process::Command::new`]
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Self {
        Self {
            inner: async_process::Command::new(program),
            stdin: false,
            stdout: false,
            stderr: false,
            pre_exec_set: false,
            pre_exec: None,
        }
    }

    /// See [`async_process::Command::arg`]
    pub fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    /// See [`async_process::Command::args`]
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.args(args);
        self
    }

    /// See [`async_process::Command::env`]
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.env(key, val);
        self
    }

    /// See [`async_process::Command::envs`]
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.envs(vars);
        self
    }

    /// See [`async_process::Command::env_remove`]
    pub fn env_remove<K: AsRef<std::ffi::OsStr>>(
        &mut self,
        key: K,
    ) -> &mut Self {
        self.inner.env_remove(key);
        self
    }

    /// See [`async_process::Command::env_clear`]
    pub fn env_clear(&mut self) -> &mut Self {
        self.inner.env_clear();
        self
    }

    /// See [`async_process::Command::current_dir`]
    pub fn current_dir<P: AsRef<std::path::Path>>(
        &mut self,
        dir: P,
    ) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    /// See [`async_process::Command::stdin`]
    pub fn stdin<T: Into<std::process::Stdio>>(
        &mut self,
        cfg: T,
    ) -> &mut Self {
        self.stdin = true;
        self.inner.stdin(cfg);
        self
    }

    /// See [`async_process::Command::stdout`]
    pub fn stdout<T: Into<std::process::Stdio>>(
        &mut self,
        cfg: T,
    ) -> &mut Self {
        self.stdout = true;
        self.inner.stdout(cfg);
        self
    }

    /// See [`async_process::Command::stderr`]
    pub fn stderr<T: Into<std::process::Stdio>>(
        &mut self,
        cfg: T,
    ) -> &mut Self {
        self.stderr = true;
        self.inner.stderr(cfg);
        self
    }

    /// Executes the command as a child process via
    /// [`async_process::Command::spawn`], and attaches the given `pty` to
    /// that child. The pty will be attached to all of `stdin`, `stdout`, and
    /// `stderr` of the child, unless those file descriptors were previously
    /// overridden through calls to [`stdin`](Self::stdin),
    /// [`stdout`](Self::stdout), or [`stderr`](Self::stderr). The newly
    /// created child process will also be made the session leader of a new
    /// session, and will have the given `pty` instance set as its controlling
    /// terminal.
    ///
    /// # Errors
    /// Returns an error if we fail to allocate new file descriptors for
    /// attaching the pty to the child process, or if we fail to spawn the
    /// child process (see the documentation for
    /// [`async_process::Command::spawn`]), or if we fail to make the child a
    /// session leader or set its controlling terminal.
    pub fn spawn(
        &mut self,
        pty: &crate::Pty,
    ) -> crate::Result<async_process::Child> {
        let pts = pty.pts();
        let (stdin, stdout, stderr) = crate::sys::setup_subprocess(pts)?;

        if !self.stdin {
            self.inner.stdin(stdin);
        }
        if !self.stdout {
            self.inner.stdout(stdout);
        }
        if !self.stderr {
            self.inner.stderr(stderr);
        }

        let mut session_leader = crate::sys::session_leader(pts);
        // Safety: setsid() is an async-signal-safe function and ioctl() is a
        // raw syscall (which is inherently async-signal-safe).
        if let Some(mut custom) = self.pre_exec.take() {
            unsafe {
                self.inner.pre_exec(move || {
                    session_leader()?;
                    custom()?;
                    Ok(())
                })
            };
        } else if !self.pre_exec_set {
            unsafe { self.inner.pre_exec(session_leader) };
        }
        self.pre_exec_set = true;

        Ok(self.inner.spawn()?)
    }

    /// See [`async_process::unix::CommandExt::uid`]
    pub fn uid(&mut self, id: u32) -> &mut Self {
        self.inner.uid(id);
        self
    }

    /// See [`async_process::unix::CommandExt::gid`]
    pub fn gid(&mut self, id: u32) -> &mut Self {
        self.inner.gid(id);
        self
    }

    /// See [`async_process::unix::CommandExt::pre_exec`]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn pre_exec<F>(&mut self, f: F) -> &mut Self
    where
        F: FnMut() -> std::io::Result<()> + Send + Sync + 'static,
    {
        self.pre_exec = Some(Box::new(f));
        self
    }

    /// See [`async_process::unix::CommandExt::arg0`]
    pub fn arg0<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.arg0(arg);
        self
    }
}
