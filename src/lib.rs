//! SigmaOS-acct — comprehensive accounting component.
//!
//! Provides process accounting functionality for tracking system usage,
//! including command execution, resource consumption, and user activity.

use std::collections::HashMap;

/// Process accounting record
#[derive(Debug, Clone)]
pub struct ProcessRecord {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub comm: String,
    /// User ID
    pub uid: u32,
    /// Group ID
    pub gid: u32,
    /// Start time
    pub start_time: u64,
    /// End time
    pub end_time: u64,
    /// CPU time (user + system)
    pub cpu_time: u64,
    /// Memory usage
    pub memory_usage: u64,
    /// Disk I/O
    pub disk_io: u64,
    /// Network I/O
    pub network_io: u64,
    /// Exit status
    pub exit_status: i32,
    /// Command arguments
    pub args: Vec<String>,
}

/// Accounting statistics
#[derive(Debug, Clone)]
pub struct AccountingStatistics {
    /// Total processes
    pub total_processes: u64,
    /// Total CPU time
    pub total_cpu_time: u64,
    /// Total memory usage
    pub total_memory_usage: u64,
    /// Total disk I/O
    pub total_disk_io: u64,
    /// Total network I/O
    pub total_network_io: u64,
    /// Average CPU time per process
    pub avg_cpu_time: f64,
    /// Average memory usage per process
    pub avg_memory_usage: f64,
}

/// Accounting manager
#[derive(Debug, Clone)]
pub struct AccountingManager {
    /// Process records
    records: Vec<ProcessRecord>,
    /// Process index by PID
    process_index: HashMap<u32, usize>,
    /// Statistics
    statistics: AccountingStatistics,
    /// Max records to keep
    max_records: usize,
}

impl Default for AccountingManager {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            process_index: HashMap::new(),
            statistics: AccountingStatistics {
                total_processes: 0,
                total_cpu_time: 0,
                total_memory_usage: 0,
                total_disk_io: 0,
                total_network_io: 0,
                avg_cpu_time: 0.0,
                avg_memory_usage: 0.0,
            },
            max_records: 10000,
        }
    }
}

impl AccountingManager {
    /// Create new accounting manager
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Start process accounting
    pub fn start_process(&mut self, pid: u32, comm: String, uid: u32, gid: u32, args: Vec<String>) -> bool {
        // Check if process already exists
        if self.process_index.contains_key(&pid) {
            return false;
        }
        
        let record = ProcessRecord {
            pid,
            comm,
            uid,
            gid,
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            end_time: 0,
            cpu_time: 0,
            memory_usage: 0,
            disk_io: 0,
            network_io: 0,
            exit_status: 0,
            args,
        };
        
        let index = self.records.len();
        self.records.push(record);
        self.process_index.insert(pid, index);
        self.statistics.total_processes += 1;
        
        true
    }
    
    /// Update process statistics
    pub fn update_process(&mut self, pid: u32, cpu_time: u64, memory_usage: u64, disk_io: u64, network_io: u64) -> bool {
        if let Some(&index) = self.process_index.get(&pid) {
            if let Some(record) = self.records.get_mut(index) {
                record.cpu_time = cpu_time;
                record.memory_usage = memory_usage;
                record.disk_io = disk_io;
                record.network_io = network_io;
                return true;
            }
        }
        false
    }
    
    /// End process accounting
    pub fn end_process(&mut self, pid: u32, exit_status: i32) -> bool {
        if let Some(&index) = self.process_index.get(&pid) {
            if let Some(record) = self.records.get_mut(index) {
                record.end_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                record.exit_status = exit_status;
                
                // Update statistics
                self.statistics.total_cpu_time += record.cpu_time;
                self.statistics.total_memory_usage += record.memory_usage;
                self.statistics.total_disk_io += record.disk_io;
                self.statistics.total_network_io += record.network_io;
                
                // Calculate averages
                if self.statistics.total_processes > 0 {
                    self.statistics.avg_cpu_time = self.statistics.total_cpu_time as f64 / self.statistics.total_processes as f64;
                    self.statistics.avg_memory_usage = self.statistics.total_memory_usage as f64 / self.statistics.total_processes as f64;
                }
                
                return true;
            }
        }
        false
    }
    
    /// Get process record
    pub fn get_process(&self, pid: u32) -> Option<&ProcessRecord> {
        self.process_index.get(&pid).and_then(|&index| self.records.get(index))
    }
    
    /// Get all process records
    pub fn get_all_processes(&self) -> Vec<&ProcessRecord> {
        self.records.iter().collect()
    }
    
    /// Get processes by user
    pub fn get_processes_by_user(&self, uid: u32) -> Vec<&ProcessRecord> {
        self.records.iter()
            .filter(|r| r.uid == uid)
            .collect()
    }
    
