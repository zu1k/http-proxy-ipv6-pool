use hyper::{
    client::HttpConnector,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Client, Method, Request, Response, Server,
};
use rand::Rng;
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr, ToSocketAddrs},
    sync::{atomic::AtomicU64, Arc},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpSocket,
};

pub async fn start_proxy(listen_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let id = Arc::new(AtomicU64::new(0));

    let make_service = make_service_fn(move |_: &AddrStream| {
        let id = Arc::clone(&id);
        async move { Ok::<_, hyper::Error>(service_fn(move |req| Proxy { id: id.clone() }.proxy(req))) }
    });

    Server::bind(&listen_addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(make_service)
        .await
        .map_err(|err| err.into())
}

#[derive(Clone)]
pub(crate) struct Proxy {
    pub id: Arc<AtomicU64>,
}

impl Proxy {
    pub(crate) async fn proxy(self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let id = self.id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        match if req.method() == Method::CONNECT {
            self.process_connect(req, id).await
        } else {
            self.process_request(req, id).await
        } {
            Ok(resp) => Ok(resp),
            Err(e) => Err(e),
        }
    }

    async fn process_connect(
        self,
        req: Request<Body>,
        id: u64,
    ) -> Result<Response<Body>, hyper::Error> {
        tokio::task::spawn(async move {
            let remote_addr = req.uri().authority().map(|auth| auth.to_string()).unwrap();
            let mut upgraded = hyper::upgrade::on(req).await.unwrap();
            tunnel(&mut upgraded, remote_addr, id).await
        });
        Ok(Response::new(Body::empty()))
    }

    async fn process_request(
        self,
        req: Request<Body>,
        _id: u64,
    ) -> Result<Response<Body>, hyper::Error> {
        let bind_addr = get_rand_ipv6();
        let mut http = HttpConnector::new();
        http.set_local_address(Some(bind_addr));

        let client = Client::builder()
            .http1_title_case_headers(true)
            .http1_preserve_header_case(true)
            .build(http);
        let res = client.request(req).await?;
        Ok(res)
    }
}

async fn tunnel<A>(upgraded: &mut A, addr_str: String, _id: u64) -> std::io::Result<()>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    if let Ok(addrs) = addr_str.to_socket_addrs() {
        for addr in addrs {
            let socket = TcpSocket::new_v6()?;

            let bind_addr = get_rand_ipv6_socket_addr();

            println!("{addr_str} via {bind_addr}");

            socket.bind(bind_addr).unwrap();
            if let Ok(mut server) = socket.connect(addr).await {
                tokio::io::copy_bidirectional(upgraded, &mut server).await?;
                return Ok(());
            }
        }
    } else {
        println!("error: {addr_str}");
    }

    Ok(())
}

fn get_rand_ipv6_socket_addr() -> SocketAddr {
    let mut rng = rand::thread_rng();
    SocketAddr::new(get_rand_ipv6(), rng.gen::<u16>())
}

fn get_rand_ipv6() -> IpAddr {
    let mut rng = rand::thread_rng();
    let ipv6 = Ipv6Addr::new(
        0x2001,
        0x19f0,
        0x6001,
        0x48e4,
        rng.gen::<u16>(),
        rng.gen::<u16>(),
        rng.gen::<u16>(),
        rng.gen::<u16>(),
    );
    IpAddr::V6(ipv6)
}
