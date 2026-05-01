use crate::error::IpError;
use crate::resolver::IpResolver;
use crate::types::IpCore;

pub struct IpRegistry {
    resolver: IpResolver,
}

impl IpRegistry {
    pub fn new() -> Self {
        Self {
            resolver: IpResolver::new(),
        }
    }

    pub fn register(&mut self, ip: IpCore) -> Result<(), IpError> {
        self.resolver.register(ip)
    }

    pub fn resolve(&self, ip_id: &str) -> Result<Vec<IpCore>, IpError> {
        self.resolver.resolve(ip_id)
    }

    pub fn find(&self, ip_id: &str) -> Option<&IpCore> {
        self.resolver.find(ip_id)
    }
}

impl Default for IpRegistry {
    fn default() -> Self {
        Self::new()
    }
}