// Implementation of K12, based on the reference implementation:
// https://github.com/gvanas/KeccakCodePackage/blob/master/Standalone/KangarooTwelve-reference/K12.py
//
// This is a straight port from Python and not well optimised.
//
// To the extent possible under law, the implementer has waived all copyright
// and related or neighboring rights to the source code in this file.
// http://creativecommons.org/publicdomain/zero/1.0/


// I can't be bothered to rename!
#![allow(non_snake_case)]

use std::cmp::min;

/// Copied from `arrayref` crate
macro_rules! array_ref {
    ($arr:expr, $offset:expr, $len:expr) => {{
        {
            #[inline]
            unsafe fn as_array<T>(slice: &[T]) -> &[T; $len] {
                &*(slice.as_ptr() as *const [_; $len])
            }
            let offset = $offset;
            let slice = & $arr[offset..offset + $len];
            unsafe {
                as_array(slice)
            }
        }
    }}
}

fn KeccakP1600onLanes(lanes: &mut [[u64; 5]; 5], nrRounds: u32) {
    fn ROL64(a: u64, n: u64) -> u64 {
        let n64 = n % 64;
        (a >> (64 - n64)) + (a << n64)
    }

    let mut R = 1u64;
    for round in 0..24 {
        if round + nrRounds >= 24 {
            // θ
            let mut C = [0u64; 5];
            for x in 0..5 {
                C[x] = lanes[x].iter().fold(0, |a, b| a ^ b);
            }
            
            let mut D = [0u64; 5];
            for x in 0..5 {
                D[x] = C[(x+4)%5] ^ ROL64(C[(x+1)%5], 1);
            }
            
            for x in 0..5 {
                for y in 0..5 {
                    lanes[x][y] ^= D[x];
                }
            }
            
            // ρ and π
            let (mut x, mut y) = (1, 0);
            let mut current = lanes[x][y];
            for t in 0..24 {
                let x0 = x;
                x = y;
                y = (2*x0 + 3*y) % 5;
                
                let current0 = current;
                current = lanes[x][y];
                lanes[x][y] = ROL64(current0, (t+1)*(t+2)/2);
            }
            
            // χ
            for y in 0..5 {
                let mut T = [0u64; 5];
                for x in 0..5 {
                    T[x] = lanes[x][y];
                }
                for x in 0..5 {
                    lanes[x][y] = T[x] ^((!T[(x+1) % 5]) & T[(x+2)%5]);
                }
            }
            
            // ι
            for j in 0..7 {
                R = ((R << 1) ^ ((R >> 7)*0x71)) % 256;
                if R & 2 != 0 {
                    lanes[0][0] ^= 1 << ((1<<j)-1);
                }
            }
        } else {
            for _ in 0..7 {
                R = ((R << 1) ^ ((R >> 7)*0x71)) % 256;
            }
        }
    }
}

fn read_u64(bytes: &[u8; 8]) -> u64 {
    unsafe{ *(bytes as *const [u8; 8] as *const u64) }.to_le()
}
fn write_u64(val: u64) -> [u8; 8] {
    unsafe{ *(&val.to_le() as *const u64 as *const [u8; 8]) }
}

fn KeccakP1600(state: &mut [u8; 200], nrRounds: u32) {
    let mut lanes = [[0u64; 5]; 5];
    for x in 0..5 {
        for y in 0..5 {
            lanes[x][y] = read_u64(array_ref!(state, 8*(x+5*y), 8));
        }
    }
    KeccakP1600onLanes(&mut lanes, nrRounds);
    for x in 0..5 {
        for y in 0..5 {
            let i = 8*(x+5*y);  // TODO index
            state[i..i+8].copy_from_slice(&write_u64(lanes[x][y]));
        }
    }
}

