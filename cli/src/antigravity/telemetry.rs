// Telemetria local opt-in para falhas sem envio de dados sensíveis
pub struct LocalTelemetry;

impl LocalTelemetry {
    pub fn record_failure(_error_type: &str) {
        // TODO: Implement
    }
}
