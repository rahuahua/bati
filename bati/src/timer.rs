use crate::hub_proto::*;
use crate::metric_proto::*;
use crate::session_proto::*;
use log::warn;
use ntex::{rt, time};

pub fn start_metric_cron(mut sender: MetricSender) {
    rt::spawn(async move {
        loop {
            time::sleep(time::Millis(5000)).await;
            if let Err(e) = sender.send_timer_collect_msg().await {
                warn!("failed to send metric timer msg: {}", e.to_string());
                break;
            }
        }
    });
}

pub fn start_hub_stat_cron(mut sender: HubSender) {
    rt::spawn(async move {
        loop {
            time::sleep(time::Millis(5000)).await;
            if let Err(e) = sender.send_timer_msg(Timer2HubMsg::MetricStat).await {
                warn!("failed to send hub timer msg: {}", e.to_string());
            }
        }
    });
}

pub fn start_session_hearbeat_cron(sender: SessionSender) {
    rt::spawn(async move {
        loop {
            time::sleep(time::Millis(90_000)).await;
            if let Err(e) = sender.send_timer_msg(Timer2SessionMsg::HearBeatCheck).await {
                warn!(
                    "failed to send session timer msg: {} - {}",
                    sender.id,
                    e.to_string()
                );
                break;
            }
        }
    });
}