    /// Get processes by group
    pub fn get_processes_by_group(&self, gid: u32) -> Vec<&ProcessRecord> {
        self.records.iter()
            .filter(|r| r.gid == gid)
            .collect()
    }
    
    /// Get top processes by CPU time
    pub fn get_top_processes_by_cpu(&self, limit: usize) -> Vec<&ProcessRecord> {
        let mut processes: Vec<&ProcessRecord> = self.records.iter().collect();
        processes.sort_by(|a, b| b.cpu_time.cmp(&a.cpu_time));
        processes.truncate(limit);
        processes
    }
    
    /// Get top processes by memory usage
    pub fn get_top_processes_by_memory(&self, limit: usize) -> Vec<&ProcessRecord> {
        let mut processes: Vec<&ProcessRecord> = self.records.iter().collect();
        processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
        processes.truncate(limit);
        processes
    }
    
    /// Get processes by command name
    pub fn get_processes_by_command(&self, comm: &str) -> Vec<&ProcessRecord> {
        self.records.iter()
            .filter(|r| r.comm == comm)
            .collect()
    }
    
    /// Get process duration
    pub fn get_process_duration(&self, pid: u32) -> Option<u64> {
        self.get_process(pid).map(|r| {
            if r.end_time > 0 {
                r.end_time - r.start_time
            } else {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - r.start_time
            }
        })
    }
    
    /// Get accounting statistics
    pub fn get_statistics(&self) -> &AccountingStatistics {
        &self.statistics
    }
    
    /// Clean old records
    pub fn clean_old_records(&mut self, max_age_secs: u64) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let initial_len = self.records.len();
        self.records.retain(|r| {
            now - r.start_time <= max_age_secs
        });
        
        // Rebuild index
        self.process_index.clear();
        for (i, record) in self.records.iter().enumerate() {
            self.process_index.insert(record.pid, i);
        }
        
