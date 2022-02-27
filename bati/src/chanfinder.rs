use crate::cfg::ChannelRegistryCfg;
use crate::pilot_proto::PilotSender;
use bati_lib::{self as lib, ChannelRegistry, ChannelRegistryConf};
use log::error;
use ntex::{rt, time};

// ChanFinder负责监听channel配置启动对应的postman
pub struct ChanFinder {
    pilots: Vec<PilotSender>,
    chan_registry: ChannelRegistryCfg,
}

impl ChanFinder {
    pub fn new(chan_registry: ChannelRegistryCfg) -> Self {
        ChanFinder {
            chan_registry,
            pilots: vec![],
        }
    }

    pub fn add_pilot(&mut self, pilot: PilotSender) {
        self.pilots.push(pilot);
    }

    pub fn start(&self, pilots: Vec<PilotSender>) {
        let mut cfg = ChannelRegistryConf {
            file: self.chan_registry.file.clone(),
            consul: None,
        };
        if let Some(v) = self.chan_registry.consul.clone() {
            cfg.consul = Some(lib::ConsulConf {
                addr: v.addr,
                channel_path: v.channel_conf_path,
            });
        }
        let chan_reg = ChannelRegistry::new(cfg);

        rt::Arbiter::new().exec_fn(move || {
            let mut s = time::Millis(10);
            rt::spawn(async move {
                loop {
                    let _ = Box::pin(time::sleep(s)).await;
                    if s < time::Millis(5000) {
                        s = time::Millis(5000)
                    }
                    let channels = chan_reg.get_all_channels().await;
                    if channels.is_err() {
                        error!("failed to get all channels: {}", channels.err().unwrap());
                        continue;
                    }
                    let channels = channels.unwrap().clone();
                    let chan_confs = channels.clone();
                    let pilots = pilots.clone();
                    rt::spawn(async move {
                        for mut pilot in pilots {
                            for conf in chan_confs.iter() {
                                pilot
                                    .send_chanfinder_msg(conf.clone())
                                    .await
                                    .unwrap_or_else(|e| {
                                        error!("failed to send channel conf message: {}", e);
                                    });
                            }
                        }
                    });
                }
            });
        });
    }
}