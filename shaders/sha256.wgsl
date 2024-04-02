//! Adapted from https://github.com/MarcoCiaramella/sha256-gpu

fn swap_endianess32(val: u32) -> u32 {
	return ((val >> 24u) & 0xffu) | ((val >> 8u) & 0xff00u) | ((val << 8u) & 0xff0000u) | ((val << 24u) & 0xff000000u);
}

fn shw(x: u32, n: u32) -> u32 {
	return (x << (n & 31u)) & 0xffffffffu;
}

fn r(x: u32, n: u32) -> u32 {
	return (x >> n) | shw(x, 32u - n);
}

fn g0(x: u32) -> u32 {
	return r(x, 7u) ^ r(x, 18u) ^ (x >> 3u);
}

fn g1(x: u32) -> u32 {
	return r(x, 17u) ^ r(x, 19u) ^ (x >> 10u);
}

fn s0(x: u32) -> u32 {
	return r(x, 2u) ^ r(x, 13u) ^ r(x, 22u);
}

fn s1(x: u32) -> u32 {
	return r(x, 6u) ^ r(x, 11u) ^ r(x, 25u);
}

fn maj(a: u32, b: u32, c: u32) -> u32 {
	return (a & b) ^ (a & c) ^ (b & c);
}

fn ch(e: u32, f: u32, g: u32) -> u32 {
	return (e & f) ^ ((~e) & g);
}

@group(0) @binding(0) var<storage, read> inputHeader: array<u32, 20>;
@group(0) @binding(1) var<storage, read> inputTarget: array<u32, 8>;
@group(0) @binding(2) var<storage, read_write> atomicFlag: atomic<u32>;
@group(0) @binding(3) var<storage, read_write> outputHeader: array<u32, 20>;

const workgroupSize: u32 = 64u;
const numWorkgroups: u32 = 256u;
const numThreads: u32 = workgroupSize * numWorkgroups;

fn sha256_80byte(m: array<u32, 20>) -> array<u32, 8> {
	// padding
	var message = array<u32, 32>(
		m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7],
		m[8], m[9], m[10], m[11], m[12], m[13], m[14], m[15],
		m[16], m[17], m[18], m[19], 0x00000080, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, swap_endianess32(640u)
	);

	var hash = array<u32, 8> (
		0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
		0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
	);

	var k = array<u32, 64> (
		0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u, 0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
		0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u, 0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
		0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu, 0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
		0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u, 0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
		0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u, 0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
		0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u, 0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
		0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u, 0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
		0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u, 0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u
	);

	var w: array<u32, 64> = array<u32, 64>();

	for (var i = 0u; i < 2u; i++) {
		let chunk_index = i * 16u;

		var a = hash[0];
		var b = hash[1];
		var c = hash[2];
		var d = hash[3];
		var e = hash[4];
		var f = hash[5];
		var g = hash[6];
		var h = hash[7];

		for (var j = 0u; j < 16u; j++){
			w[j] = swap_endianess32(message[chunk_index + j]);
		}

		for (var j = 16u; j < 64u; j++){
			w[j] = w[j - 16u] + g0(w[j - 15u]) + w[j - 7u] + g1(w[j - 2u]);
		}

		for (var j = 0u; j < 64u; j++){
			let t2 = s0(a) + maj(a, b, c);
			let t1 = h + s1(e) + ch(e, f, g) + k[j] + w[j];

			h = g;
			g = f;
			f = e;
			e = d + t1;
			d = c;
			c = b;
			b = a;
			a = t1 + t2;
		}

		hash[0] += a;
		hash[1] += b;
		hash[2] += c;
		hash[3] += d;
		hash[4] += e;
		hash[5] += f;
		hash[6] += g;
		hash[7] += h;
	}

	hash[0] = swap_endianess32(hash[0]);
	hash[1] = swap_endianess32(hash[1]);
	hash[2] = swap_endianess32(hash[2]);
	hash[3] = swap_endianess32(hash[3]);
	hash[4] = swap_endianess32(hash[4]);
	hash[5] = swap_endianess32(hash[5]);
	hash[6] = swap_endianess32(hash[6]);
	hash[7] = swap_endianess32(hash[7]);

	return hash;
}

