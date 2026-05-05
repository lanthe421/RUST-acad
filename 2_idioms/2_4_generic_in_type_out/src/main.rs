use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

fn main() {
    // Error::new now accepts &str, String, or anything Into<String>
    let mut err = Error::new("NO_USER");
    err.status(404).message("User not found");

    // Server::bind accepts anything that implements ToSocketAddrs
    let mut server = Server::default();
    server.bind("127.0.0.1:8080");
}

#[derive(Debug)]
pub struct Error {
    code: String,
    status: u16,
    message: String,
}

impl Default for Error {
    #[inline]
    fn default() -> Self {
        Self {
            code: "UNKNOWN".to_string(),
            status: 500,
            message: "Unknown error has happened.".to_string(),
        }
    }
}

impl Error {
    // Accept any type convertible to String: &str, String, Cow<str>, etc.
    pub fn new(code: impl Into<String>) -> Self {
        let mut err = Self::default();
        err.code = code.into();
        err
    }

    pub fn status(&mut self, s: u16) -> &mut Self {
        self.status = s;
        self
    }

    // Accept any type convertible to String instead of requiring String
    pub fn message(&mut self, m: impl Into<String>) -> &mut Self {
        self.message = m.into();
        self
    }
}

#[derive(Debug, Default)]
pub struct Server(Option<SocketAddr>);

impl Server {
    // Accept anything that can be resolved to a SocketAddr: "ip:port", (IpAddr, u16), etc.
    pub fn bind(&mut self, addr: impl ToSocketAddrs) {
        self.0 = addr.to_socket_addrs().ok().and_then(|mut a| a.next());
    }
}

#[cfg(test)]
mod server_spec {
    use super::*;

    mod bind {
        use std::net::Ipv4Addr;

        use super::*;

        #[test]
        fn sets_provided_address_to_server() {
            let mut server = Server::default();

            // (IpAddr, u16) tuple — ToSocketAddrs
            server.bind((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
            assert_eq!(format!("{}", server.0.unwrap()), "127.0.0.1:8080");

            // string slice — ToSocketAddrs
            server.bind("[::1]:9911");
            assert_eq!(format!("{}", server.0.unwrap()), "[::1]:9911");
        }
    }
}
