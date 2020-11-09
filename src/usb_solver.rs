use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::executor::block_on;
use futures::SinkExt;
use starcoin_miner_client::{ConsensusStrategy, Solver, U256};
use std::io::Cursor;
use std::time::Duration;
use usbderive::{Config, DeriveResponse, UsbDerive};

#[derive(Clone)]
pub struct UsbSolver {
    derive: UsbDerive,
}

#[no_mangle]
impl UsbSolver {
    pub fn new() -> Result<Self> {
        let path = "/dev/cu.usbmodem2065325550561";
        let mut derive = UsbDerive::open(path, Config::default())?;
        derive.set_hw_params()?;
        derive.set_opcode()?;
        derive.get_state()?;
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
        _strategy: ConsensusStrategy,
        minting_blob: &[u8],
        diff: U256,
        mut nonce_tx: UnboundedSender<(Vec<u8>, u32)>,
        mut stop_rx: UnboundedReceiver<bool>,
    ) {
        let target = UsbSolver::difficulty_to_target_u32(diff);
        self.derive.set_job(0x1, target, minting_blob).unwrap();
        loop {
            if stop_rx.try_next().is_ok() {
                break;
            }
            match self.derive.read() {
                Ok(resp) => {
                    match resp {
                        DeriveResponse::SolvedJob(seal) => {
                            block_on(async {
                                let _ = nonce_tx.send((minting_blob.to_owned(), seal.nonce)).await;
                            });
                            break;
                        }
                        _ => {
                            //TODO:process it
                            continue;
                        }
                    }
                }
                Err(_e) => {
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }
}
