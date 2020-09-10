// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod atomic_histogram;
pub mod aws;
pub mod cluster;
pub mod cluster_builder;
pub mod cluster_swarm;
pub mod effects;
pub mod experiments;
pub mod genesis_helper;
pub mod github;
pub mod health;
pub mod instance;
pub mod prometheus;
pub mod report;
pub mod slack;
pub mod stats;
pub mod suite;
pub mod tx_emitter;

pub mod util {

    pub fn human_readable_bytes_per_sec(bytes_per_sec: f64) -> String {
        if bytes_per_sec.round() < 1024.0 {
            return format!("{:.0} Bps", bytes_per_sec);
        }

        let kbytes_per_sec = bytes_per_sec / 1024.0;
        if kbytes_per_sec.round() < 1024.0 {
            return format!("{:.0} KBps", kbytes_per_sec);
        }

        let mbytes_per_sec = kbytes_per_sec / 1024.0;
        format!("{:.2} MBps", mbytes_per_sec)
    }

    #[cfg(test)]
    mod tests {
        use crate::util::human_readable_bytes_per_sec;

        #[test]
        fn test_human_readable_bytes_per_sec() {
            assert_eq!(&human_readable_bytes_per_sec(0.3), "0 Bps");
            assert_eq!(&human_readable_bytes_per_sec(0.7), "1 Bps");
            assert_eq!(&human_readable_bytes_per_sec(1.0), "1 Bps");
            assert_eq!(&human_readable_bytes_per_sec(1023.4), "1023 Bps");
            assert_eq!(&human_readable_bytes_per_sec(1023.5), "1 KBps");
            assert_eq!(&human_readable_bytes_per_sec(1024.0 * 3.5), "4 KBps");
            assert_eq!(&human_readable_bytes_per_sec(1024.0 * 1023.4), "1023 KBps");
            assert_eq!(&human_readable_bytes_per_sec(1024.0 * 1023.5), "1.00 MBps");
            assert_eq!(
                &human_readable_bytes_per_sec(1024.0 * 1024.0 * 2.28),
                "2.28 MBps"
            );
        }
    }
}
