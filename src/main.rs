use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::convert::TryInto;
use std::fs;
use std::io::{self, Write};

// ======================
// HUFFMAN TREE
// ======================
#[derive(Clone)]
struct Node {
    freq: u32,
    byte: Option<u8>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}
impl Eq for Node {}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool { self.freq == other.freq }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// build huffman
fn build_huffman_tree(data: &[u8]) -> Node {
    let mut freq_map = HashMap::new();
    for &b in data {
        *freq_map.entry(b).or_insert(0u32) += 1;
    }
    if freq_map.len() == 1 {
        // edge case: only one symbol
        let (b, f) = freq_map.into_iter().next().unwrap();
        return Node{ freq:f, byte:Some(b), left:None, right:None };
    }
    let mut heap = BinaryHeap::new();
    for (b, f) in freq_map {
        heap.push(Node{ freq:f, byte:Some(b), left:None, right:None });
    }
    while heap.len() > 1 {
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        heap.push(Node{ freq:a.freq+b.freq, byte:None, left:Some(Box::new(a)), right:Some(Box::new(b)) });
    }
    heap.pop().unwrap()
}

fn build_codes(node: &Node, prefix: Vec<bool>, table: &mut HashMap<u8, Vec<bool>>) {
    if let Some(b) = node.byte {
        table.insert(b, prefix);
    } else {
        if let Some(ref l) = node.left {
            let mut p = prefix.clone();
            p.push(false);
            build_codes(l, p, table);
        }
        if let Some(ref r) = node.right {
            let mut p = prefix.clone();
            p.push(true);
            build_codes(r, p, table);
        }
    }
}

fn huffman_compress(data: &[u8]) -> (Vec<u8>, Node, usize) {
    let tree = build_huffman_tree(data);
    let mut table = HashMap::new();
    build_codes(&tree, Vec::new(), &mut table);
    let mut bits = Vec::new();
    for &b in data {
        if let Some(code) = table.get(&b) {
            bits.extend_from_slice(code);
        }
    }
    let mut out = Vec::new();
    let mut current = 0u8;
    let mut count = 0;
    for bit in bits {
        current <<= 1;
        if bit { current |= 1; }
        count += 1;
        if count == 8 {
            out.push(current);
            current = 0;
            count = 0;
        }
    }
    if count > 0 {
        current <<= 8 - count;
        out.push(current);
    }
    (out, tree, data.len())
}

fn huffman_decompress(data: &[u8], tree: &Node, orig_len: usize) -> Vec<u8> {
    let mut bits = Vec::<bool>::new();
    for &byte in data {
        for i in (0..8).rev() {
            bits.push(((byte >> i) & 1) == 1);
        }
    }
    let mut out = Vec::new();
    let mut node = tree;
    for bit in bits {
        node = if !bit { node.left.as_ref().unwrap() } else { node.right.as_ref().unwrap() };
        if let Some(b) = node.byte {
            out.push(b);
            if out.len() == orig_len {
                break;
            }
            node = tree;
        }
    }
    out
}

// serialize tree: pre-order traversal
fn serialize_tree(node: &Node, out: &mut Vec<u8>) {
    if let Some(b) = node.byte {
        out.push(1);
        out.push(b);
    } else {
        out.push(0);
        serialize_tree(node.left.as_ref().unwrap(), out);
        serialize_tree(node.right.as_ref().unwrap(), out);
    }
}
fn deserialize_tree(data: &[u8], idx: &mut usize) -> Node {
    let flag = data[*idx]; *idx += 1;
    if flag == 1 {
        let b = data[*idx]; *idx += 1;
        Node { freq:0, byte:Some(b), left:None, right:None }
    } else {
        let left = deserialize_tree(data, idx);
        let right = deserialize_tree(data, idx);
        Node { freq:0, byte:None, left:Some(Box::new(left)), right:Some(Box::new(right)) }
    }
}

