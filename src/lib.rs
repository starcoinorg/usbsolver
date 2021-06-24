mod usb_solver;

use crate::usb_solver::UsbSolver;
use starcoin_miner_client_api::Solver;
#[no_mangle]
pub extern "C" fn create_solver() -> Box<dyn Solver> {
    Box::new(UsbSolver::new().expect("Failed to create usb solver"))
}

#[test]
fn test() {
    let _ = create_solver();
}