        initial_len - self.records.len()
    }
    
    /// Export accounting data
    pub fn export_data(&self) -> Vec<ProcessRecord> {
        self.records.clone()
    }
    
    /// Import accounting data
    pub fn import_data(&mut self, records: Vec<ProcessRecord>) {
        self.records = records;
        self.process_index.clear();
        
        for (i, record) in self.records.iter().enumerate() {
            self.process_index.insert(record.pid, i);
        }
        
        // Recalculate statistics
        self.statistics.total_processes = self.records.len() as u64;
        self.statistics.total_cpu_time = self.records.iter().map(|r| r.cpu_time).sum();
        self.statistics.total_memory_usage = self.records.iter().map(|r| r.memory_usage).sum();
        self.statistics.total_disk_io = self.records.iter().map(|r| r.disk_io).sum();
        self.statistics.total_network_io = self.records.iter().map(|r| r.network_io).sum();
        
        if self.statistics.total_processes > 0 {
            self.statistics.avg_cpu_time = self.statistics.total_cpu_time as f64 / self.statistics.total_processes as f64;
            self.statistics.avg_memory_usage = self.statistics.total_memory_usage as f64 / self.statistics.total_processes as f64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_record_creation() {
        let record = ProcessRecord {
            pid: 1234,
            comm: "test".into(),
            uid: 1000,
            gid: 1000,
            start_time: 1000,
            end_time: 2000,
            cpu_time: 100,
            memory_usage: 1024,
            disk_io: 512,
            network_io: 256,
            exit_status: 0,
            args: vec!["test".into(), "--arg".into()],
        };
        
        assert_eq!(record.pid, 1234);
        assert_eq!(record.comm, "test");
        assert_eq!(record.cpu_time, 100);
    }
    
    #[test]
    fn test_accounting_manager_creation() {
        let manager = AccountingManager::new();
        assert_eq!(manager.get_statistics().total_processes, 0);
    }
    
    #[test]
    fn test_start_process() {
        let mut manager = AccountingManager::new();
        
        assert!(manager.start_process(1234, "test".into(), 1000, 1000, vec!["test".into()]));
        assert!(!manager.start_process(1234, "test2".into(), 1000, 1000, vec![])); // Duplicate PID
        
        assert_eq!(manager.get_statistics().total_processes, 1);
        assert!(manager.get_process(1234).is_some());
    }
    
    #[test]
    fn test_update_process() {
        let mut manager = AccountingManager::new();
        manager.start_process(1234, "test".into(), 1000, 1000, vec![]);
        
        assert!(manager.update_process(1234, 100, 1024, 512, 256));
        
        let process = manager.get_process(1234).unwrap();
        assert_eq!(process.cpu_time, 100);
        assert_eq!(process.memory_usage, 1024);
        assert_eq!(process.disk_io, 512);
        assert_eq!(process.network_io, 256);
    }
    
    #[test]
    fn test_end_process() {
        let mut manager = AccountingManager::new();
        manager.start_process(1234, "test".into(), 1000, 1000, vec![]);
        manager.update_process(1234, 100, 1024, 512, 256);
        
        assert!(manager.end_process(1234, 0));
        
        let process = manager.get_process(1234).unwrap();
        assert!(process.end_time > 0);
        assert_eq!(process.exit_status, 0);
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_cpu_time, 100);
        assert_eq!(stats.total_memory_usage, 1024);
    }
    
    #[test]
    fn test_get_processes_by_user() {
        let mut manager = AccountingManager::new();
        manager.start_process(1234, "test1".into(), 1000, 1000, vec![]);
        manager.start_process(1235, "test2".into(), 1001, 1001, vec![]);
        manager.start_process(1236, "test3".into(), 1000, 1000, vec![]);
        
        let user_processes = manager.get_processes_by_user(1000);
        assert_eq!(user_processes.len(), 2);
    }
    
    #[test]
    fn test_get_processes_by_group() {
        let mut manager = AccountingManager::new();
        manager.start_process(1234, "test1".into(), 1000, 1000, vec![]);
        manager.start_process(1235, "test2".into(), 1001, 1001, vec![]);
        manager.start_process(1236, "test3".into(), 1002, 1000, vec![]);
        
        let group_processes = manager.get_processes_by_group(1000);
        assert_eq!(group_processes.len(), 2);
    }
    
    #[test]
    fn test_get_top_processes_by_cpu() {
        let mut manager = AccountingManager::new();
        manager.start_process(1234, "test1".into(), 1000, 1000, vec![]);
        manager.update_process(1234, 100, 1024, 512, 256);
        
        manager.start_process(1235, "test2".into(), 1000, 1000, vec![]);
        manager.update_process(1235, 200, 2048, 1024, 512);
        
        let top = manager.get_top_processes_by_cpu(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].pid, 1235); // Higher CPU time
    }
    
    #[test]
    fn test_get_top_processes_by_memory() {
        let mut manager = AccountingManager::new();
        manager.start_process(1234, "test1".into(), 1000, 1000, vec![]);
        manager.update_process(1234, 100, 1024, 512, 256);
        
        manager.start_process(1235, "test2".into(), 1000, 1000, vec![]);
        manager.update_process(1235, 200, 2048, 1024, 512);
        
        let top = manager.get_top_processes_by_memory(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].pid, 1235); // Higher memory usage
    }
    
    #[test]
    fn test_get_processes_by_command() {
        let mut manager = AccountingManager::new();
        manager.start_process(1234, "test".into(), 1000, 1000, vec![]);
        manager.start_process(1235, "test".into(), 1000, 1000, vec![]);
        manager.start_process(1236, "other".into(), 1000, 1000, vec![]);
        
        let test_processes = manager.get_processes_by_command("test");
        assert_eq!(test_processes.len(), 2);
    }
    
    #[test]
    fn test_process_duration() {
        let mut manager = AccountingManager::new();
        manager.start_process(1234, "test".into(), 1000, 1000, vec![]);
        
        // Process is still running
        let duration = manager.get_process_duration(1234).unwrap();
        assert!(duration >= 0);
        
        // End process
        manager.end_process(1234, 0);
        let duration = manager.get_process_duration(1234).unwrap();
        assert!(duration >= 0);
    }
    
    #[test]
    fn test_clean_old_records() {
        let mut manager = AccountingManager::new();
        
        // Add old record (simulate by directly manipulating)
        let old_record = ProcessRecord {
            pid: 9999,
            comm: "old".into(),
            uid: 1000,
            gid: 1000,
            start_time: 1000, // Very old
            end_time: 2000,
            cpu_time: 100,
            memory_usage: 1024,
            disk_io: 512,
            network_io: 256,
            exit_status: 0,
            args: vec![],
        };
        manager.records.push(old_record);
        manager.process_index.insert(9999, 0);
        manager.statistics.total_processes = 1;
        
        let removed = manager.clean_old_records(3600); // 1 hour
        assert!(removed > 0);
        assert!(manager.get_process(9999).is_none());
    }
    
    #[test]
    fn test_export_import() {
        let mut manager = AccountingManager::new();
        manager.start_process(1234, "test".into(), 1000, 1000, vec![]);
        manager.update_process(1234, 100, 1024, 512, 256);
        manager.end_process(1234, 0);
        
        let export = manager.export_data();
        
        let mut manager2 = AccountingManager::new();
        manager2.import_data(export);
        
        assert_eq!(manager2.get_statistics().total_processes, 1);
        assert!(manager2.get_process(1234).is_some());
    }
}
