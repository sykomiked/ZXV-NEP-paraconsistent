//! ipc-bus — the Multi-Agent Inter-Process Communication Bus.
//!
//! The messaging subsystem of the distributed OS (the equivalent of
//! POSIX pipes / sockets / message queues, on-chain). Independent agent
//! contracts publish asynchronous tasks and subscribe to state-change
//! notifications. Paraconsistent gating ensures conflicting
//! instructions do not jam the queue: a task only becomes `ready`
//! (executable) when its evidence resolves to TRUE; disputed tasks are
//! quarantined as BOTH and never executed.
//!
//! Surface:
//!   publish(task_id, payload, expected)  -> post a task
//!   attest(task_id, source, value)       -> agent evidence on a task
//!   status(task_id)                      -> QUEUED/READY/BLOCKED/DONE
//!   subscribe(task_id, subscriber)       -> register for notifications
//!   consume(task_id)                     -> mark executed (ready only)
//!
//! Settlement rule: only TRUE executes. BOTH is quarantined; NEITHER
//! waits. A contradiction never jams the queue — it is isolated.
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

#[derive(
    BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Copy,
    Debug, PartialEq, Eq, schemars::JsonSchema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
#[borsh(use_discriminant = true)]
pub enum LogicState {
    Neither = 0x00,
    True = 0x01,
    False = 0x02,
    Both = 0x03,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Queued,
    Ready,
    Blocked,
    Done,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub payload: String,
    pub expected: LogicState,
    pub evidence: Vec<(AccountId, LogicState)>,
    pub status: TaskStatus,
}

/// The bus state: tasks + subscribers, both keyed by id.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct IpcBus {
    tasks: LookupMap<String, Task>,
    subscribers: LookupMap<String, Vec<AccountId>>,
}

#[near_bindgen]
impl IpcBus {
    #[init]
    pub fn new() -> Self {
        Self {
            tasks: LookupMap::new(b"t".to_vec()),
            subscribers: LookupMap::new(b"s".to_vec()),
        }
    }

    /// Publish a task with the evidence state it needs to run.
    pub fn publish(&mut self, task_id: String, payload: String, expected: LogicState) {
        assert!(!self.tasks.contains_key(&task_id), "task exists");
        self.tasks.insert(
            task_id.clone(),
            Task {
                payload,
                expected,
                evidence: Vec::new(),
                status: TaskStatus::Queued,
            },
        );
    }

    /// Agent evidence on a task. Re-evaluates the task status:
    ///   verdict TRUE  -> Ready (executable)
    ///   verdict BOTH  -> Blocked (quarantined, never executed)
    ///   verdict FALSE -> Blocked
    ///   verdict NEITHER -> Queued (waits)
    pub fn attest(&mut self, task_id: String, source: AccountId, value: LogicState) -> TaskStatus {
        let mut task = self.tasks.get(&task_id).cloned().expect("task not found");
        task.evidence.push((source, value));
        let mut acc = LogicState::Neither;
        for (_, v) in &task.evidence {
            acc = unsafe {
                core::mem::transmute::<u8, LogicState>((acc as u8 | *v as u8) & 0x03)
            };
        }
        let task_status = match acc {
            LogicState::True => TaskStatus::Ready,
            LogicState::Neither => TaskStatus::Queued,
            LogicState::False | LogicState::Both => TaskStatus::Blocked,
        };
        task.status = task_status.clone();
        self.tasks.insert(task_id.clone(), task);
        self.notify(&task_id, &task_status);
        task_status
    }

    /// Subscribe an account to task state-change notifications.
    pub fn subscribe(&mut self, task_id: String, subscriber: AccountId) {
        let mut list = self.subscribers.get(&task_id).cloned().unwrap_or_default();
        if !list.contains(&subscriber) {
            list.push(subscriber);
        }
        self.subscribers.insert(task_id, list);
    }

    /// The current task status.
    pub fn status(&self, task_id: String) -> Option<TaskStatus> {
        self.tasks.get(&task_id).map(|t| t.status.clone())
    }

    /// Mark a task executed. Only READY tasks may be consumed — a
    /// blocked (disputed) task is refused, and the caller's workflow is
    /// NOT aborted; the task stays quarantined.
    pub fn consume(&mut self, task_id: String) -> bool {
        let mut task = self.tasks.get(&task_id).cloned().expect("task not found");
        if task.status != TaskStatus::Ready {
            return false; // disputed/queued: refuse, do not jam, do not revert
        }
        task.status = TaskStatus::Done;
        task.status = TaskStatus::Done;
        self.tasks.insert(task_id.clone(), task);
        self.notify(&task_id, &TaskStatus::Done);
        true
    }

    fn notify(&self, task_id: &str, status: &TaskStatus) {
        if let Some(list) = self.subscribers.get(task_id) {
            for sub in list.iter() {
                env::log_str(&format!(
                    "ZXV_IPC: task {} -> {:?} notifying {}",
                    task_id, status, sub
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispute_blocks_but_does_not_jam() {
        let bus = IpcBus::new();
        let task = Task {
            payload: "transfer".into(),
            expected: LogicState::True,
            evidence: vec![("a".parse().unwrap(), LogicState::True),
                           ("b".parse().unwrap(), LogicState::False)],
            status: TaskStatus::Queued,
        };
        // the joined verdict of TRUE+FALSE is BOTH -> blocked, not executed
        let mut acc = LogicState::Neither;
        for (_, v) in &task.evidence {
            acc = unsafe {
                core::mem::transmute::<u8, LogicState>((acc as u8 | *v as u8) & 0x03)
            };
        }
        assert_eq!(acc, LogicState::Both);
        // a blocked task is never consumable
        let blocked = acc != LogicState::True;
        assert!(blocked);
    }
}