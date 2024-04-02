use std::borrow::Cow;

pub const SHADER_DESC: wgpu::ShaderModuleDescriptor = wgpu::ShaderModuleDescriptor {
	label: Some("SHA-256 Shader"),
	source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(concat!(
		env!("CARGO_MANIFEST_DIR"),
		"/shaders/sha256.wgsl"
	)))),
};

pub const BIND_GROUP_LAYOUT: wgpu::BindGroupLayoutDescriptor = wgpu::BindGroupLayoutDescriptor {
	label: Some("Compute Bind Group Layout"),
	entries: &[
		wgpu::BindGroupLayoutEntry {
			binding: 0,
			visibility: wgpu::ShaderStages::COMPUTE,
			ty: wgpu::BindingType::Buffer {
				ty: wgpu::BufferBindingType::Storage { read_only: true },
				has_dynamic_offset: false,
				min_binding_size: None,
			},
			count: None,
		},
		wgpu::BindGroupLayoutEntry {
			binding: 1,
			visibility: wgpu::ShaderStages::COMPUTE,
			ty: wgpu::BindingType::Buffer {
				ty: wgpu::BufferBindingType::Storage { read_only: true },
				has_dynamic_offset: false,
				min_binding_size: None,
			},
			count: None,
		},
		wgpu::BindGroupLayoutEntry {
			binding: 2,
			visibility: wgpu::ShaderStages::COMPUTE,
			ty: wgpu::BindingType::Buffer {
				ty: wgpu::BufferBindingType::Storage { read_only: false },
				has_dynamic_offset: false,
				min_binding_size: None,
			},
			count: None,
		},
	],
};

pub const INPUT_HEADER_DESC: wgpu::util::BufferInitDescriptor = wgpu::util::BufferInitDescriptor {
	label: Some("Input Header Buffer"),
	contents: &[0; 80],
	usage: wgpu::BufferUsages::from_bits_truncate(
		wgpu::BufferUsages::COPY_DST.bits() | wgpu::BufferUsages::STORAGE.bits(),
	),
};

pub const INPUT_TARGET_DESC: wgpu::util::BufferInitDescriptor = wgpu::util::BufferInitDescriptor {
	label: Some("Input Target Buffer"),
	contents: &[0; 32],
	usage: wgpu::BufferUsages::from_bits_truncate(
		wgpu::BufferUsages::COPY_DST.bits() | wgpu::BufferUsages::STORAGE.bits(),
	),
};

pub const OUTPUT_DESC: wgpu::BufferDescriptor = wgpu::BufferDescriptor {
	label: Some("Output Buffer"),
	size: 80,
	// https://github.com/bitflags/bitflags/issues/180
	usage: wgpu::BufferUsages::from_bits_truncate(
		wgpu::BufferUsages::COPY_SRC.bits() | wgpu::BufferUsages::STORAGE.bits(),
	),
	mapped_at_creation: false,
};

pub const MAPPABLE_DESC: wgpu::BufferDescriptor = wgpu::BufferDescriptor {
	label: Some("Mappable Buffer"),
	size: 80,
	usage: wgpu::BufferUsages::from_bits_truncate(
		wgpu::BufferUsages::MAP_READ.bits() | wgpu::BufferUsages::COPY_DST.bits(),
	),
	mapped_at_creation: false,
};
