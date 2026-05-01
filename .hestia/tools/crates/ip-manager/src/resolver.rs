use std::collections::{HashMap, HashSet, VecDeque};

use crate::error::IpError;
use crate::types::IpCore;

pub struct IpResolver {
    ips: HashMap<String, IpCore>,
    dependencies: HashMap<String, Vec<String>>,
}

impl IpResolver {
    pub fn new() -> Self {
        Self {
            ips: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn register(&mut self, ip: IpCore) -> Result<(), IpError> {
        if self.ips.contains_key(&ip.id) {
            return Err(IpError::DuplicateIp(ip.id.clone()));
        }
        let ip_id = ip.id.clone();
        let deps: Vec<String> = ip.dependencies.iter().map(|d| d.ip_id.clone()).collect();
        self.ips.insert(ip_id.clone(), ip);
        self.dependencies.insert(ip_id, deps);
        Ok(())
    }

    pub fn resolve(&self, ip_id: &str) -> Result<Vec<IpCore>, IpError> {
        let _root = self
            .ips
            .get(ip_id)
            .ok_or_else(|| IpError::NotFound(ip_id.to_string()))?;

        let mut visited = HashSet::new();
        let mut order = Vec::new();
        let mut stack = vec![ip_id.to_string()];

        while let Some(current_id) = stack.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());
            order.push(current_id.clone());

            if let Some(deps) = self.dependencies.get(&current_id) {
                for dep_id in deps {
                    if !visited.contains(dep_id) {
                        stack.push(dep_id.clone());
                    }
                    if visited.contains(dep_id) && self.has_path(dep_id, &current_id) {
                        return Err(IpError::CircularDependency(format!(
                            "{} -> {}",
                            dep_id, current_id
                        )));
                    }
                }
            }
        }

        order.reverse();
        Ok(order
            .iter()
            .filter_map(|id| self.ips.get(id))
            .cloned()
            .collect())
    }

    fn has_path(&self, from: &str, to: &str) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![from.to_string()];
        while let Some(current) = stack.pop() {
            if current == to {
                return true;
            }
            if visited.insert(current.clone()) {
                if let Some(deps) = self.dependencies.get(&current) {
                    for dep in deps {
                        stack.push(dep.clone());
                    }
                }
            }
        }
        false
    }

    pub fn check_cycles(&self) -> Result<(), IpError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for ip_id in self.ips.keys() {
            if self.has_cycle(ip_id, &mut visited, &mut rec_stack)? {
                return Err(IpError::CircularDependency(ip_id.clone()));
            }
        }
        Ok(())
    }

    fn has_cycle(
        &self,
        ip_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<bool, IpError> {
        if rec_stack.contains(ip_id) {
            return Ok(true);
        }
        if visited.contains(ip_id) {
            return Ok(false);
        }

        visited.insert(ip_id.to_string());
        rec_stack.insert(ip_id.to_string());

        if let Some(deps) = self.dependencies.get(ip_id) {
            for dep_id in deps {
                if self.has_cycle(dep_id, visited, rec_stack)? {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(ip_id);
        Ok(false)
    }

    pub fn find(&self, ip_id: &str) -> Option<&IpCore> {
        self.ips.get(ip_id)
    }
}

impl Default for IpResolver {
    fn default() -> Self {
        Self::new()
    }
}