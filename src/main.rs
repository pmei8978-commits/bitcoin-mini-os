use std::io::{Read, Write};
use std::net::TcpStream;

use sha2::{Sha256, Digest};

use secp256k1::{Secp256k1, SecretKey, PublicKey};


// 比特币测试网 (Testnet3) 的魔数 (Magic Bytes)
// 主网是: 0xF9BEB4D9，测试网是: 0x0B110907
const TESTNET_MAGIC: [u8; 4] = [0x0B, 0x11, 0x09, 0x07];

fn main() -> std::io::Result<()> {

    // 1. 先调用我们刚刚写的密码学钱包生成函数！
    generate_wallet();

    // 这是一个公开的比特币测试网节点 Seed
    let peer_addr = "testnet-seed.bitcoin.jonasschnelli.ch:18333";
    println!("正在连接到比特币测试网节点: {}...", peer_addr);

    // 1. 建立基础 TCP 连接
    let mut stream = TcpStream::connect(peer_addr)?;
    println!("TCP 连接成功建立！");

    // 2. 构造并发送 Version 握手消息
    send_version_message(&mut stream)?;

    // 3. 接收来自节点的响应并回复 verack
    println!("等待接收节点响应...");
    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer)?;
    
    if n > 0 {
        println!("收到来自节点原始字节流 ({} 字节): {:02X?}", n, &buffer[..n]);
        
        // 收到对方的 version/verack 后，我们需要向对方回发一个 verack 报文
        send_verack_message(&mut stream)?;
    } else {
        println!("对方节点关闭了连接。");
    }
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

    // 计算真实的双 SHA256 校验和！
    let checksum = calculate_checksum(&payload);

    // 组装完整报文
    let mut packet = Vec::new();
    packet.extend_from_slice(&TESTNET_MAGIC);
    packet.extend_from_slice(command);
    packet.extend_from_slice(&payload_len.to_le_bytes());
    packet.extend_from_slice(&checksum); // 填入真正的校验和
    packet.extend_from_slice(&payload);

    // 发送给节点
    stream.write_all(&packet)?;
    println!("已向节点发送 Version 握手报文（{} 字节）", packet.len());

    Ok(())
}


// 计算比特币标准的双 SHA256 校验和（取前 4 字节）
fn calculate_checksum(payload: &[u8]) -> [u8; 4] {
    // 第一次 SHA256
    let mut hasher1 = Sha256::new();
    hasher1.update(payload);
    let result1 = hasher1.finalize();

    // 第二次 SHA256
    let mut hasher2 = Sha256::new();
    hasher2.update(&result1);
    let result2 = hasher2.finalize();

    // 取前 4 字节
    let mut checksum = [0u8; 4];
    checksum.copy_from_slice(&result2[..4]);
    checksum
}



// 发送 verack 确认报文
fn send_verack_message(stream: &mut TcpStream) -> std::io::Result<()> {
    const TESTNET_MAGIC: [u8; 4] = [0x0B, 0x11, 0x09, 0x07];
    let command = b"verack\0\0\0\0\0\0"; // 凑满 12 字节
    let payload: &[u8] = &[];             // payload 为空
    let payload_len = 0u32;

    // 计算空 payload 的双 SHA256 校验和
    let checksum = calculate_checksum(payload);

    // 组装完整报文
    let mut packet = Vec::new();
    packet.extend_from_slice(&TESTNET_MAGIC);
    packet.extend_from_slice(command);
    packet.extend_from_slice(&payload_len.to_le_bytes());
    packet.extend_from_slice(&checksum);
    // 因为 payload 为空，所以不需要 extend payload

    stream.write_all(&packet)?;
    println!("已向节点发送 verack 握手确认报文（{} 字节）", packet.len());

    Ok(())
}


fn generate_wallet() {
    // 1. 初始化 Secp256k1 上下文
    let _secp = Secp256k1::new();

    // 2. 使用 rand v0.10 直接生成 32 字节的随机数组作为私钥
    // rand::random() 可以直接生成任意基本类型或定长数组
    let secret_bytes: [u8; 32] = rand::random();

    // 3. 从随机字节构造私钥
    let secret_key = SecretKey::from_secret_bytes(secret_bytes)
        .expect("32 random bytes should be a valid secret key");
    
    println!("生成的私钥 (Secret Key): {:02X?}", secret_key.to_secret_bytes());

    // 4. 从私钥推导公钥
    let public_key = PublicKey::from_secret_key(&secret_key);
    println!("生成的公钥 (Public Key): {}", public_key);
}






