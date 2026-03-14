pub mod packet_parser;
pub mod process_mapper;

use crossbeam_channel::Sender;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::capture::packet_parser::ConnectionEvent;

pub struct CaptureEngine {
    running: Arc<AtomicBool>,
    interface_name: String,
}

impl CaptureEngine {
    pub fn new(interface_name: Option<String>) -> Self {
        let iface = interface_name.unwrap_or_else(|| Self::default_interface());
        Self {
            running: Arc::new(AtomicBool::new(false)),
            interface_name: iface,
        }
    }

    pub fn default_interface() -> String {
        pcap::Device::lookup()
            .ok()
            .flatten()
            .map(|d| d.name)
            .unwrap_or_else(|| "any".to_string())
    }

    pub fn list_interfaces() -> Vec<String> {
        pcap::Device::list()
            .unwrap_or_default()
            .into_iter()
            .map(|d| d.name)
            .collect()
    }

    pub fn set_interface(&mut self, name: String) {
        self.interface_name = name;
    }

    pub fn start(&self, sender: Sender<Vec<ConnectionEvent>>) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Capture already running".to_string());
        }

        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);
        let interface = self.interface_name.clone();

        std::thread::Builder::new()
            .name("pcap-capture".to_string())
            .spawn(move || {
                if let Err(e) = Self::capture_loop(&interface, &sender, &running) {
                    log::error!("Capture loop error: {e}");
                }
                running.store(false, Ordering::SeqCst);
            })
            .map_err(|e| format!("Failed to spawn capture thread: {e}"))?;

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn capture_loop(
        interface: &str,
        sender: &Sender<Vec<ConnectionEvent>>,
        running: &Arc<AtomicBool>,
    ) -> Result<(), String> {
        let mut cap = pcap::Capture::from_device(interface)
            .map_err(|e| format!("Failed to open device '{interface}': {e}"))?
            .promisc(true)
            .snaplen(256)
            .timeout(500)
            .buffer_size(1_000_000)
            .open()
            .map_err(|e| format!("Failed to activate capture on '{interface}': {e}"))?;

        cap.filter("ip", true)
            .map_err(|e| format!("Failed to set BPF filter: {e}"))?;

        let mut batch: Vec<ConnectionEvent> = Vec::with_capacity(128);
        let mut last_flush = std::time::Instant::now();
        let flush_interval = std::time::Duration::from_millis(500);

        while running.load(Ordering::SeqCst) {
            match cap.next_packet() {
                Ok(packet) => {
                    if let Some(event) = packet_parser::parse_packet(packet.data, packet.header.ts) {
                        batch.push(event);
                    }
                }
                Err(pcap::Error::TimeoutExpired) => {
                    // Normal timeout, just check if we should flush
                }
                Err(e) => {
                    log::warn!("Packet read error: {e}");
                    continue;
                }
            }

            if last_flush.elapsed() >= flush_interval && !batch.is_empty() {
                let events = std::mem::take(&mut batch);
                if sender.send(events).is_err() {
                    log::error!("Receiver dropped, stopping capture");
                    break;
                }
                last_flush = std::time::Instant::now();
            }
        }

        // Flush remaining
        if !batch.is_empty() {
            let _ = sender.send(batch);
        }

        Ok(())
    }
}
