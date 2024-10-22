use criterion::{criterion_group, criterion_main, Criterion};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use std::sync::atomic::{AtomicUsize, Ordering};