fn sha256_32byte(m: array<u32, 8>) -> array<u32, 8> {
	// padding
	var message = array<u32, 16>(
		m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7],
		0x00000080, 0, 0, 0, 0, 0, 0, swap_endianess32(256u)
	);

	var hash = array<u32, 8>(
		0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
		0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
	);

	var k = array<u32, 64> (
		0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u, 0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
		0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u, 0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
		0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu, 0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
		0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u, 0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
		0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u, 0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
		0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u, 0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
		0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u, 0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
		0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u, 0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u
	);

	var w: array<u32, 64> = array<u32, 64>();

	for (var j = 0u; j < 16u; j++){
		w[j] = swap_endianess32(message[j]);
	}

	for (var j = 16u; j < 64u; j++){
		w[j] = w[j - 16u] + g0(w[j - 15u]) + w[j - 7u] + g1(w[j - 2u]);
	}

	var a = hash[0];
	var b = hash[1];
	var c = hash[2];
	var d = hash[3];
	var e = hash[4];
	var f = hash[5];
	var g = hash[6];
	var h = hash[7];

	for (var j = 0u; j < 64u; j++){
		let t2 = s0(a) + maj(a, b, c);
		let t1 = h + s1(e) + ch(e, f, g) + k[j] + w[j];

		h = g;
		g = f;
		f = e;
		e = d + t1;
		d = c;
		c = b;
		b = a;
		a = t1 + t2;
	}

	hash[0] += a;
	hash[1] += b;
	hash[2] += c;
	hash[3] += d;
	hash[4] += e;
	hash[5] += f;
	hash[6] += g;
	hash[7] += h;

	hash[0] = swap_endianess32(hash[0]);
	hash[1] = swap_endianess32(hash[1]);
	hash[2] = swap_endianess32(hash[2]);
	hash[3] = swap_endianess32(hash[3]);
	hash[4] = swap_endianess32(hash[4]);
	hash[5] = swap_endianess32(hash[5]);
	hash[6] = swap_endianess32(hash[6]);
	hash[7] = swap_endianess32(hash[7]);

	return hash;
}

/// Checks if hash < target
fn meets_target(hash: array<u32, 8>, targ: array<u32, 8>) -> bool {
	if (hash[7] < targ[7]) {
		return true;
	} else if (hash[7] > targ[7]) {
		return false;
	}

	if (hash[6] < targ[6]) {
		return true;
	} else if (hash[6] > targ[6]) {
		return false;
	}

	if (hash[5] < targ[5]) {
		return true;
	} else if (hash[5] > targ[5]) {
		return false;
	}

	if (hash[4] < targ[4]) {
		return true;
	} else if (hash[4] > targ[4]) {
		return false;
	}

	if (hash[3] < targ[3]) {
		return true;
	} else if (hash[3] > targ[3]) {
		return false;
	}

	if (hash[2] < targ[2]) {
		return true;
	} else if (hash[2] > targ[2]) {
		return false;
	}

	if (hash[1] < targ[1]) {
		return true;
	} else if (hash[1] > targ[1]) {
		return false;
	}

	if (hash[0] <= targ[0]) {
		return true;
	}

	return false;
}

@compute @workgroup_size(workgroupSize, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
	if (global_id.x >= 1) {
		return;
	}

	var localHeader: array<u32, 20>;

	for (var i = 0u; i < 20u; i++) {
		localHeader[i] = inputHeader[i];
	}

	var hash = array<u32, 8> (
		0, 0xffffffff, 0, 0, 0, 0, 0, 0
	);//sha256_32byte(sha256_80byte(localHeader));

	for (var i = 0u; i < 8u; i++) {
		outputHeader[i] = hash[i];
	}

	for (var i = 9u; i < (9u + 8u); i++) {
		outputHeader[i] = inputTarget[i - 9];
	}

	let index: u32 = global_id.x;
	var nonce: u32 = index;

	loop {
		localHeader[19] = nonce;

		// double sha256
		let hash = sha256_32byte(sha256_80byte(localHeader));

		if (meets_target(hash, inputTarget)) {
			outputHeader = localHeader;
			atomicStore(&atomicFlag, 1u);

			break;
		}

		if (0xffffffffu - nonce < numThreads) {
			if (atomicLoad(&atomicFlag) != 0u) {
				break;
			}

			localHeader[17] += 1u;
			nonce = index;
		} else {
			nonce += numThreads;
		}
	}
}
