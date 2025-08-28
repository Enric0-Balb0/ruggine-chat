// Factory for NewCpuUsageLog for integration tests
// Similar to text_message_factory, but for cpu_usage_log_service

use ruggine_server::entity::cpu_usage_log::NewCpuUsageLog;
use rand::Rng;

pub struct CpuUsageLogFactory;

impl CpuUsageLogFactory {
    pub fn unique_fake_new_cpu_usage_log() -> NewCpuUsageLog {
        let mut rng = rand::thread_rng();
        NewCpuUsageLog {
            cpu_usage_percent: rng.gen_range(0.0..100.0),
        }
    }
}
