use std::num::NonZeroU32;

use wgpu::{include_wgsl, util::DeviceExt as _};

pub fn main() {
	futures::executor::block_on(test(
		*b"abcdefghijklmpopqrsduvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzab",
	));
}

pub async fn test(input: [u8; 80]) {
	let instance = wgpu::Instance::default();
	let adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions::default())
		.await
		.unwrap();
	let (device, queue) = adapter
		.request_device(&wgpu::DeviceDescriptor::default(), None)
		.await
		.unwrap();

	// Create the compute pipeline
	let shader = device.create_shader_module(include_wgsl!("../sha256.wgsl"));

	let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: Some("Compute Bind Group Layout"),
		entries: &[
			// Input buffer
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
			// Input size buffer
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
			// Atomic u32 flag
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
			// Output buffer
			wgpu::BindGroupLayoutEntry {
				binding: 3,
				visibility: wgpu::ShaderStages::COMPUTE,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Storage { read_only: false },
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
		],
	});

	let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Compute Pipeline Layout"),
		bind_group_layouts: &[&bind_group_layout],
		push_constant_ranges: &[],
	});

	let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
		label: Some("Compute Pipeline"),
		layout: Some(&pipeline_layout),
		module: &shader,
		entry_point: "main",
	});

	let input_header_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("Input Header Buffer"),
		contents: bytemuck::cast_slice(&input),
		usage: wgpu::BufferUsages::STORAGE,
	});

	let input_target_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("Input Target Buffer"),
		// TODO: use actual target
		contents: bytemuck::cast_slice(&[0u8; 32]),
		usage: wgpu::BufferUsages::STORAGE,
	});

	let atomic_flag_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("Atomic Flag Buffer"),
		contents: bytemuck::cast_slice(&[0u32]),
		usage: wgpu::BufferUsages::STORAGE,
	});

	let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		label: Some("Output Buffer"),
		size: 80,
		usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
		mapped_at_creation: false,
	});

	let mappable_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		label: Some("Mappable Buffer"),
		size: 80,
		usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
		mapped_at_creation: false,
	});

	// Create a command encoder and compute pass
	let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
		label: Some("Compute Encoder"),
	});

	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: Some("Compute Bind Group"),
		layout: &bind_group_layout,
		entries: &[
			// Bind the input buffer
			wgpu::BindGroupEntry {
				binding: 0,
				resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
					buffer: &input_header_buffer,
					offset: 0,
					size: None,
				}),
			},
			// Bind the input size buffer
			wgpu::BindGroupEntry {
				binding: 1,
				resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
					buffer: &input_target_buffer,
					offset: 0,
					size: None,
				}),
			},
			// Bind the atomic flag buffer
			wgpu::BindGroupEntry {
				binding: 2,
				resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
					buffer: &atomic_flag_buffer,
					offset: 0,
					size: None,
				}),
			},
			// Bind the output buffer
			wgpu::BindGroupEntry {
				binding: 3,
				resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
					buffer: &output_buffer,
					offset: 0,
					size: None,
				}),
			},
		],
	});

	{
		let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
			label: Some("Compute Pass"),
			timestamp_writes: None,
		});
		compute_pass.set_pipeline(&compute_pipeline);
		compute_pass.set_bind_group(0, &bind_group, &[]);
		// NOTE: when modifying this value, also change `numWorkgroups` in sha256.wgsl
		compute_pass.dispatch_workgroups(256, 1, 1);
	}

	encoder.copy_buffer_to_buffer(&output_buffer, 0, &mappable_buffer, 0, 32);

	queue.submit(Some(encoder.finish()));

	let buffer_slice = mappable_buffer.slice(..);
	let (tx, rx) = oneshot::channel();

	buffer_slice.map_async(wgpu::MapMode::Read, |res| {
		tx.send(res).unwrap();
	});

	device.poll(wgpu::Maintain::Wait);

	if let Ok(()) = rx.recv().unwrap() {
		let data = buffer_slice.get_mapped_range();
		// Cast `data` to your data type and process it
		// print out the hash as hex
		println!("{:?}", hex::encode(&data));

		drop(data);
		mappable_buffer.unmap();
	} else {
		eprintln!("Error reading data from GPU!");
	}
}
