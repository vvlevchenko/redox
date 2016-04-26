use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use arch::context::ContextManager;
use arch::intex::Intex;
use common::event::Event;
use common::time::Duration;
use disk::Disk;
use fs::{KScheme, Resource, Scheme, VecResource, Url};
use logging::LogLevel;
use sync::WaitQueue;

use system::error::{Error, Result, ENOENT, EEXIST};
use system::syscall::{O_CREAT, Stat};

use self::console::Console;

/// The Kernel Console
pub mod console;

/// The kernel environment
pub struct Environment {
    /// Contexts
    pub contexts: Intex<ContextManager>,

    /// Clock realtime (default)
    pub clock_realtime: Intex<Duration>,
    /// Monotonic clock
    pub clock_monotonic: Intex<Duration>,

    /// Default console
    pub console: Intex<Console>,
    /// Disks
    pub disks: Intex<Vec<Box<Disk>>>,
    /// Pending events
    pub events: WaitQueue<Event>,
    /// Kernel logs
    pub logs: Intex<Vec<(LogLevel, String)>>,
    /// Schemes
    pub schemes: Intex<Vec<Box<KScheme>>>,

    /// Interrupt stats
    pub interrupts: Intex<[u64; 256]>,
}

impl Environment {
    pub fn new() -> Box<Environment> {
        box Environment {
            contexts: Intex::new(ContextManager::new()),

            clock_realtime: Intex::new(Duration::new(0, 0)),
            clock_monotonic: Intex::new(Duration::new(0, 0)),

            console: Intex::new(Console::new()),
            disks: Intex::new(Vec::new()),
            events: WaitQueue::new(),
            logs: Intex::new(Vec::new()),
            schemes: Intex::new(Vec::new()),

            interrupts: Intex::new([0; 256]),
        }
    }

    pub fn on_irq(&self, irq: u8) {
        for mut scheme in self.schemes.lock().iter_mut() {
            scheme.on_irq(irq);
        }
    }

    /// Open a new resource
    pub fn open(&self, url: Url, flags: usize) -> Result<Box<Resource>> {
        let url_scheme = url.scheme();
        if url_scheme.is_empty() {
            let url_path = url.reference();
            if url_path.trim_matches('/').is_empty() {
                let mut list = String::new();

                for scheme in self.schemes.lock().iter() {
                    let scheme_str = scheme.scheme();
                    if !scheme_str.is_empty() {
                        if !list.is_empty() {
                            list = list + "\n" + scheme_str;
                        } else {
                            list = scheme_str.to_string();
                        }
                    }
                }

                Ok(box VecResource::new(":".to_string(), list.into_bytes()))
            } else if flags & O_CREAT == O_CREAT {
                for scheme in self.schemes.lock().iter_mut() {
                    if scheme.scheme() == url_path {
                        return Err(Error::new(EEXIST));
                    }
                }

                match Scheme::new(url_path) {
                    Ok((scheme, server)) => {
                        self.schemes.lock().push(scheme);
                        Ok(server)
                    },
                    Err(err) => Err(err)
                }
            } else {
                Err(Error::new(ENOENT))
            }
        } else {
            for mut scheme in self.schemes.lock().iter_mut() {
                if scheme.scheme() == url_scheme {
                    return scheme.open(url, flags);
                }
            }
            Err(Error::new(ENOENT))
        }
    }

    /// Makes a directory
    pub fn mkdir(&self, url: Url, flags: usize) -> Result<()> {
        let url_scheme = url.scheme();
        if !url_scheme.is_empty() {
            for mut scheme in self.schemes.lock().iter_mut() {
                if scheme.scheme() == url_scheme {
                    return scheme.mkdir(url, flags);
                }
            }
        }
        Err(Error::new(ENOENT))
    }

    /// Remove a directory
    pub fn rmdir(&self, url: Url) -> Result<()> {
        let url_scheme = url.scheme();
        if !url_scheme.is_empty() {
            for mut scheme in self.schemes.lock().iter_mut() {
                if scheme.scheme() == url_scheme {
                    return scheme.rmdir(url);
                }
            }
        }
        Err(Error::new(ENOENT))
    }

    /// Stat a path
    pub fn stat(&self, url: Url, stat: &mut Stat) -> Result<()> {
        let url_scheme = url.scheme();
        if !url_scheme.is_empty() {
            for mut scheme in self.schemes.lock().iter_mut() {
                if scheme.scheme() == url_scheme {
                    return scheme.stat(url, stat);
                }
            }
        }
        Err(Error::new(ENOENT))
    }

    /// Unlink a resource
    pub fn unlink(&self, url: Url) -> Result<()> {
        let url_scheme = url.scheme();
        if !url_scheme.is_empty() {
            for mut scheme in self.schemes.lock().iter_mut() {
                if scheme.scheme() == url_scheme {
                    return scheme.unlink(url);
                }
            }
        }
        Err(Error::new(ENOENT))
    }
}
