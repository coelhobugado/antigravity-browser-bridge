//! Application layer for goal-oriented Antigravity work.
//!
//! MCP is an adapter only. This module owns work identity, state transitions,
//! deadlines, idempotency, cancellation, journaling, checkpoints and the
//! authenticated bridge transport.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex, OnceLock,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::runtime::{Builder, Runtime};
use tokio::time::timeout;
use uuid::Uuid;

const SCHEMA_VERSION: u32 = 1;
const PROTOCOL_VERSION: &str = "1.0";
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const DEFAULT_TASK_DEADLINE_MS: u64 = 120_000;
const DEFAULT_STEP_DEADLINE_MS: u64 = 30_000;
const DEFAULT_TRANSPORT_DEADLINE_MS: u64 = 20_000;
const DEFAULT_VERIFICATION_DEADLINE_MS: u64 = 10_000;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn generate(prefix: &str) -> Self {
                Self(format!("{}-{}", prefix, Uuid::new_v4()))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

id_type!(WorkId);
id_type!(StepId);
id_type!(AttemptId);
id_type!(RequestId);
id_type!(IdempotencyKey);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkState {
    Created,
    Planning,
    WaitingForTab,
    Observing,
    WaitingForApproval,
    Executing,
    Verifying,
    Recovering,
    Completed,
    Failed,
    Cancelled,
}

impl WorkState {
    pub fn can_transition_to(self, next: Self) -> bool {
        use WorkState::*;
        matches!(
            (self, next),
            (Created, Planning | Cancelled)
                | (Planning, WaitingForTab | Failed | Cancelled)
                | (WaitingForTab, Observing | Failed | Cancelled)
                | (
                    Observing,
                    WaitingForApproval | Executing | Failed | Cancelled
                )
                | (WaitingForApproval, Executing | Failed | Cancelled)
                | (Executing, Verifying | Recovering | Failed | Cancelled)
                | (Verifying, Completed | Recovering | Failed | Cancelled)
                | (Recovering, Observing | Executing | Failed | Cancelled)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkGoal {
    pub objective: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub risk: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deadline {
    pub task_ms: u64,
    pub step_ms: u64,
    pub transport_ms: u64,
    pub verification_ms: u64,
}

impl Default for Deadline {
    fn default() -> Self {
        Self {
            task_ms: DEFAULT_TASK_DEADLINE_MS,
            step_ms: DEFAULT_STEP_DEADLINE_MS,
            transport_ms: DEFAULT_TRANSPORT_DEADLINE_MS,
            verification_ms: DEFAULT_VERIFICATION_DEADLINE_MS,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionIntent {
    pub action: String,
    #[serde(default)]
    pub tab_id: Option<u64>,
    #[serde(default)]
    pub target: Value,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Postcondition {
    pub kind: String,
    #[serde(default)]
    pub expected: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkStep {
    pub id: StepId,
    pub intent: ActionIntent,
    #[serde(default)]
    pub postconditions: Vec<Postcondition>,
    pub deadline: Deadline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPlan {
    pub version: u32,
    pub steps: Vec<WorkStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResult {
    pub success: bool,
    #[serde(default)]
    pub data: Value,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRecord {
    pub schema_version: u32,
    pub work_id: WorkId,
    pub request_id: RequestId,
    pub goal: WorkGoal,
    pub plan: WorkPlan,
    pub state: WorkState,
    pub current_step: Option<StepId>,
    pub current_attempt: Option<AttemptId>,
    pub created_at_ms: u128,
    pub updated_at_ms: u128,
    pub deadline: Deadline,
    pub tab_id: Option<u64>,
    pub origin: Option<String>,
    pub document_generation: Option<String>,
    pub last_observation: Option<Value>,
    pub result: Option<WorkResult>,
    pub effects_confirmed: Vec<String>,
    pub approval_receipt: Option<Value>,
    pub idempotency_results: HashMap<IdempotencyKey, WorkResult>,
    pub next_decision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub schema_version: u32,
    pub checkpoint_id: String,
    pub created_at_ms: u128,
    pub work: WorkRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkStatus {
    pub work_id: WorkId,
    pub state: WorkState,
    pub current_step: Option<StepId>,
    pub progress: f32,
    pub blocked: bool,
    pub next_decision: Option<String>,
    pub result: Option<WorkResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkErrorCode {
    InvalidRequest,
    InvalidTransition,
    Transport,
    Authorization,
    Target,
    Navigation,
    Policy,
    Verification,
    Site,
    DeadlineExceeded,
    Cancelled,
    Conflict,
    Persistence,
    NotFound,
    Unsupported,
}

impl WorkErrorCode {
    fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::InvalidTransition => "invalid_transition",
            Self::Transport => "transport",
            Self::Authorization => "authorization",
            Self::Target => "target",
            Self::Navigation => "navigation",
            Self::Policy => "policy",
            Self::Verification => "verification",
            Self::Site => "site",
            Self::DeadlineExceeded => "deadline_exceeded",
            Self::Cancelled => "cancelled",
            Self::Conflict => "conflict",
            Self::Persistence => "persistence",
            Self::NotFound => "not_found",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkError {
    pub code: WorkErrorCode,
    pub message: String,
    #[serde(default)]
    pub details: Value,
}

impl WorkError {
    fn new(code: WorkErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: Value::Null,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WorkOperation {
    Start,
    Observe,
    Execute,
    Verify,
    Checkpoint,
    Resume,
    Export,
    RequestApproval,
    Status,
    Cancel,
    Journal,
}

#[async_trait]
pub trait WorkClient: Send + Sync {
    async fn call(&self, operation: WorkOperation, args: Value) -> Result<Value, WorkError>;
}

#[async_trait]
trait BridgeTransport: Send + Sync {
    async fn request(&self, request: Value, deadline: Duration) -> Result<Value, WorkError>;
}

#[derive(Debug, Clone, Copy, Default)]
struct NativeBridgeTransport;

#[async_trait]
impl BridgeTransport for NativeBridgeTransport {
    async fn request(&self, request: Value, deadline: Duration) -> Result<Value, WorkError> {
        timeout(
            deadline,
            tokio::task::spawn_blocking(move || call_bridge_blocking(request)),
        )
        .await
        .map_err(|_| WorkError::new(WorkErrorCode::DeadlineExceeded, "bridge deadline exceeded"))?
        .map_err(|error| WorkError::new(WorkErrorCode::Transport, error.to_string()))?
    }
}

#[derive(Clone)]
pub struct WorkService {
    root: Arc<PathBuf>,
    records: Arc<Mutex<HashMap<WorkId, WorkRecord>>>,
    cancelled: Arc<Mutex<HashMap<WorkId, Arc<std::sync::atomic::AtomicBool>>>>,
    idempotency: Arc<Mutex<HashMap<IdempotencyKey, Value>>>,
    transport: Arc<dyn BridgeTransport>,
}

impl WorkService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::with_transport(root, Arc::new(NativeBridgeTransport))
    }

    fn with_transport(root: impl Into<PathBuf>, transport: Arc<dyn BridgeTransport>) -> Self {
        let root = root.into();
        let service = Self {
            root: Arc::new(root),
            records: Arc::new(Mutex::new(HashMap::new())),
            cancelled: Arc::new(Mutex::new(HashMap::new())),
            idempotency: Arc::new(Mutex::new(HashMap::new())),
            transport,
        };
        let _ = service.load_journal();
        service
    }

    fn journal_path(&self) -> PathBuf {
        self.root.join("work-journal.jsonl")
    }

    fn checkpoint_dir(&self) -> PathBuf {
        self.root.join("checkpoints")
    }

    fn append_event(
        &self,
        event: &str,
        record: &WorkRecord,
        payload: Value,
    ) -> Result<(), WorkError> {
        if let Some(parent) = self.journal_path().parent() {
            fs::create_dir_all(parent)
                .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?;
        }
        let line = json!({
            "schemaVersion": SCHEMA_VERSION,
            "timestampMs": now_ms(),
            "event": event,
            "work": record,
            "payload": payload
        });
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.journal_path())
            .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?;
        serde_json::to_writer(&mut file, &line)
            .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?;
        file.write_all(b"\n")
            .and_then(|_| file.sync_data())
            .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))
    }

    fn load_journal(&self) -> Result<(), WorkError> {
        let path = self.journal_path();
        if !path.exists() {
            return Ok(());
        }
        let file = File::open(path)
            .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?;
        let mut records = self
            .records
            .lock()
            .map_err(|_| WorkError::new(WorkErrorCode::Persistence, "work store lock poisoned"))?;
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            if let Ok(event) = serde_json::from_str::<Value>(&line) {
                if let Some(work) = event.get("work") {
                    if let Ok(record) = serde_json::from_value::<WorkRecord>(work.clone()) {
                        records.insert(record.work_id.clone(), record);
                    }
                }
            }
        }
        Ok(())
    }

    fn insert_record(&self, record: WorkRecord) -> Result<(), WorkError> {
        self.append_event("created", &record, Value::Null)?;
        self.records
            .lock()
            .map_err(|_| WorkError::new(WorkErrorCode::Persistence, "work store lock poisoned"))?
            .insert(record.work_id.clone(), record);
        Ok(())
    }

    fn get_record(&self, work_id: &WorkId) -> Result<WorkRecord, WorkError> {
        self.records
            .lock()
            .map_err(|_| WorkError::new(WorkErrorCode::Persistence, "work store lock poisoned"))?
            .get(work_id)
            .cloned()
            .ok_or_else(|| {
                WorkError::new(
                    WorkErrorCode::NotFound,
                    format!("work {} was not found", work_id.as_str()),
                )
            })
    }

    fn save_record(&self, record: WorkRecord) -> Result<(), WorkError> {
        self.records
            .lock()
            .map_err(|_| WorkError::new(WorkErrorCode::Persistence, "work store lock poisoned"))?
            .insert(record.work_id.clone(), record);
        Ok(())
    }

    fn transition(
        &self,
        work_id: &WorkId,
        next: WorkState,
        payload: Value,
    ) -> Result<WorkRecord, WorkError> {
        let mut record = self.get_record(work_id)?;
        if !record.state.can_transition_to(next) {
            return Err(WorkError::new(
                WorkErrorCode::InvalidTransition,
                format!("cannot transition {:?} to {:?}", record.state, next),
            ));
        }
        record.state = next;
        record.updated_at_ms = now_ms();
        self.append_event("transition", &record, payload)?;
        self.save_record(record.clone())?;
        Ok(record)
    }

    fn cancellation_token(&self, work_id: &WorkId) -> Arc<std::sync::atomic::AtomicBool> {
        let mut tokens = self
            .cancelled
            .lock()
            .expect("work cancellation store lock poisoned");
        tokens
            .entry(work_id.clone())
            .or_insert_with(|| Arc::new(std::sync::atomic::AtomicBool::new(false)))
            .clone()
    }

    fn check_cancelled(&self, work_id: &WorkId) -> Result<(), WorkError> {
        if self
            .cancellation_token(work_id)
            .load(std::sync::atomic::Ordering::Acquire)
        {
            Err(WorkError::new(
                WorkErrorCode::Cancelled,
                "work was cancelled",
            ))
        } else {
            Ok(())
        }
    }

    async fn start(&self, args: Value) -> Result<Value, WorkError> {
        if let Some(key) = args
            .get("idempotencyKey")
            .and_then(Value::as_str)
            .map(IdempotencyKey::new)
        {
            if let Some(previous) = self
                .idempotency
                .lock()
                .map_err(|_| {
                    WorkError::new(
                        WorkErrorCode::Persistence,
                        "idempotency store lock poisoned",
                    )
                })?
                .get(&key)
                .cloned()
            {
                return Ok(json!({"reused": true, "result": previous}));
            }
            if let Some(previous) = self
                .records
                .lock()
                .map_err(|_| {
                    WorkError::new(WorkErrorCode::Persistence, "work store lock poisoned")
                })?
                .values()
                .find_map(|record| record.idempotency_results.get(&key).cloned())
            {
                return Ok(json!({"reused": true, "result": previous}));
            }
        }
        let work_id = args
            .get("workId")
            .and_then(Value::as_str)
            .map(WorkId::new)
            .unwrap_or_else(|| WorkId::generate("work"));
        let request_id_value = args
            .get("requestId")
            .and_then(Value::as_str)
            .map(RequestId::new)
            .unwrap_or_else(|| RequestId::generate("request"));
        if let Ok(existing) = self.get_record(&work_id) {
            return Ok(json!({"work": existing, "reused": true}));
        }
        let goal = WorkGoal {
            objective: args
                .get("objective")
                .and_then(Value::as_str)
                .unwrap_or("Operate the authorized browser tab")
                .to_string(),
            scope: args
                .get("scope")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            risk: args
                .get("risk")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        };
        let deadline = parse_deadline(&args);
        let record = WorkRecord {
            schema_version: SCHEMA_VERSION,
            work_id: work_id.clone(),
            request_id: request_id_value,
            goal,
            plan: WorkPlan {
                version: 1,
                steps: Vec::new(),
            },
            state: WorkState::Created,
            current_step: None,
            current_attempt: None,
            created_at_ms: now_ms(),
            updated_at_ms: now_ms(),
            deadline,
            tab_id: args.get("tabId").and_then(Value::as_u64),
            origin: None,
            document_generation: None,
            last_observation: None,
            result: None,
            effects_confirmed: Vec::new(),
            approval_receipt: None,
            idempotency_results: HashMap::new(),
            next_decision: Some("observe an authorized tab".to_string()),
        };
        self.insert_record(record)?;
        self.transition(&work_id, WorkState::Planning, Value::Null)?;
        self.transition(&work_id, WorkState::WaitingForTab, Value::Null)?;
        let record = self.get_record(&work_id)?;
        let tabs = self
            .transport
            .request(
                json!({"protocolVersion": PROTOCOL_VERSION, "id": request_id(&args), "action": "tabs.authorized"}),
                Duration::from_millis(record.deadline.transport_ms),
            )
            .await?;
        let result = json!({"work": record, "tabs": tabs, "capabilities": ["observe", "execute", "verify", "checkpoint", "resume", "cancel", "export"]});
        if let Some(key) = args
            .get("idempotencyKey")
            .and_then(Value::as_str)
            .map(IdempotencyKey::new)
        {
            let mut persisted = self.get_record(&work_id)?;
            persisted.idempotency_results.insert(
                key.clone(),
                WorkResult {
                    success: true,
                    data: result.clone(),
                    error_code: None,
                    verified: true,
                },
            );
            self.append_event(
                "idempotency_bound",
                &persisted,
                json!({"key": key.as_str()}),
            )?;
            self.save_record(persisted)?;
            self.idempotency
                .lock()
                .map_err(|_| {
                    WorkError::new(
                        WorkErrorCode::Persistence,
                        "idempotency store lock poisoned",
                    )
                })?
                .insert(key, result.clone());
        }
        Ok(result)
    }

    async fn observe(&self, args: Value) -> Result<Value, WorkError> {
        let work_id = required_work_id(&args)?;
        self.check_cancelled(&work_id)?;
        let record = self.get_record(&work_id)?;
        ensure_task_deadline(&record)?;
        if matches!(
            record.state,
            WorkState::WaitingForTab | WorkState::Recovering
        ) {
            self.transition(&work_id, WorkState::Observing, Value::Null)?;
        } else if record.state != WorkState::Observing {
            return Err(WorkError::new(
                WorkErrorCode::InvalidTransition,
                "work must be waiting for a tab or recovering before observe",
            ));
        }
        let response = self.transport.request(json!({"protocolVersion": PROTOCOL_VERSION, "id": request_id(&args), "action": "observe", "tabId": args.get("tabId").and_then(Value::as_u64).or(record.tab_id)}), Duration::from_millis(record.deadline.transport_ms)).await?;
        self.check_cancelled(&work_id)?;
        let mut updated = self.get_record(&work_id)?;
        updated.last_observation = Some(response.clone());
        updated.tab_id = args.get("tabId").and_then(Value::as_u64).or(updated.tab_id);
        updated.updated_at_ms = now_ms();
        updated.next_decision = Some("execute a step or request approval".to_string());
        self.append_event("observed", &updated, response.clone())?;
        self.save_record(updated.clone())?;
        Ok(json!({"work": updated, "observation": response}))
    }

    async fn execute(&self, args: Value) -> Result<Value, WorkError> {
        let work_id = required_work_id(&args)?;
        self.check_cancelled(&work_id)?;
        let mut record = self.get_record(&work_id)?;
        ensure_task_deadline(&record)?;
        if let Some(key) = args
            .get("idempotencyKey")
            .and_then(Value::as_str)
            .map(IdempotencyKey::new)
        {
            if let Some(previous) = record.idempotency_results.get(&key) {
                return Ok(json!({"work": record, "result": previous, "reused": true}));
            }
        }
        if record.state == WorkState::WaitingForApproval {
            return Err(WorkError::new(
                WorkErrorCode::Policy,
                "approval is required before executing this step",
            ));
        }
        if record.state != WorkState::Observing && record.state != WorkState::Recovering {
            return Err(WorkError::new(
                WorkErrorCode::InvalidTransition,
                "work must be observing or recovering before execute",
            ));
        }
        let intent = parse_intent(&args)?;
        let step = WorkStep {
            id: StepId::generate("step"),
            intent: intent.clone(),
            postconditions: parse_postconditions(&args),
            deadline: parse_deadline(&args),
        };
        record.plan.steps.push(step.clone());
        record.current_step = Some(step.id.clone());
        record.current_attempt = Some(AttemptId::generate("attempt"));
        self.save_record(record.clone())?;
        self.transition(&work_id, WorkState::Executing, json!({"stepId": step.id}))?;
        self.check_cancelled(&work_id)?;
        let response = self.transport.request(json!({"protocolVersion": PROTOCOL_VERSION, "id": request_id(&args), "action": intent.action, "tabId": intent.tab_id.or(record.tab_id), "target": intent.target, "text": intent.text}), Duration::from_millis(step.deadline.transport_ms)).await?;
        self.check_cancelled(&work_id)?;
        self.transition(&work_id, WorkState::Verifying, json!({"stepId": step.id}))?;
        let verified = verify_response(
            &response,
            &step.postconditions,
            Duration::from_millis(step.deadline.verification_ms),
        )
        .await?;
        let result = WorkResult {
            success: true,
            data: response,
            error_code: None,
            verified,
        };
        let mut updated = self.get_record(&work_id)?;
        updated.result = Some(result.clone());
        updated.effects_confirmed.push(step.id.as_str().to_string());
        updated.next_decision = Some("work completed".to_string());
        if let Some(key) = args
            .get("idempotencyKey")
            .and_then(Value::as_str)
            .map(IdempotencyKey::new)
        {
            updated.idempotency_results.insert(key, result.clone());
        }
        self.append_event("effect_confirmed", &updated, json!({"stepId": step.id}))?;
        self.save_record(updated.clone())?;
        self.transition(
            &work_id,
            WorkState::Completed,
            json!({"verified": verified}),
        )?;
        let updated = self.get_record(&work_id)?;
        Ok(json!({"work": updated, "result": result}))
    }

    async fn verify(&self, args: Value) -> Result<Value, WorkError> {
        let work_id = required_work_id(&args)?;
        let record = self.get_record(&work_id)?;
        let result = record.result.clone().ok_or_else(|| {
            WorkError::new(
                WorkErrorCode::Verification,
                "no effect is available to verify",
            )
        })?;
        if record.state == WorkState::Verifying {
            self.transition(
                &work_id,
                WorkState::Completed,
                json!({"verified": result.verified}),
            )?;
        }
        Ok(
            json!({"work": self.get_record(&work_id)?, "verified": result.verified, "result": result}),
        )
    }

    fn status(&self, args: Value) -> Result<Value, WorkError> {
        let work_id = required_work_id(&args)?;
        let record = self.get_record(&work_id)?;
        let progress = if record.plan.steps.is_empty() {
            0.0
        } else if record.state == WorkState::Completed {
            1.0
        } else {
            0.5
        };
        Ok(
            json!({"status": WorkStatus { work_id, state: record.state, current_step: record.current_step, progress, blocked: record.state == WorkState::WaitingForApproval || record.state == WorkState::WaitingForTab, next_decision: record.next_decision, result: record.result }}),
        )
    }

    fn checkpoint(&self, args: Value) -> Result<Value, WorkError> {
        let work_id = required_work_id(&args)?;
        let work = self.get_record(&work_id)?;
        fs::create_dir_all(self.checkpoint_dir())
            .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?;
        let checkpoint = Checkpoint {
            schema_version: SCHEMA_VERSION,
            checkpoint_id: format!("checkpoint-{}", Uuid::new_v4()),
            created_at_ms: now_ms(),
            work,
        };
        let path = self
            .checkpoint_dir()
            .join(format!("{}.json", checkpoint.checkpoint_id));
        let bytes = serde_json::to_vec_pretty(&checkpoint)
            .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?;
        let mut file = File::create(&path)
            .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_data())
            .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?;
        self.append_event(
            "checkpoint",
            &checkpoint.work,
            json!({"checkpointId": checkpoint.checkpoint_id}),
        )?;
        Ok(json!({"checkpoint": checkpoint.checkpoint_id, "path": path, "work": checkpoint.work}))
    }

    fn resume(&self, args: Value) -> Result<Value, WorkError> {
        let checkpoint_id = args
            .get("checkpointId")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                WorkError::new(WorkErrorCode::InvalidRequest, "checkpointId is required")
            })?;
        let path = self
            .checkpoint_dir()
            .join(format!("{}.json", checkpoint_id));
        let checkpoint: Checkpoint = serde_json::from_slice(
            &fs::read(&path)
                .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?,
        )
        .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?;
        if checkpoint.schema_version != SCHEMA_VERSION {
            return Err(WorkError::new(
                WorkErrorCode::Unsupported,
                format!(
                    "checkpoint schema {} is not supported; export and recreate it with schema {}",
                    checkpoint.schema_version, SCHEMA_VERSION
                ),
            ));
        }
        self.save_record(checkpoint.work.clone())?;
        Ok(json!({"checkpoint": checkpoint_id, "work": checkpoint.work, "resumed": true}))
    }

    fn export(&self, args: Value) -> Result<Value, WorkError> {
        let work_id = required_work_id(&args)?;
        let work = self.get_record(&work_id)?;
        let mut exported = serde_json::to_value(work)
            .map_err(|error| WorkError::new(WorkErrorCode::Persistence, error.to_string()))?;
        redact(&mut exported);
        Ok(
            json!({"manifest": {"schemaVersion": SCHEMA_VERSION, "workId": work_id, "redacted": true}, "work": exported}),
        )
    }

    fn request_approval(&self, args: Value) -> Result<Value, WorkError> {
        let work_id = required_work_id(&args)?;
        let mut record = self.get_record(&work_id)?;
        if record.state != WorkState::Observing {
            return Err(WorkError::new(
                WorkErrorCode::InvalidTransition,
                "approval can only be requested while observing",
            ));
        }
        let receipt = json!({"receiptId": format!("approval-{}", Uuid::new_v4()), "intentHash": sha256_json(args.get("intent").unwrap_or(&Value::Null)), "createdAtMs": now_ms()});
        record.approval_receipt = Some(receipt.clone());
        self.save_record(record)?;
        let updated = self.transition(&work_id, WorkState::WaitingForApproval, receipt.clone())?;
        Ok(json!({"approval": receipt, "work": updated}))
    }

    fn cancel(&self, args: Value) -> Result<Value, WorkError> {
        let work_id = required_work_id(&args)?;
        self.cancellation_token(&work_id)
            .store(true, std::sync::atomic::Ordering::Release);
        let record = self.get_record(&work_id)?;
        if !matches!(
            record.state,
            WorkState::Completed | WorkState::Failed | WorkState::Cancelled
        ) {
            let updated = self.transition(
                &work_id,
                WorkState::Cancelled,
                json!({"reason": "requested by client"}),
            )?;
            return Ok(json!({"cancelled": true, "work": updated}));
        }
        Ok(json!({"cancelled": record.state == WorkState::Cancelled, "work": record}))
    }

    fn journal(&self, args: Value) -> Result<Value, WorkError> {
        let action = args
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("inspect");
        match action {
            "inspect" => {
                let work_filter = args.get("workId").and_then(Value::as_str);
                let mut events = Vec::new();
                if self.journal_path().exists() {
                    let file = File::open(self.journal_path()).map_err(|error| {
                        WorkError::new(WorkErrorCode::Persistence, error.to_string())
                    })?;
                    for line in BufReader::new(file).lines().map_while(Result::ok) {
                        if let Ok(mut event) = serde_json::from_str::<Value>(&line) {
                            if work_filter.is_none()
                                || event["work"]["workId"].as_str() == work_filter
                            {
                                redact(&mut event);
                                events.push(event);
                            }
                        }
                    }
                }
                Ok(
                    json!({"events": events, "count": events.len(), "retention": "explicit user-controlled deletion"}),
                )
            }
            "delete" => {
                if args.get("confirm").and_then(Value::as_bool) != Some(true) {
                    return Err(WorkError::new(
                        WorkErrorCode::Policy,
                        "journal deletion requires confirm=true",
                    ));
                }
                if self.journal_path().exists() {
                    fs::remove_file(self.journal_path()).map_err(|error| {
                        WorkError::new(WorkErrorCode::Persistence, error.to_string())
                    })?;
                }
                if self.checkpoint_dir().exists() {
                    fs::remove_dir_all(self.checkpoint_dir()).map_err(|error| {
                        WorkError::new(WorkErrorCode::Persistence, error.to_string())
                    })?;
                }
                self.records
                    .lock()
                    .map_err(|_| {
                        WorkError::new(WorkErrorCode::Persistence, "work store lock poisoned")
                    })?
                    .clear();
                self.cancelled
                    .lock()
                    .map_err(|_| {
                        WorkError::new(
                            WorkErrorCode::Persistence,
                            "cancellation store lock poisoned",
                        )
                    })?
                    .clear();
                self.idempotency
                    .lock()
                    .map_err(|_| {
                        WorkError::new(
                            WorkErrorCode::Persistence,
                            "idempotency store lock poisoned",
                        )
                    })?
                    .clear();
                Ok(json!({"deleted": true, "scope": "all_work_journal_events_and_checkpoints"}))
            }
            _ => Err(WorkError::new(
                WorkErrorCode::InvalidRequest,
                "journal action must be inspect or delete",
            )),
        }
    }
}

#[async_trait]
impl WorkClient for WorkService {
    async fn call(&self, operation: WorkOperation, args: Value) -> Result<Value, WorkError> {
        match operation {
            WorkOperation::Start => self.start(args).await,
            WorkOperation::Observe => self.observe(args).await,
            WorkOperation::Execute => self.execute(args).await,
            WorkOperation::Verify => self.verify(args).await,
            WorkOperation::Checkpoint => self.checkpoint(args),
            WorkOperation::Resume => self.resume(args),
            WorkOperation::Export => self.export(args),
            WorkOperation::RequestApproval => self.request_approval(args),
            WorkOperation::Status => self.status(args),
            WorkOperation::Cancel => self.cancel(args),
            WorkOperation::Journal => self.journal(args),
        }
    }
}

struct InstanceLock {
    path: PathBuf,
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub struct WorkRuntime {
    root: PathBuf,
    service: Mutex<Option<Arc<WorkService>>>,
    runtime: Runtime,
    last_activity: Mutex<Instant>,
    active_tasks: AtomicUsize,
    idle_timeout: Duration,
    instance_lock: Mutex<Option<InstanceLock>>,
}

impl WorkRuntime {
    fn create() -> Self {
        let root = dirs::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("agent-browser")
            .join("work");
        Self {
            root,
            service: Mutex::new(None),
            runtime: Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to create work runtime"),
            last_activity: Mutex::new(Instant::now()),
            active_tasks: AtomicUsize::new(0),
            idle_timeout: Duration::from_secs(15 * 60),
            instance_lock: Mutex::new(None),
        }
    }

    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<WorkRuntime> = OnceLock::new();
        INSTANCE.get_or_init(Self::create)
    }

    pub fn service(&self) -> Result<Arc<WorkService>, WorkError> {
        let mut service = self
            .service
            .lock()
            .expect("work runtime service lock poisoned");
        let expired = self.active_tasks.load(Ordering::Acquire) == 0
            && self
                .last_activity
                .lock()
                .expect("work runtime clock lock poisoned")
                .elapsed()
                >= self.idle_timeout;
        if service.is_none() || expired {
            if let Some(previous) = service.take() {
                let _ = previous;
            }
            let _ = fs::create_dir_all(&self.root);
            let mut instance_lock = self
                .instance_lock
                .lock()
                .expect("work runtime lock store poisoned");
            if instance_lock.is_none() {
                let lock_path = self.root.join("runtime.lock");
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&lock_path)
                    .map_err(|_| {
                        WorkError::new(
                            WorkErrorCode::Conflict,
                            "another Antigravity work runtime owns this user instance",
                        )
                    })?;
                writeln!(file, "{}", std::process::id()).map_err(|error| {
                    WorkError::new(WorkErrorCode::Persistence, error.to_string())
                })?;
                *instance_lock = Some(InstanceLock { path: lock_path });
            }
            *service = Some(Arc::new(WorkService::new(self.root.clone())));
        }
        *self
            .last_activity
            .lock()
            .expect("work runtime clock lock poisoned") = Instant::now();
        Ok(Arc::clone(
            service
                .as_ref()
                .expect("work runtime service not initialized"),
        ))
    }

    pub fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    fn call(&self, operation: WorkOperation, args: Value) -> Result<Value, WorkError> {
        self.active_tasks.fetch_add(1, Ordering::AcqRel);
        let result = match self.service() {
            Ok(service) => self.block_on(service.call(operation, args)),
            Err(error) => Err(error),
        };
        self.active_tasks.fetch_sub(1, Ordering::AcqRel);
        *self
            .last_activity
            .lock()
            .expect("work runtime clock lock poisoned") = Instant::now();
        result
    }

    pub fn should_stop(&self) -> bool {
        self.active_tasks.load(Ordering::Acquire) == 0
            && self
                .last_activity
                .lock()
                .expect("work runtime clock lock poisoned")
                .elapsed()
                >= self.idle_timeout
    }
}

pub fn call_global(operation: WorkOperation, args: Value) -> Result<Value, WorkError> {
    WorkRuntime::global().call(operation, args)
}

fn required_work_id(args: &Value) -> Result<WorkId, WorkError> {
    args.get("workId")
        .and_then(Value::as_str)
        .map(WorkId::new)
        .ok_or_else(|| WorkError::new(WorkErrorCode::InvalidRequest, "workId is required"))
}

fn request_id(args: &Value) -> String {
    args.get("requestId")
        .and_then(Value::as_str)
        .unwrap_or("request-generated")
        .to_string()
}

fn parse_deadline(args: &Value) -> Deadline {
    let mut deadline = Deadline::default();
    if let Some(value) = args.get("deadline").and_then(Value::as_object) {
        if let Some(value) = value.get("taskMs").and_then(Value::as_u64) {
            deadline.task_ms = value;
        }
        if let Some(value) = value.get("stepMs").and_then(Value::as_u64) {
            deadline.step_ms = value;
        }
        if let Some(value) = value.get("transportMs").and_then(Value::as_u64) {
            deadline.transport_ms = value;
        }
        if let Some(value) = value.get("verificationMs").and_then(Value::as_u64) {
            deadline.verification_ms = value;
        }
    }
    deadline
}

fn ensure_task_deadline(record: &WorkRecord) -> Result<(), WorkError> {
    let elapsed = now_ms().saturating_sub(record.created_at_ms);
    if elapsed > u128::from(record.deadline.task_ms) {
        Err(WorkError::new(
            WorkErrorCode::DeadlineExceeded,
            format!(
                "work task deadline of {}ms exceeded",
                record.deadline.task_ms
            ),
        ))
    } else {
        Ok(())
    }
}

fn parse_intent(args: &Value) -> Result<ActionIntent, WorkError> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .ok_or_else(|| WorkError::new(WorkErrorCode::InvalidRequest, "action is required"))?;
    if !matches!(action, "click" | "fill" | "type" | "focus" | "get_text") {
        return Err(WorkError::new(
            WorkErrorCode::Unsupported,
            format!("action {action} is not supported by the connector"),
        ));
    }
    let target = args.get("target").cloned().unwrap_or_else(|| json!({}));
    if !target.is_object() {
        return Err(WorkError::new(
            WorkErrorCode::InvalidRequest,
            "target must be an object",
        ));
    }
    Ok(ActionIntent {
        action: action.to_string(),
        tab_id: args.get("tabId").and_then(Value::as_u64),
        target,
        text: args
            .get("text")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

fn parse_postconditions(args: &Value) -> Vec<Postcondition> {
    args.get("postconditions")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| serde_json::from_value(item.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
}

async fn verify_response(
    response: &Value,
    postconditions: &[Postcondition],
    _deadline: Duration,
) -> Result<bool, WorkError> {
    if response
        .get("isError")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(WorkError::new(
            WorkErrorCode::Site,
            "bridge reported an action error",
        ));
    }
    for condition in postconditions {
        match condition.kind.as_str() {
            "contains" => {
                let expected = condition.expected.as_str().ok_or_else(|| {
                    WorkError::new(
                        WorkErrorCode::Verification,
                        "contains expected value must be text",
                    )
                })?;
                if !response.to_string().contains(expected) {
                    return Err(WorkError::new(
                        WorkErrorCode::Verification,
                        format!("response does not contain expected value: {expected}"),
                    ));
                }
            }
            "equals" => {
                if response != &condition.expected {
                    return Err(WorkError::new(
                        WorkErrorCode::Verification,
                        "response did not equal postcondition",
                    ));
                }
            }
            _ => {
                return Err(WorkError::new(
                    WorkErrorCode::Verification,
                    format!("unsupported postcondition kind {}", condition.kind),
                ))
            }
        }
    }
    Ok(true)
}

fn call_bridge_blocking(request: Value) -> Result<Value, WorkError> {
    let path = dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("agent-browser")
        .join("extension-bridge.json");
    let bytes = fs::read(&path)
        .map_err(|_| WorkError::new(WorkErrorCode::Transport, "Chrome connector is not running"))?;
    let state: Value = serde_json::from_slice(&bytes).map_err(|error| {
        WorkError::new(
            WorkErrorCode::Transport,
            format!("invalid bridge state: {error}"),
        )
    })?;
    if state.get("protocolVersion").and_then(Value::as_str) != Some(PROTOCOL_VERSION) {
        return Err(WorkError::new(
            WorkErrorCode::Transport,
            "unsupported bridge protocol",
        ));
    }
    let port = state
        .get("port")
        .and_then(Value::as_u64)
        .ok_or_else(|| WorkError::new(WorkErrorCode::Transport, "bridge port missing"))?
        as u16;
    let token = state
        .get("authToken")
        .and_then(Value::as_str)
        .ok_or_else(|| WorkError::new(WorkErrorCode::Authorization, "bridge token missing"))?;
    let address = ("127.0.0.1", port)
        .to_socket_addrs()
        .map_err(|error| WorkError::new(WorkErrorCode::Transport, error.to_string()))?
        .next()
        .ok_or_else(|| WorkError::new(WorkErrorCode::Transport, "bridge address unavailable"))?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(20))
        .map_err(|error| WorkError::new(WorkErrorCode::Transport, error.to_string()))?;
    stream.set_read_timeout(Some(Duration::from_secs(20))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(20))).ok();
    let mut payload = serde_json::to_vec(&json!({"authToken": token, "request": request}))
        .map_err(|error| WorkError::new(WorkErrorCode::Transport, error.to_string()))?;
    payload.push(b'\n');
    stream
        .write_all(&payload)
        .and_then(|_| stream.flush())
        .map_err(|error| WorkError::new(WorkErrorCode::Transport, error.to_string()))?;
    let mut reader = std::io::BufReader::new(stream);
    let mut line = String::new();
    let read = reader
        .read_line(&mut line)
        .map_err(|error| WorkError::new(WorkErrorCode::Transport, error.to_string()))?;
    if read == 0 || read > MAX_RESPONSE_BYTES {
        return Err(WorkError::new(
            WorkErrorCode::Transport,
            "invalid bridge response size",
        ));
    }
    serde_json::from_str(line.trim())
        .map_err(|error| WorkError::new(WorkErrorCode::Transport, error.to_string()))
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn sha256_json(value: &Value) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(value.to_string().as_bytes()))
}

fn redact(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map.iter_mut() {
                if ["password", "token", "authToken", "cookie", "secret"]
                    .iter()
                    .any(|s| key.eq_ignore_ascii_case(s))
                {
                    *value = Value::String("[REDACTED]".to_string());
                } else {
                    redact(value);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(redact),
        _ => {}
    }
}

pub fn response_from(result: Result<Value, WorkError>) -> Value {
    match result {
        Ok(value) => {
            json!({"isError": false, "content": [{"type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())}], "structuredContent": value})
        }
        Err(error) => {
            json!({"isError": true, "content": [{"type": "text", "text": format!("{}: {}", error.code.as_str(), error.message)}], "structuredContent": {"error": error}})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeTransport;

    #[async_trait]
    impl BridgeTransport for FakeTransport {
        async fn request(&self, request: Value, _deadline: Duration) -> Result<Value, WorkError> {
            Ok(json!({"ok": true, "action": request["action"], "request": request}))
        }
    }

    #[test]
    fn invalid_state_transitions_are_rejected() {
        assert!(WorkState::Created.can_transition_to(WorkState::Planning));
        assert!(!WorkState::Created.can_transition_to(WorkState::Executing));
        assert!(!WorkState::Completed.can_transition_to(WorkState::Observing));
    }

    #[test]
    fn terminal_states_have_no_outgoing_transitions() {
        for terminal in [
            WorkState::Completed,
            WorkState::Failed,
            WorkState::Cancelled,
        ] {
            for next in [
                WorkState::Created,
                WorkState::Planning,
                WorkState::WaitingForTab,
                WorkState::Observing,
                WorkState::WaitingForApproval,
                WorkState::Executing,
                WorkState::Verifying,
                WorkState::Recovering,
                WorkState::Completed,
                WorkState::Failed,
                WorkState::Cancelled,
            ] {
                assert!(!terminal.can_transition_to(next));
            }
        }
    }

    #[test]
    fn ids_are_distinct_types() {
        let work = WorkId::new("work");
        let step = StepId::new("step");
        assert_ne!(work.as_str(), step.as_str());
    }

    #[test]
    fn redaction_removes_sensitive_values() {
        let mut value = json!({"token": "secret", "nested": {"password": "pw", "safe": "ok"}});
        redact(&mut value);
        assert_eq!(value["token"], "[REDACTED]");
        assert_eq!(value["nested"]["password"], "[REDACTED]");
        assert_eq!(value["nested"]["safe"], "ok");
    }

    #[tokio::test]
    async fn service_journals_idempotent_effects_and_checkpoints() {
        let directory = tempfile::tempdir().unwrap();
        let service = WorkService::with_transport(directory.path(), Arc::new(FakeTransport));
        let start = service
            .call(
                WorkOperation::Start,
                json!({"workId":"work-test", "idempotencyKey":"start-1", "objective":"test"}),
            )
            .await
            .unwrap();
        assert_eq!(start["work"]["state"], "waiting_for_tab");
        let reused = service
            .call(
                WorkOperation::Start,
                json!({"workId":"different", "idempotencyKey":"start-1"}),
            )
            .await
            .unwrap();
        assert_eq!(reused["reused"], true);
        service
            .call(WorkOperation::Observe, json!({"workId":"work-test"}))
            .await
            .unwrap();
        let executed = service
            .call(
                WorkOperation::Execute,
                json!({"workId":"work-test", "idempotencyKey":"effect-1", "action":"click", "target":{"ref":"e1"}}),
            )
            .await
            .unwrap();
        assert_eq!(executed["work"]["state"], "completed");
        let duplicate = service
            .call(
                WorkOperation::Execute,
                json!({"workId":"work-test", "idempotencyKey":"effect-1", "action":"click", "target":{"ref":"e1"}}),
            )
            .await
            .unwrap();
        assert_eq!(duplicate["reused"], true);
        let checkpoint = service
            .call(WorkOperation::Checkpoint, json!({"workId":"work-test"}))
            .await
            .unwrap();
        assert!(checkpoint["checkpoint"].as_str().is_some());
        let journal = service
            .call(WorkOperation::Journal, json!({"action":"inspect"}))
            .await
            .unwrap();
        assert!(journal["count"].as_u64().unwrap() >= 4);
    }
}
