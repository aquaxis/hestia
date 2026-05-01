use crate::types::IpCore;

pub fn satisfies(ip: &IpCore, req: &semver::VersionReq) -> bool {
    req.matches(&ip.version)
}

pub fn find_best_match<'a>(ips: &'a [IpCore], req: &semver::VersionReq) -> Option<&'a IpCore> {
    ips.iter()
        .filter(|ip| req.matches(&ip.version))
        .max_by_key(|ip| &ip.version)
}