// ======================
// LZ77 IMPLEMENTATION
// ======================
fn lz77_compress(data: &[u8]) -> Vec<(usize, usize, u8)> {
    let mut out = Vec::new();
    let window_size = 1024;
    let mut i = 0;
    while i < data.len() {
        let mut match_len = 0;
        let mut match_dist = 0;
        let search_start = if i >= window_size { i - window_size } else { 0 };
        for j in search_start..i {
            let mut k = 0;
            while i + k < data.len() && data[j + k] == data[i + k] {
                k += 1;
            }
            if k > match_len {
                match_len = k;
                match_dist = i - j;
            }
        }
        if match_len >= 3 {
            let next = if i + match_len < data.len() { data[i + match_len] } else { 0 };
            out.push((match_dist, match_len, next));
            i += match_len + 1;
        } else {
            out.push((0, 0, data[i]));
            i += 1;
        }
    }
    out
}
fn lz77_decompress(tokens: &[(usize, usize, u8)]) -> Vec<u8> {
    let mut out = Vec::new();
    for &(dist, len, next) in tokens {
        if dist == 0 && len == 0 {
            out.push(next);
        } else {
            let start = out.len() - dist;
            for i in 0..len {
                out.push(out[start + i]);
            }
            out.push(next);
        }
    }
    out
}

// helper to serialize/deserialize lz tokens
fn serialize_lz(tokens: &[(usize, usize, u8)]) -> Vec<u8> {
    let mut out = Vec::new();
    let count = tokens.len() as u32;
    out.extend_from_slice(&count.to_le_bytes());
    for (d, l, n) in tokens {
        out.extend_from_slice(&(*d as u32).to_le_bytes());
        out.extend_from_slice(&(*l as u32).to_le_bytes());
        out.push(*n);
    }
    out
}
fn deserialize_lz(data: &[u8]) -> Vec<(usize, usize, u8)> {
    let mut idx = 0;
    let count = u32::from_le_bytes(data[idx..idx+4].try_into().unwrap()) as usize;
    idx += 4;
    let mut tokens = Vec::with_capacity(count);
    for _ in 0..count {
        let d = u32::from_le_bytes(data[idx..idx+4].try_into().unwrap()) as usize;
        idx += 4;
        let l = u32::from_le_bytes(data[idx..idx+4].try_into().unwrap()) as usize;
        idx += 4;
        let n = data[idx];
        idx += 1;
        tokens.push((d, l, n));
    }
    tokens
}

// ======================
// FEISTEL ENCRYPTION
// ======================
fn round_function(input: u32, key: u32) -> u32 {
    let x = input.wrapping_add(key);
    x.rotate_left(5) ^ (x >> 3)
}
fn feistel_encrypt_block(mut left: u32, mut right: u32, keys: &[u32]) -> (u32, u32) {
    for &k in keys {
        let f = round_function(right, k);
        let new_left = right;
        let new_right = left ^ f;
        left = new_left;
        right = new_right;
    }
    (left, right)
}
fn feistel_decrypt_block(mut left: u32, mut right: u32, keys: &[u32]) -> (u32, u32) {
    for &k in keys.iter().rev() {
        let f = round_function(left, k);
        let new_right = left;
        let new_left = right ^ f;
        left = new_left;
        right = new_right;
    }
    (left, right)
}
fn derive_keys(key_material: &[u8]) -> Vec<u32> {
    let mut keys = Vec::new();
    for chunk in key_material.chunks(4) {
        let mut kbytes = [0u8; 4];
        for (i, &b) in chunk.iter().enumerate() {
            kbytes[i] = b;
        }
        keys.push(u32::from_le_bytes(kbytes));
    }
    keys
}
fn feistel_encrypt(data: &[u8], key_material: &[u8]) -> Vec<u8> {
    let keys = derive_keys(key_material);
    let mut out = Vec::new();
    for chunk in data.chunks(8) {
        let mut block = [0u8; 8];
        for (i, &b) in chunk.iter().enumerate() {
            block[i] = b;
        }
        let left = u32::from_le_bytes(block[0..4].try_into().unwrap());
        let right = u32::from_le_bytes(block[4..8].try_into().unwrap());
        let (el, er) = feistel_encrypt_block(left, right, &keys);
        out.extend_from_slice(&el.to_le_bytes());
        out.extend_from_slice(&er.to_le_bytes());
    }
    out
}
fn feistel_decrypt(data: &[u8], key_material: &[u8]) -> Vec<u8> {
    let keys = derive_keys(key_material);
    let mut out = Vec::new();
    for chunk in data.chunks(8) {
        let mut block = [0u8; 8];
        for (i, &b) in chunk.iter().enumerate() {
            block[i] = b;
        }
        let left = u32::from_le_bytes(block[0..4].try_into().unwrap());
        let right = u32::from_le_bytes(block[4..8].try_into().unwrap());
        let (dl, dr) = feistel_decrypt_block(left, right, &keys);
        out.extend_from_slice(&dl.to_le_bytes());
        out.extend_from_slice(&dr.to_le_bytes());
    }
    out
}

