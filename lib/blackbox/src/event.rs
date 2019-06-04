// Copyright 2019 Facebook, Inc.
//
// This software may be used and distributed according to the terms of the
// GNU General Public License version 2 or any later version.

//! Events used in blackbox.
//!
//! This is specified to the host application (source control) use-case. Other
//! applications might want to define different events.
//!
//! This module assumes that all events are known here. There are no external
//! types of events that are outside this module.

use failure::Fallible;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

pub use serde_json::Value;

// Most serde attributes are used extensively to reduce the space usage.
//
// The 'alias' attribute is used for converting from JSON, as an easy way to
// construct the native Event type from a JSON coming from the Python land.

/// All possible [`Event`]s for the (source control) application.
///
/// Changing this `enum` and its dependencies needs to be careful to avoid
/// breaking the ability to read old data. Namely:
///
/// - Use (short) `serde rename` everywhere. Once a `rename` was assigned,
///   do not change its value.
/// - When adding new fields to an event type, consider `serde default` to
///   make it compatible with old data.
/// - Always use enum struct form `Event::TypeName { a: .., b: .. }`,
///   instead of enum tuple form `Event::TypeName(a, b)`.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Event {
    /// Resolved alias.
    #[serde(rename = "A", alias = "alias")]
    Alias {
        #[serde(rename = "F", alias = "from")]
        from: String,

        #[serde(rename = "T", alias = "to")]
        to: String,
    },

    /// Waiting for other operations (ex. editor).
    ///
    /// Not including watchman commands or network operations.
    /// They have dedicated event types.
    #[serde(rename = "B", alias = "blocked")]
    Blocked {
        #[serde(rename = "O", alias = "op")]
        op: BlockedOp,

        #[serde(
            rename = "N",
            alias = "name",
            default,
            skip_serializing_if = "is_default"
        )]
        name: Option<String>,

        #[serde(rename = "D", alias = "duration_ms")]
        duration_ms: u64,
    },

    /// A subset of interesting configs.
    #[serde(rename = "C", alias = "config")]
    Config {
        #[serde(rename = "I", alias = "interactive")]
        interactive: bool,

        #[serde(rename = "M", alias = "items")]
        items: BTreeMap<String, String>,
    },

    /// Free-form debug message.
    #[serde(rename = "D", alias = "debug")]
    Debug {
        #[serde(rename = "V", alias = "value")]
        value: Value,
    },

    #[serde(rename = "E", alias = "exception")]
    Exception {
        #[serde(rename = "M", alias = "msg")]
        msg: String,
    },

    /// Information collected at the end of the process.
    #[serde(rename = "F", alias = "finish")]
    Finish {
        #[serde(rename = "E", alias = "exit_code")]
        exit_code: u8,

        #[serde(rename = "R", alias = "max_rss")]
        max_rss: u64,

        #[serde(rename = "D", alias = "duration_ms")]
        duration_ms: u64,
    },

    /// Legacy blackbox message for compatibility.
    #[serde(rename = "L", alias = "legacy_log")]
    LegacyLog {
        // Matches `ui.log(service, *msg, **opts)` API.
        #[serde(rename = "S", alias = "service")]
        service: String,

        #[serde(
            rename = "M",
            alias = "msg",
            default,
            skip_serializing_if = "is_default"
        )]
        msg: String,

        #[serde(
            rename = "O",
            alias = "opts",
            default,
            skip_serializing_if = "is_default"
        )]
        opts: Value,
    },

    /// A single network operation.
    #[serde(rename = "N", alias = "network")]
    Network {
        #[serde(rename = "O", alias = "op")]
        op: NetworkOp,

        #[serde(
            rename = "R",
            alias = "read_bytes",
            default,
            skip_serializing_if = "is_default"
        )]
        read_bytes: u64,

        #[serde(
            rename = "W",
            alias = "write_bytes",
            default,
            skip_serializing_if = "is_default"
        )]
        write_bytes: u64,

        #[serde(
            rename = "C",
            alias = "calls",
            default,
            skip_serializing_if = "is_default"
        )]
        calls: u64,

        #[serde(
            rename = "D",
            alias = "duration_ms",
            default,
            skip_serializing_if = "is_default"
        )]
        duration_ms: u64,

        #[serde(
            rename = "L",
            alias = "latency_ms",
            default,
            skip_serializing_if = "is_default"
        )]
        latency_ms: u64,

        /// Optional free-form extra metadata about the result.
        #[serde(
            rename = "R",
            alias = "result",
            default,
            skip_serializing_if = "is_default"
        )]
        result: Option<Value>,
    },

    #[serde(rename = "PE", alias = "perftrace")]
    PerfTrace {
        #[serde(rename = "M", alias = "msg")]
        msg: String,
    },

    /// Process tree.
    ///
    /// When collecting this information, the parent processes might exit.
    /// So it's a best effort.
    #[serde(rename = "PR", alias = "process_tree")]
    ProcessTree {
        #[serde(rename = "N", alias = "names")]
        names: Vec<String>,
    },

    #[serde(rename = "P", alias = "profile")]
    Profile {
        #[serde(rename = "M", alias = "msg")]
        msg: String,
    },

    /// Repo initialization with basic information attached.
    #[serde(rename = "R", alias = "repo")]
    Repo {
        #[serde(rename = "P", alias = "path")]
        path: String,

        #[serde(rename = "N", alias = "name")]
        name: String,
    },

    /// Immutable process environment.
    #[serde(rename = "S", alias = "start")]
    Start {
        #[serde(
            rename = "P",
            alias = "pid",
            default,
            skip_serializing_if = "is_default"
        )]
        pid: u32,

        #[serde(
            rename = "U",
            alias = "uid",
            default,
            skip_serializing_if = "is_default"
        )]
        uid: u32,

        #[serde(
            rename = "N",
            alias = "nice",
            default,
            skip_serializing_if = "is_default"
        )]
        nice: i32,

        // A subset of interesting environment variables.
        #[serde(rename = "E", alias = "env")]
        env: BTreeMap<String, String>,

        #[serde(rename = "A", alias = "args")]
        args: Vec<String>,
    },

    /// A watchman command has finished.
    #[serde(rename = "W", alias = "watchman")]
    Watchman {
        #[serde(rename = "A", alias = "args")]
        args: Value,

        #[serde(rename = "D", alias = "duration_ms")]
        duration_ms: u64,

        /// Optional free-form extra metadata about the result.
        #[serde(
            rename = "R",
            alias = "result",
            default,
            skip_serializing_if = "is_default"
        )]
        result: Option<Value>,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum NetworkOp {
    #[serde(rename = "t", alias = "ssh_gettreepack")]
    SshGetTreePack,

    #[serde(rename = "f", alias = "ssh_getfiles")]
    SshGetFiles,

    #[serde(rename = "p", alias = "ssh_getpack")]
    SshGetPack,

    #[serde(rename = "T", alias = "http_gettreepack")]
    HttpGetTreePack,

    #[serde(rename = "F", alias = "http_getfiles")]
    HttpGetFiles,

    #[serde(rename = "P", alias = "http_getpack")]
    HttpGetPack,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum BlockedOp {
    #[serde(rename = "E", alias = "editor")]
    Editor,

    #[serde(rename = "P", alias = "pager")]
    Pager,

    #[serde(rename = "D", alias = "extdiff")]
    ExtDiff,

    #[serde(rename = "H", alias = "exthook")]
    ExtHook,

    #[serde(rename = "h", alias = "pythonhook")]
    PythonHook,

    #[serde(rename = "B", alias = "bisect_check")]
    BisectCheck,

    #[serde(rename = "X", alias = "histedit_exec")]
    HisteditExec,

    #[serde(rename = "C", alias = "curses")]
    Curses,

    #[serde(rename = "M", alias = "mergedriver")]
    MergeDriver,
}

fn is_default<T: PartialEq + Default>(value: &T) -> bool {
    value == &Default::default()
}

fn json_to_string(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<cannot decode>".to_string())
}

impl Event {
    pub fn from_json(json: &str) -> Fallible<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Event::*;
        match self {
            Alias { from, to } => write!(f, "[command_alias] {:?} expands to {:?}", from, to)?,
            Blocked {
                op,
                name,
                duration_ms,
            } => match name {
                Some(name) => write!(
                    f,
                    "[blocked] {:?} ({}) finished in {} ms",
                    op, name, duration_ms
                )?,
                None => write!(f, "[blocked] {:?} finished in {} ms", op, duration_ms)?,
            },
            Config { items, interactive } => {
                let interactive = if *interactive {
                    "interactive"
                } else {
                    "non-interactive"
                };
                write!(
                    f,
                    "[config] {} {}",
                    interactive,
                    items
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(" ")
                )?;
            }
            Debug { value } => write!(f, "[debug] {}", json_to_string(value))?,
            Exception { msg } => write!(f, "[command_exception] {}", msg)?,
            Finish {
                exit_code,
                max_rss,
                duration_ms,
            } => {
                write!(
                    f,
                    "[commmand_finish] exited {} in {} ms, max RSS: {} bytes",
                    exit_code, duration_ms, max_rss
                )?;
            }
            LegacyLog {
                service,
                msg,
                opts: _,
            } => {
                write!(f, "[legacy][{}] {}", service, msg,)?;
            }
            Network {
                op,
                read_bytes,
                write_bytes,
                calls,
                duration_ms,
                latency_ms,
                result,
            } => {
                let result = match result {
                    Some(result) => format!(" with result {}", json_to_string(result)),
                    None => "".to_string(),
                };
                write!(
                    f,
                    "[network] {:?} finished in {} calls, duration {} ms, latency {} ms, read {} bytes, write {} bytes{}",
                    op, calls, duration_ms, latency_ms, read_bytes, write_bytes, result,
                )?;
            }
            Start {
                pid,
                uid,
                nice,
                env: _,
                args,
            } => {
                write!(
                    f,
                    "[command] {:?} started by uid {} as pid {} with nice {}",
                    args, uid, pid, nice
                )?;
            }
            PerfTrace { msg } => write!(f, "[perftrace] {}", msg)?,
            ProcessTree { names } => write!(f, "[process_tree] {}", names.join(" -> "))?,
            Profile { msg } => write!(f, "[profile] {}", msg)?,
            Watchman {
                args,
                duration_ms,
                result,
            } => {
                let result = match result {
                    Some(result) => format!(" with result {}", json_to_string(result)),
                    None => "".to_string(),
                };
                write!(
                    f,
                    "[watchman] command {} finished in {} ms{}",
                    json_to_string(args),
                    duration_ms,
                    result,
                )?;
            }
            _ => {
                // Fallback to "Debug"
                write!(f, "[uncategorized] {:?}", self)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_string() {
        // Construct Event from plain JSON, then convert it to String (Display).
        // Does not cover every type. But some interesting ones.

        assert_eq!(
            f(r#"{"alias":{"from":"a","to":"b"}}"#),
            "[command_alias] \"a\" expands to \"b\""
        );

        assert_eq!(
            f(r#"{"blocked":{"op":"editor","duration_ms":3000}}"#),
            "[blocked] Editor finished in 3000 ms"
        );

        assert_eq!(
            f(r#"{"blocked":{"op":"pythonhook","name":"foo","duration_ms":50}}"#),
            "[blocked] PythonHook (foo) finished in 50 ms"
        );

        assert_eq!(
            f(r#"{"config":{"interactive":false,"items":{"a.b":"1","a.c":"2"}}}"#),
            "[config] non-interactive a.b=1 a.c=2"
        );

        assert_eq!(
            f(r#"{"debug":{"value":["debug","msg"]}}"#),
            "[debug] [\"debug\",\"msg\"]"
        );

        assert_eq!(
            f(r#"{"legacy_log":{"service":"fsmonitor","msg":"command completed"}}"#),
            "[legacy][fsmonitor] command completed"
        );

        assert_eq!(
            f(r#"{"process_tree":{"names":["systemd","bash","node"]}}"#),
            "[process_tree] systemd -> bash -> node"
        );

        assert_eq!(
            f(r#"{"watchman":{"args":["state-enter","update",{"rev":"abcd"}],"duration_ms":42}}"#),
            "[watchman] command [\"state-enter\",\"update\",{\"rev\":\"abcd\"}] finished in 42 ms"
        );
    }

    /// Convenient way to convert from a JSON string to human-readable message.
    fn f(s: &str) -> String {
        format!("{}", Event::from_json(s).unwrap())
    }
}
