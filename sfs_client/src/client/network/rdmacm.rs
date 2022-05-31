use std::ptr::null_mut;
use rdma_sys::{rdma_event_channel, rdma_cm_event, rdma_get_cm_event, rdma_ack_cm_event, rdma_event_str, rdma_cm_event_type::{RDMA_CM_EVENT_ESTABLISHED, RDMA_CM_EVENT_DISCONNECTED, RDMA_CM_EVENT_ADDR_RESOLVED, RDMA_CM_EVENT_ROUTE_RESOLVED}, rdma_resolve_route, rdma_conn_param, rdma_connect, rdma_cm_id, rdma_destroy_event_channel};
use sfs_rdma::{build_params, rdma::RDMAContext};
use tokio::sync::oneshot::{Sender};


pub struct RDMACMContext{
    pub ctx: u64,
    pub s_ctx: *mut RDMAContext,
    pub on_route_resolved: fn(*mut rdma_cm_id),
    pub on_established: fn(*mut rdma_cm_id),
    pub on_disconnect: fn(*mut rdma_cm_id),
    
    pub tx: Option<Sender<u64>>
}
pub fn process_cm_event(ec: u64){
    unsafe{
        let ec = ec as *mut rdma_event_channel;
        let mut cm_event: *mut rdma_cm_event = null_mut();
        let mut connected_id = 0;
        
        while rdma_get_cm_event(ec, &mut cm_event) == 0 {
            let ret = (*cm_event).status;
            if ret != 0 {
                println!("CM event {} has non zero status: {}", std::ffi::CStr::from_ptr(rdma_event_str((*cm_event).event)).to_string_lossy().into_owned(), ret);
                rdma_ack_cm_event(cm_event);
                continue;
            }
            match (*cm_event).event {
                RDMA_CM_EVENT_ADDR_RESOLVED => {
                    let cm_id = (*cm_event).id;
                    rdma_ack_cm_event(cm_event);
                    assert_eq!(rdma_resolve_route(cm_id, 2000), 0);
                }
                RDMA_CM_EVENT_ROUTE_RESOLVED => {
                    let cm_id = (*cm_event).id;
                    rdma_ack_cm_event(cm_event);

                    let ctx = (*cm_id).context as *mut RDMACMContext;
                    ((*ctx).on_route_resolved)(cm_id);
                    
                    // connect server
                    let mut cm_params: rdma_conn_param = std::mem::zeroed();
                    build_params(&mut cm_params);
                    rdma_connect(cm_id, &mut cm_params);
                }
                RDMA_CM_EVENT_ESTABLISHED => {
                    let cm_id = (*cm_event).id;
                    rdma_ack_cm_event(cm_event);

                    connected_id += 1;
                    
                    let ctx = (*cm_id).context as *mut RDMACMContext;
                    ((*ctx).on_established)(cm_id);
                }
                RDMA_CM_EVENT_DISCONNECTED => {
                    let cm_id = (*cm_event).id;
                    rdma_ack_cm_event(cm_event);
                    
                    let ctx = (*cm_id).context as *mut RDMACMContext;
                    ((*ctx).on_disconnect)(cm_id);

                    connected_id -= 1;
                    if connected_id == 0{
                        break;
                    }
                }
                _ => {
                    println!(
                        "unexpected event: {}",
                        std::ffi::CStr::from_ptr(rdma_event_str((*cm_event).event))
                            .to_string_lossy()
                            .into_owned()
                    );
                    break;
                }
            }
        }
        rdma_destroy_event_channel(ec);
    }
}