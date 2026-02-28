use crate::cli::{HandoffCfg, Mode};
use anyhow::{Context, Result, bail};
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, UnixStream};

pub struct Lane {
    pub r: Pin<Box<dyn AsyncRead + Send>>,
    pub w: Pin<Box<dyn AsyncWrite + Send>>,
}

pub struct Lanes {
    pub diag: Lane,
    pub event: Lane,
    pub ctrl: Lane,
}

impl Lanes {
    #[cfg(unix)]
    fn from_uds(stream: UnixStream) -> Lane {
        let (r, w) = stream.into_split();
        Lane {
            r: Box::pin(r) as Pin<Box<dyn AsyncRead + Send>>,
            w: Box::pin(w) as Pin<Box<dyn AsyncWrite + Send>>,
        }
    }

    fn from_tcp(stream: TcpStream) -> Lane {
        let (r, w) = stream.into_split();
        Lane {
            r: Box::pin(r) as Pin<Box<dyn AsyncRead + Send>>,
            w: Box::pin(w) as Pin<Box<dyn AsyncWrite + Send>>,
        }
    }
}

pub async fn open_lanes(cfg: &HandoffCfg) -> Result<Lanes> {
    match cfg.mode {
        Mode::Uds => {
            #[cfg(unix)]
            {
                let diag = lane_from_fd(cfg.fd_diag.context("fd_diag required")?)?;
                let event = lane_from_fd(cfg.fd_event.context("fd_event required")?)?;
                let ctrl = lane_from_fd(cfg.fd_ctrl.context("fd_ctrl required")?)?;
                Ok(Lanes { diag, event, ctrl })
            }
            #[cfg(not(unix))]
            {
                bail!("uds mode is only supported on unix");
            }
        }
        Mode::Tcp => {
            let host = cfg.host.as_deref().context("host required")?;
            let mk = |p: u16| async move {
                TcpStream::connect((host, p))
                    .await
                    .with_context(|| format!("tcp connect to {}:{}", host, p))
            };

            let d = mk(cfg.port_diag.context("port_diag required")?).await?;
            let e = mk(cfg.port_event.context("port_event required")?).await?;
            let c = mk(cfg.port_ctrl.context("port_ctrl required")?).await?;

            Ok(Lanes {
                diag: Lanes::from_tcp(d),
                event: Lanes::from_tcp(e),
                ctrl: Lanes::from_tcp(c),
            })
        }
    }
}

#[cfg(unix)]
fn lane_from_fd(fd: i32) -> Result<Lane> {
    use std::os::fd::{FromRawFd, OwnedFd};
    // Take ownership of the fd as a std UnixStream
    // Safety: we immediately wrap the fd and set nonblocking before handing to Tokio.
    let owned = unsafe { OwnedFd::from_raw_fd(fd) };
    let std_uds = std::os::unix::net::UnixStream::from(owned);
    std_uds
        .set_nonblocking(true)
        .context("setting UDS nonblocking")?;
    let uds = UnixStream::from_std(std_uds).context("tokio UnixStream from std")?;
    Ok(Lanes::from_uds(uds))
}