fn F(inputBytes: &[u8], delimitedSuffix: u8, mut outputByteLen: usize) -> Vec<u8> {
    let mut state = [0u8; 200];
    let rateInBytes = 1344 / 8;
    let mut blockSize = 0;
    let mut inputOffset = 0;
    
    // === Absorb all the input blocks ===
    while inputOffset < inputBytes.len() {
        blockSize = min(inputBytes.len() - inputOffset, rateInBytes);
        for i in 0..blockSize {
            // TODO: is this sufficiently optimisable or better to convert to u64 first?
            state[i] ^= inputBytes[i+inputOffset];
        }
        inputOffset += blockSize;
        if blockSize == rateInBytes {
            KeccakP1600(&mut state, 12);
            blockSize = 0;
        }
    }
    
    // === Do the padding and switch to the squeezing phase ===
    state[blockSize] ^= delimitedSuffix;
    if ((delimitedSuffix & 0x80) != 0) && (blockSize == (rateInBytes-1)) {
        KeccakP1600(&mut state, 12);
    }
    state[rateInBytes-1] ^= 0x80;
    KeccakP1600(&mut state, 12);
    
    // === Squeeze out all the output blocks ===
    let mut outputBytes = Vec::with_capacity(outputByteLen);
    while outputByteLen > 0 {
        blockSize = min(outputByteLen, rateInBytes);
        outputBytes.extend_from_slice(&state[0..blockSize]);
        outputByteLen -= blockSize;
        if outputByteLen > 0 {
            KeccakP1600(&mut state, 12);
        }
    }
    outputBytes
}

fn right_encode(mut x: usize) -> Vec<u8> {
    let mut S = Vec::new();
    while x > 0 {
        S.push((x % 256) as u8);
        x /= 256;
    }
    S.reverse();
    let len = S.len();
    S.push(len as u8);
    S
}

