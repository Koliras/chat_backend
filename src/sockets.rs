use socketioxide::extract::SocketRef;

pub async fn on_connect(socket: SocketRef) {
    println!("socket connected: {}", socket.id);
}
