use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::executor::block_on;
use futures::SinkExt;
use starcoin_logger::prelude::*;
use starcoin_types::{U256, system_events::{SealEvent, MintBlockEvent}, block::BlockHeaderExtra};
use std::io::{Cursor, Write};
use usbderive::{Config, DeriveResponse, UsbDerive};
use rand::Rng;
use std::borrow::BorrowMut;
use starcoin_miner_client_api::Solver;

#[derive(Clone)]
pub struct UsbSolver {
    derive: UsbDerive,
}

const VID: u16 = 1155;
const PID: u16 = 22336;

impl UsbSolver {
    pub fn new() -> Result<Self> {
        let _ = starcoin_logger::init();
        let ports = UsbDerive::detect(VID, PID)?;
        let mut usb_derive: Option<UsbDerive> = None;
        for port in ports {
            match UsbDerive::open(&port.port_name, Config::default()) {
                Ok(derive) => {
                    usb_derive = Some(derive);
                    break;
                }
                Err(e) => {
                    warn!("Failed to open port:{:?}", e);
                    continue;
                }
            }
        }
        let mut derive = match usb_derive {
            None => anyhow::bail!("No usb derive found"),
            Some(d) => d,
        };
        derive.set_hw_params()?;
        derive.set_opcode()?;
        info!("Usb solver inited");

        Ok(Self { derive })
    }

    fn difficulty_to_target_u32(difficulty: U256) -> u32 {
        let target = U256::max_value() / difficulty;
        let mut tb = [0u8; 32];
        target.to_big_endian(tb.as_mut());
        let mut data = Cursor::new(tb);
        data.read_u32::<BigEndian>().unwrap()
    }
}

impl Solver for UsbSolver {
    fn solve(
        &mut self,
        event: MintBlockEvent,
        nonce_tx: UnboundedSender<SealEvent>,
        mut stop_rx: UnboundedReceiver<bool>,
    ) {
        let target = UsbSolver::difficulty_to_target_u32(event.difficulty);
        let mut rng = rand::thread_rng();
        let job_id: u8 = rng.gen();
        let mut nonce_tx = nonce_tx.clone();
        let mut blob = event.minting_blob.clone();
        let extra = match &event.extra {
            None => { BlockHeaderExtra::new([0u8; 4]) }
            Some(e) => { e.extra }
        };
        let _ = blob[35..39].borrow_mut().write_all(extra.as_slice());

        if let Err(e) = self.derive.set_job(job_id, target, &blob) {
            error!("Set mint job to derive failed: {:?}", e);
            return;
        }

        loop {
            if stop_rx.try_next().is_ok() {
                debug!("Stop solver");
                break;
            }
            // Blocking read since the poll has non-zero timeout
            match self.derive.read() {
                Ok(resp) => match resp {
                    DeriveResponse::SolvedJob(seal) => {
                        block_on(async {
                            let _ = nonce_tx.send(SealEvent {
                                minting_blob: event.minting_blob.clone(),
                                nonce: seal.nonce,
                                extra: event.extra,
                                hash_result: hex::encode(seal.hash),
                            }).await;
                        });
                        break;
                    }
                    resp => {
                        info!("get resp {:?}", resp);
                        continue;
                    }
                },
                Err(e) => {
                    debug!("Failed to solve: {:?}", e);
                }
            }
            info!("job_id:{:?}", job_id);
            let _ = self.derive.write_state();
            if let Err(e) = self.derive.set_job(job_id, target, &blob) {
                error!("Reset mint job to derive failed: {:?}", e);
                return;
            }
        }
    }
}