pub fn KangarooTwelve<TA: AsRef<[u8]>, TB: AsRef<[u8]>>(inputMessage: TA,
        customizationString: TB, outputByteLen: usize) -> Vec<u8>
{
    let B = 8192;
    let c = 256;
    
    let mut S = Vec::new();
    S.extend_from_slice(inputMessage.as_ref());
    S.extend_from_slice(customizationString.as_ref());
    S.extend_from_slice(&right_encode(customizationString.as_ref().len())[..]);
    
    // === Cut the input string into chunks of B bytes ===
    let n = (S.len() + B - 1) / B;
    let mut Si = Vec::with_capacity(n);
    for i in 0..n {
        let ub = min((i+1)*B, S.len());
        Si.push(&S[i*B .. ub]);
    }
    
    if n == 1 {
        // === Process the tree with only a final node ===
        F(Si[0], 0x07, outputByteLen)
    } else {
        // === Process the tree with kangaroo hopping ===
        // TODO: in parallel
        let mut CVi = Vec::with_capacity(n-1);
        for i in 0..n-1 {
            CVi.push(F(Si[i+1], 0x0B, c/8));
        }
        
        let mut NodeStar = Vec::new();
        NodeStar.extend_from_slice(Si[0]);
        NodeStar.extend_from_slice(&[3,0,0,0,0,0,0,0]);
        for i in 0..n-1 {
            NodeStar.extend_from_slice(&CVi[i][..]);
        }
        NodeStar.extend_from_slice(&right_encode(n-1));
        NodeStar.extend_from_slice(b"\xFF\xFF");
        
        F(&NodeStar[..], 0x06, outputByteLen)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter;
    
    fn read_bytes<T: AsRef<[u8]>>(s: T) -> Vec<u8> {
        fn b(c: u8) -> u8 {
            match c {
                b'0' ... b'9' => c - b'0',
                b'a' ... b'f' => c - b'a' + 10,
                b'A' ... b'F' => c - b'A' + 10,
                _ => unreachable!(),
            }
        }
        let s = s.as_ref();
        let mut i = 0;
        let mut v = Vec::new();
        while i < s.len() {
            if s[i] == b' ' || s[i] == b'\n' { i += 1; continue; }
            
            let n = b(s[i]) * 16 + b(s[i+1]);
            v.push(n);
            i += 2;
        }
        v
    }
    
    #[test]
    fn empty() {
        // Source: reference paper
        assert_eq!(KangarooTwelve("", "", 32), read_bytes("1a c2 d4 50 fc 3b 42 05 d1 9d a7 bf ca
                1b 37 51 3c 08 03 57 7a c7 16 7f 06 fe 2c e1 f0 ef 39 e5"));
        
        assert_eq!(KangarooTwelve("", "", 64), read_bytes("1a c2 d4 50 fc 3b 42 05 d1 9d a7 bf ca
                1b 37 51 3c 08 03 57 7a c7 16 7f 06 fe 2c e1 f0 ef 39 e5 42 69 c0 56 b8 c8 2e
                48 27 60 38 b6 d2 92 96 6c c0 7a 3d 46 45 27 2e 31 ff 38 50 81 39 eb 0a 71"));
        
        assert_eq!(KangarooTwelve("", "", 10032)[10000..], read_bytes("e8 dc 56 36 42 f7 22 8c 84
                68 4c 89 84 05 d3 a8 34 79 91 58 c0 79 b1 28 80 27 7a 1d 28 e2 ff 6d")[..]);
    }
    
    #[test]
    fn patM() {
        let expected = [
                "2b da 92 45 0e 8b 14 7f 8a 7c b6 29 e7 84 a0 58 ef ca 7c f7
                d8 21 8e 02 d3 45 df aa 65 24 4a 1f",
                "6b f7 5f a2 23 91 98 db 47 72 e3 64 78 f8 e1 9b 0f 37 12 05
                f6 a9 a9 3a 27 3f 51 df 37 12 28 88",
                "0c 31 5e bc de db f6 14 26 de 7d cf 8f b7 25 d1 e7 46 75 d7
                f5 32 7a 50 67 f3 67 b1 08 ec b6 7c",
                "cb 55 2e 2e c7 7d 99 10 70 1d 57 8b 45 7d df 77 2c 12 e3 22
                e4 ee 7f e4 17 f9 2c 75 8f 0d 59 d0",
                "87 01 04 5e 22 20 53 45 ff 4d da 05 55 5c bb 5c 3a f1 a7 71
                c2 b8 9b ae f3 7d b4 3d 99 98 b9 fe",
                "84 4d 61 09 33 b1 b9 96 3c bd eb 5a e3 b6 b0 5c c7 cb d6 7c
                ee df 88 3e b6 78 a0 a8 e0 37 16 82",
                "3c 39 07 82 a8 a4 e8 9f a6 36 7f 72 fe aa f1 32 55 c8 d9 58
                78 48 1d 3c d8 ce 85 f5 8e 88 0a f8"];
        for i in 0..5 /*NOTE: can be up to 7 but is slow*/ {
            let len = 17usize.pow(i);
            let M: Vec<u8> = (0..len).map(|j| (j % 251) as u8).collect();
            let result = KangarooTwelve(M, "", 32);
            assert_eq!(result, read_bytes(expected[i as usize]));
        }
    }
    
    #[test]
    fn patC() {
        let expected = [
                "fa b6 58 db 63 e9 4a 24 61 88 bf 7a f6 9a 13 30 45 f4 6e e9
                84 c5 6e 3c 33 28 ca af 1a a1 a5 83",
                "d8 48 c5 06 8c ed 73 6f 44 62 15 9b 98 67 fd 4c 20 b8 08 ac
                c3 d5 bc 48 e0 b0 6b a0 a3 76 2e c4",
                "c3 89 e5 00 9a e5 71 20 85 4c 2e 8c 64 67 0a c0 13 58 cf 4c
                1b af 89 44 7a 72 42 34 dc 7c ed 74",
                "75 d2 f8 6a 2e 64 45 66 72 6b 4f bc fc 56 57 b9 db cf 07 0c
                7b 0d ca 06 45 0a b2 91 d7 44 3b cf"];
        for i in 0..4 {
            let M: Vec<u8> = iter::repeat(0xFF).take(2usize.pow(i) - 1).collect();
            let len = 41usize.pow(i);
            let C: Vec<u8> = (0..len).map(|j| (j % 251) as u8).collect();
            let result = KangarooTwelve(M, C, 32);
            assert_eq!(result, read_bytes(expected[i as usize]));
        }
    }
}