// ======================
// Rs-Zip CLI
// ======================
fn main() {
    loop {
        println!("\n=== Rs-Zip CLI Tool ===");
        println!("1) Compress file");
        println!("2) Decompress file");
        println!("3) Encrypt file");
        println!("4) Decrypt file");
        println!("5) Exit");
        print!("Choose option: ");
        io::stdout().flush().unwrap();
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        match choice.trim() {
            "1" => {
                let (input, output) = ask_paths();
                let data = fs::read(&input).expect("Failed to read input");
                let tokens = lz77_compress(&data);
                let lz_serial = serialize_lz(&tokens);
                let (huff, tree, orig_len) = huffman_compress(&lz_serial);

                let mut tree_bytes = Vec::new();
                serialize_tree(&tree, &mut tree_bytes);

                let mut final_out = Vec::new();
                final_out.extend_from_slice(&(orig_len as u32).to_le_bytes());
                final_out.extend_from_slice(&(tree_bytes.len() as u32).to_le_bytes());
                final_out.extend_from_slice(&tree_bytes);
                final_out.extend_from_slice(&huff);

                fs::write(&output, final_out).unwrap();
                println!("Compressed successfully!");
                pause();
            }
            "2" => {
                let (input, output) = ask_paths();
                let filedata = fs::read(&input).expect("Failed to read compressed file");
                let mut idx = 0;
                let orig_len = u32::from_le_bytes(filedata[idx..idx+4].try_into().unwrap()) as usize;
                idx += 4;
                let tree_size = u32::from_le_bytes(filedata[idx..idx+4].try_into().unwrap()) as usize;
                idx += 4;
                let tree_bytes = &filedata[idx..idx+tree_size];
                idx += tree_size;
                let huff_data = &filedata[idx..];

                let mut tree_idx = 0;
                let tree = deserialize_tree(tree_bytes, &mut tree_idx);
                let lz_serial = huffman_decompress(huff_data, &tree, orig_len);
                let tokens = deserialize_lz(&lz_serial);
                let decompressed = lz77_decompress(&tokens);

                fs::write(&output, decompressed).unwrap();
                println!("Decompressed successfully!");
                pause();
            }
            "3" => {
                let (input, output) = ask_paths();
                let key = ask_key();
                let data = fs::read(&input).expect("Failed to read input");
                let enc = feistel_encrypt(&data, key.as_bytes());
                fs::write(&output, enc).unwrap();
                println!("File encrypted!");
                pause();
            }
            "4" => {
                let (input, output) = ask_paths();
                let key = ask_key();
                let data = fs::read(&input).expect("Failed to read input");
                let dec = feistel_decrypt(&data, key.as_bytes());
                fs::write(&output, dec).unwrap();
                println!("File decrypted!");
                pause();
            }
            "5" => {
                println!("Exiting Rs-Zip.");
                break;
            }
            _ => {
                println!("Invalid choice!");
                pause();
            }
        }
    }
}

fn ask_paths() -> (String, String) {
    print!("Input file path: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_string();

    print!("Output file path: ");
    io::stdout().flush().unwrap();
    let mut output = String::new();
    io::stdin().read_line(&mut output).unwrap();
    let output = output.trim().to_string();

    (input, output)
}

fn ask_key() -> String {
    print!("Enter key (any string): ");
    io::stdout().flush().unwrap();
    let mut key = String::new();
    io::stdin().read_line(&mut key).unwrap();
    key.trim().to_string()
}

fn pause() {
    print!("\nPress ENTER to continue...");
    io::stdout().flush().unwrap();
    let mut _buf = String::new();
    io::stdin().read_line(&mut _buf).unwrap();
}
