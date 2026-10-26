use std::io::{Read, Write};
use std::net::TcpStream;

// 比特币测试网 (Testnet3) 的魔数 (Magic Bytes)
// 主网是: 0xF9BEB4D9，测试网是: 0x0B110907
const TESTNET_MAGIC: [u8; 4] = [0x0B, 0x11, 0x09, 0x07];

fn main() -> std::io::Result<()> {
    // 这是一个公开的比特币测试网节点 Seed
    let peer_addr = "testnet-seed.bitcoin.jonasschnelli.ch:18333";
    println!("正在连接到比特币测试网节点: {}...", peer_addr);

    // 1. 建立基础 TCP 连接
    let mut stream = TcpStream::connect(peer_addr)?;
    println!("TCP 连接成功建立！");

    // 2. 构造并发送 Version 握手消息
    send_version_message(&mut stream)?;

    // 3. 读取对方节点响应的 P2P 消息
    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer)?;
    println!("收到来自节点原始字节流 ({} 字节): {:02X?}", n, &buffer[..n]);

    Ok(())
}

fn send_version_message(stream: &mut TcpStream) -> std::io::Result<()> {
    // 比特币 P2P 协议中，每条消息的结构为：
    // [Magic (4B)] + [Command Name (12B, 0-padded)] + [Payload Length (4B)] + [Checksum (4B)] + [Payload]

    let command = b"version\0\0\0\0\0"; // 必须凑满 12 字节
    
    // 这里我们先简化构造一个最小可用的 version 负载 Payload
    // 包含字段: version (4B), services (8B), timestamp (8B), addr_recv (26B), addr_from (26B), nonce (8B), user_agent (var_str), starting_height (4B), relay (1B)
    let mut payload = Vec::new();
    
    // Protocol version (例如 70015)
    payload.extend_from_slice(&70015_i32.to_le_bytes());
    // Node services (NODE_NETWORK = 1)
    payload.extend_from_slice(&1_u64.to_le_bytes());
    // Timestamp (当前时间戳)
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    payload.extend_from_slice(&timestamp.to_le_bytes());
    
    // 接收方地址结构 (Dummy 26 字节)
    payload.extend_from_slice(&[0u8; 26]);
    // 发送方地址结构 (Dummy 26 字节)
    payload.extend_from_slice(&[0u8; 26]);
    // Nonce (随机数，防自己连接自己)
    payload.extend_from_slice(&0x123456789abcdef0_u64.to_le_bytes());
    // User Agent (变长字符串，长度为 0 的 CompactSize 是 0x00)
    payload.push(0x00); 
    // Starting height (当前区块高度，填 0)
    payload.extend_from_slice(&0_i32.to_le_bytes());
    // Relay (是否接收交易广播，true = 1)
    payload.push(0x01);

    let payload_len = payload.len() as u32;

    // 计算 Checksum (Payload 的双 SHA256 哈希的前 4 字节)
    // 为保持代码简洁，这里先留空或计算，比特币标准要求双 sha256
    let checksum = [0x00, 0x00, 0x00, 0x00]; // 后面我们可以补上真实的 sha256 校验

    // 组装完整报文
    let mut packet = Vec::new();
    packet.extend_from_slice(&TESTNET_MAGIC);
    packet.extend_from_slice(command);
    packet.extend_from_slice(&payload_len.to_le_bytes());
    packet.extend_from_slice(&checksum);
    packet.extend_from_slice(&payload);

    // 发送给节点
    stream.write_all(&packet)?;
    println!("已向节点发送 Version 握手报文（{} 字节）", packet.len());

    Ok(())
}