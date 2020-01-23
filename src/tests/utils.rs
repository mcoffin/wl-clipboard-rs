use std::{ffi::OsString, mem, thread, time::Duration};

use wayland_protocols::wlr::unstable::data_control::v1::server::zwlr_data_control_manager_v1::{
    Request as ServerManagerRequest, ZwlrDataControlManagerV1 as ServerManager,
};
use wayland_server::protocol::wl_seat::WlSeat as ServerSeat;

use crate::{tests::TestServer, utils::*};

#[test]
fn is_primary_selection_supported_test() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerSeat, _>(6, |_, _, _| {});
    server.display
          .create_global::<ServerManager, _>(2, |manager, _, _| {
              manager.quick_assign(|_, request, _| match request {
                         ServerManagerRequest::GetDataDevice { id: device, .. } => {
                             device.primary_selection(None);
                         }
                         _ => unreachable!(),
                     });
          });

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child = thread::spawn(move || is_primary_selection_supported_internal(Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let result = child.join().unwrap().unwrap();
    assert_eq!(result, true);
}

#[test]
fn is_primary_selection_supported_primary_selection_unsupported() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerSeat, _>(6, |_, _, _| {});
    server.display
          .create_global::<ServerManager, _>(2, |manager, _, _| {
              manager.quick_assign(|_, request, _| match request {
                         ServerManagerRequest::GetDataDevice { .. } => {
                             // Not sending primary_selection means it's not
                             // supported.
                         }
                         _ => unreachable!(),
                     });
          });

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child = thread::spawn(move || is_primary_selection_supported_internal(Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let result = child.join().unwrap().unwrap();
    assert_eq!(result, false);
}

#[test]
fn is_primary_selection_supported_data_control_v1() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerManager, _>(1, |_, _, _| {});

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child = thread::spawn(move || is_primary_selection_supported_internal(Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let result = child.join().unwrap().unwrap();
    assert_eq!(result, false);
}

#[test]
fn is_primary_selection_supported_no_seats() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerManager, _>(2, |_, _, _| {});

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child = thread::spawn(move || is_primary_selection_supported_internal(Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let error = child.join().unwrap().unwrap_err();
    if let PrimarySelectionCheckError::NoSeats = error {
        // Pass
    } else {
        panic!("Invalid error: {:?}", error);
    }
}

#[test]
fn supports_v2_seats() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerSeat, _>(2, |_, _, _| {});
    server.display
          .create_global::<ServerManager, _>(2, |manager, _, _| manager.quick_assign(|_, _, _| {}));

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child = thread::spawn(move || is_primary_selection_supported_internal(Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let res = child.join().unwrap();
    if let Err(PrimarySelectionCheckError::NoSeats) = res {
        panic!("Invalid error: {:?}", res);
    }
}

#[test]
fn is_primary_selection_supported_no_data_control() {
    let mut server = TestServer::new();

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child = thread::spawn(move || is_primary_selection_supported_internal(Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let error = child.join().unwrap().unwrap_err();
    if let PrimarySelectionCheckError::MissingProtocol { name, version } = error {
        assert_eq!(name, "zwlr_data_control_manager_v1");
        assert_eq!(version, 1);
    } else {
        panic!("Invalid error: {:?}", error);
    }
}
